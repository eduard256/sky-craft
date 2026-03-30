// Per-connection handler. Manages login phase, then game packet routing.

use std::sync::Arc;
use quinn::Connection;
use skycraft_protocol::codec;
use skycraft_protocol::packets::*;
use skycraft_protocol::types::*;
use skycraft_protocol::PROTOCOL_VERSION;
use tracing::{info, warn, debug};
use uuid::Uuid;

use crate::auth;
use crate::config::ServerConfig;
use crate::game::GameState;
use crate::player::Player;

/// Handle a single client connection through login and play phases.
pub async fn handle_connection(
    conn: Connection,
    config: Arc<ServerConfig>,
    game_state: Arc<GameState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let remote = conn.remote_address();

    // Open bidirectional stream for control channel (login + keepalive)
    let (mut send, mut recv) = conn.accept_bi().await?;

    // ── Login Phase ──
    let mut buf = vec![0u8; 4096];
    let n = recv.read(&mut buf).await?
        .ok_or("connection closed during login")?;

    let (login_packet, _) = codec::decode_client_packet(&buf[..n])?
        .ok_or("incomplete login packet")?;

    let (auth_token, protocol_version) = match login_packet {
        ClientPacket::Login(login) => (login.auth_token, login.protocol_version),
        _ => {
            let disconnect = ServerPacket::Disconnect(S2CDisconnect {
                reason: "Expected login packet".to_string(),
            });
            let bytes = codec::encode_server_packet(&disconnect)?;
            send.write_all(&bytes).await?;
            send.finish()?;
            return Ok(());
        }
    };

    // Check protocol version
    if protocol_version != PROTOCOL_VERSION {
        let disconnect = ServerPacket::Disconnect(S2CDisconnect {
            reason: format!(
                "Protocol version mismatch: client={}, server={}",
                protocol_version, PROTOCOL_VERSION
            ),
        });
        let bytes = codec::encode_server_packet(&disconnect)?;
        send.write_all(&bytes).await?;
        send.finish()?;
        return Ok(());
    }

    // Check player count
    if game_state.player_count() >= config.max_players as usize {
        let disconnect = ServerPacket::Disconnect(S2CDisconnect {
            reason: "Server is full".to_string(),
        });
        let bytes = codec::encode_server_packet(&disconnect)?;
        send.write_all(&bytes).await?;
        send.finish()?;
        return Ok(());
    }

    // Validate auth token
    let nickname = match auth::validate_token(&config.auth_api_url, &auth_token).await {
        auth::AuthResult::Ok(nick) => nick,
        auth::AuthResult::Invalid => {
            let disconnect = ServerPacket::Disconnect(S2CDisconnect {
                reason: "Invalid session. Please log in again.".to_string(),
            });
            let bytes = codec::encode_server_packet(&disconnect)?;
            send.write_all(&bytes).await?;
            send.finish()?;
            return Ok(());
        }
        auth::AuthResult::ServiceError(e) => {
            warn!("Auth service error for {}: {}", remote, e);
            let disconnect = ServerPacket::Disconnect(S2CDisconnect {
                reason: "Auth service unavailable. Try again later.".to_string(),
            });
            let bytes = codec::encode_server_packet(&disconnect)?;
            send.write_all(&bytes).await?;
            send.finish()?;
            return Ok(());
        }
    };

    // Check if player is already online
    if game_state.is_player_online(&nickname) {
        let disconnect = ServerPacket::Disconnect(S2CDisconnect {
            reason: "Already connected from another client".to_string(),
        });
        let bytes = codec::encode_server_packet(&disconnect)?;
        send.write_all(&bytes).await?;
        send.finish()?;
        return Ok(());
    }

    info!("Player {} logged in from {}", nickname, remote);

    // Create player and add to game
    let player_uuid = Uuid::new_v4();
    let spawn_pos = game_state.world.get_spawn_position();

    let player = Player::new(
        player_uuid,
        nickname.clone(),
        spawn_pos,
        config.difficulty_enum(),
    );

    let entity_id = game_state.add_player(player);

    // Send login success
    let login_success = ServerPacket::LoginSuccess(S2CLoginSuccess {
        player_uuid,
        nickname: nickname.clone(),
        game_mode: GameMode::Survival,
        difficulty: config.difficulty_enum(),
        spawn_position: spawn_pos,
        world_seed: config.seed,
        view_distance: config.view_distance,
    });
    let bytes = codec::encode_server_packet(&login_success)?;
    send.write_all(&bytes).await?;

    // Send initial time
    let time_packet = ServerPacket::TimeUpdate(S2CTimeUpdate {
        world_age: game_state.world_age(),
        time_of_day: game_state.time_of_day(),
    });
    let bytes = codec::encode_server_packet(&time_packet)?;
    send.write_all(&bytes).await?;

    // Send initial weather
    let weather_packet = ServerPacket::WeatherChange(S2CWeatherChange {
        weather: game_state.current_weather(),
    });
    let bytes = codec::encode_server_packet(&weather_packet)?;
    send.write_all(&bytes).await?;

    // Send chunks around spawn
    send_initial_chunks(&mut send, &game_state, spawn_pos, config.view_distance).await?;

    // Send player position
    let pos_packet = ServerPacket::PlayerPositionAndLook(S2CPlayerPositionAndLook {
        x: spawn_pos.x,
        y: spawn_pos.y,
        z: spawn_pos.z,
        yaw: 0.0,
        pitch: 0.0,
    });
    let bytes = codec::encode_server_packet(&pos_packet)?;
    send.write_all(&bytes).await?;

    // Send initial inventory (empty)
    let inv_packet = ServerPacket::WindowItems(S2CWindowItems {
        window_id: 0,
        slots: vec![None; 46], // 36 inventory + 4 armor + 1 offhand + 4 crafting + 1 output
    });
    let bytes = codec::encode_server_packet(&inv_packet)?;
    send.write_all(&bytes).await?;

    // Send initial health
    let health_packet = ServerPacket::UpdateHealth(S2CUpdateHealth {
        health: 20.0,
        food: 20,
        saturation: 5.0,
    });
    let bytes = codec::encode_server_packet(&health_packet)?;
    send.write_all(&bytes).await?;

    // Notify other players
    let add_packet = ServerPacket::PlayerListUpdate(S2CPlayerListUpdate {
        action: PlayerListAction::Add {
            uuid: player_uuid,
            nickname: nickname.clone(),
            game_mode: GameMode::Survival,
            ping_ms: 0,
        },
    });
    game_state.broadcast_packet(&add_packet, Some(entity_id));

    // Spawn player entity for others
    let spawn_packet = ServerPacket::SpawnPlayer(S2CSpawnPlayer {
        entity_id,
        player_uuid,
        nickname: nickname.clone(),
        position: spawn_pos,
        rotation: Rotation { yaw: 0.0, pitch: 0.0 },
    });
    game_state.broadcast_packet(&spawn_packet, Some(entity_id));

    // ── Play Phase: main packet loop ──
    let result = play_loop(&mut send, &mut recv, entity_id, &game_state).await;

    // Player disconnected - cleanup
    info!("Player {} disconnected", nickname);
    game_state.remove_player(entity_id);

    // Notify others
    let remove_packet = ServerPacket::PlayerListUpdate(S2CPlayerListUpdate {
        action: PlayerListAction::Remove { uuid: player_uuid },
    });
    game_state.broadcast_packet(&remove_packet, None);

    let destroy_packet = ServerPacket::DestroyEntities(S2CDestroyEntities {
        entity_ids: vec![entity_id],
    });
    game_state.broadcast_packet(&destroy_packet, None);

    result
}

/// Send chunks around a position within view distance.
async fn send_initial_chunks(
    send: &mut quinn::SendStream,
    game_state: &GameState,
    pos: EntityPos,
    view_distance: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let center_chunk = pos.to_block_pos().to_chunk_pos();
    let vd = view_distance as i32;

    for cx in (center_chunk.x - vd)..=(center_chunk.x + vd) {
        for cz in (center_chunk.z - vd)..=(center_chunk.z + vd) {
            // Only send chunks that have actual content (not all-air void)
            for cy in -4..20 {
                let chunk_pos = ChunkPos::new(cx, cy, cz);
                let section = game_state.world.get_or_generate_chunk(chunk_pos);
                if !section.is_empty() {
                    let packet = ServerPacket::ChunkData(S2CChunkData {
                        chunk_pos,
                        section,
                    });
                    let bytes = codec::encode_server_packet(&packet)?;
                    send.write_all(&bytes).await?;
                }
            }
        }
    }

    debug!("Sent initial chunks around ({}, {}, {})", center_chunk.x, center_chunk.y, center_chunk.z);
    Ok(())
}

/// Main play phase loop: read client packets, process, send responses.
async fn play_loop(
    send: &mut quinn::SendStream,
    recv: &mut quinn::RecvStream,
    entity_id: EntityId,
    game_state: &GameState,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = vec![0u8; 65536];
    let mut read_buf = Vec::new();

    loop {
        // Check if we have outbound packets queued for this player
        while let Some(packet) = game_state.pop_outbound_packet(entity_id) {
            let bytes = codec::encode_server_packet(&packet)?;
            send.write_all(&bytes).await?;
        }

        // Read incoming data with timeout
        tokio::select! {
            result = recv.read(&mut buf) => {
                match result {
                    Ok(Some(n)) => {
                        read_buf.extend_from_slice(&buf[..n]);
                    }
                    Ok(None) | Err(_) => {
                        // Connection closed
                        return Ok(());
                    }
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(5)) => {
                // No data available, continue to process outbound
                continue;
            }
        }

        // Process all complete packets in buffer
        loop {
            match codec::decode_client_packet(&read_buf) {
                Ok(Some((packet, consumed))) => {
                    read_buf.drain(..consumed);
                    handle_client_packet(entity_id, packet, game_state);
                }
                Ok(None) => break, // need more data
                Err(e) => {
                    warn!("Packet decode error for entity {}: {}", entity_id, e);
                    return Ok(());
                }
            }
        }
    }
}

/// Process a single client packet.
fn handle_client_packet(
    entity_id: EntityId,
    packet: ClientPacket,
    game_state: &GameState,
) {
    match packet {
        ClientPacket::PlayerPosition(pos) => {
            game_state.update_player_position(entity_id, EntityPos::new(pos.x, pos.y, pos.z), pos.on_ground);
        }
        ClientPacket::PlayerLook(look) => {
            game_state.update_player_look(entity_id, look.yaw, look.pitch);
        }
        ClientPacket::PlayerPositionAndLook(pal) => {
            game_state.update_player_position(entity_id, EntityPos::new(pal.x, pal.y, pal.z), pal.on_ground);
            game_state.update_player_look(entity_id, pal.yaw, pal.pitch);
        }
        ClientPacket::BlockDig(dig) => {
            game_state.handle_block_dig(entity_id, dig);
        }
        ClientPacket::BlockPlace(place) => {
            game_state.handle_block_place(entity_id, place);
        }
        ClientPacket::ChatMessage(msg) => {
            game_state.handle_chat(entity_id, msg.message);
        }
        ClientPacket::KeepAliveResponse(ka) => {
            game_state.handle_keep_alive_response(entity_id, ka.id);
        }
        ClientPacket::HeldItemChange(h) => {
            game_state.update_held_item(entity_id, h.slot);
        }
        ClientPacket::SwingArm(swing) => {
            // Broadcast arm swing animation to other players
            let anim = ServerPacket::EntityAnimation(S2CEntityAnimation {
                entity_id,
                animation: match swing.hand {
                    Hand::Main => AnimationType::SwingMainArm,
                    Hand::Off => AnimationType::SwingOffhand,
                },
            });
            game_state.broadcast_packet(&anim, Some(entity_id));
        }
        ClientPacket::EntityInteract(interact) => {
            game_state.handle_entity_interact(entity_id, interact);
        }
        ClientPacket::UseItem(use_item) => {
            game_state.handle_use_item(entity_id, use_item);
        }
        ClientPacket::ClickSlot(click) => {
            game_state.handle_click_slot(entity_id, click);
        }
        ClientPacket::CloseWindow(close) => {
            game_state.handle_close_window(entity_id, close.window_id);
        }
        ClientPacket::ClientSettings(settings) => {
            game_state.update_client_settings(entity_id, settings);
        }
        ClientPacket::PlayerAction(action) => {
            game_state.handle_player_action(entity_id, action);
        }
        // Sky Craft specific
        ClientPacket::PlaceMarker(marker) => {
            game_state.handle_place_marker(entity_id, marker);
        }
        ClientPacket::RemoveMarker(marker) => {
            game_state.handle_remove_marker(entity_id, marker);
        }
        ClientPacket::UseGrapplingHook(hook) => {
            game_state.handle_grappling_hook(entity_id, hook);
        }
        ClientPacket::UseEmergencyRecall => {
            game_state.handle_emergency_recall(entity_id);
        }
        _ => {
            debug!("Unhandled packet from entity {}: {:?}", entity_id, std::mem::discriminant(&packet));
        }
    }
}
