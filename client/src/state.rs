// Application state and winit event handling with egui UI.
// Integrates: renderer, camera, hand, atlas, pipeline, mesh building, network.

use std::sync::Arc;
use std::sync::mpsc;
use std::collections::HashSet;
use winit::application::ApplicationHandler;
use winit::event::{WindowEvent, DeviceEvent, DeviceId};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId, WindowAttributes, CursorGrabMode};
use tracing::{info, warn};

use crate::asset_downloader::{self, DownloadProgress};

use skycraft_protocol::types::ChunkPos;

use crate::renderer::Renderer;
use crate::input::InputState;
use crate::world::ClientWorld;
use crate::session;
use crate::auth_client;
use crate::net_bridge::NetBridge;
use crate::atlas::TextureAtlas;
use crate::pipeline::RenderPipeline;
use crate::camera::Camera;
use crate::hand::Hand;
use crate::mesh;

/// Application state machine.
#[derive(Debug, Clone, PartialEq)]
pub enum AppScreen {
    MainMenu,
    Login,
    Connecting,
    Playing,
}

#[derive(Debug, Clone, PartialEq)]
enum LoginStep {
    EnterNickname,
    EnterCode,
}

/// UI-only state, separated to avoid borrow conflicts with egui.
struct UiState {
    screen: AppScreen,
    server_address: String,
    nickname: String,
    auth_code: String,
    status_message: String,
    saved_token: Option<String>,
    login_step: LoginStep,
}

/// State for the asset download modal shown over the main menu.
struct AssetDownloadUi {
    /// Whether the modal should be visible.
    show: bool,
    /// Human-readable status line shown below the progress bar.
    status: String,
    /// Progress value in 0.0–1.0. Negative means indeterminate (spinner).
    progress: f32,
    /// Set to true on error so the user can dismiss and retry later.
    has_error: bool,
}

/// Action returned from UI drawing (deferred execution after egui frame).
enum UiAction {
    None,
    RequestCode,
    VerifyCode,
    Connect,
    Quit,
    Logout,
    GoToLogin,
    GoToMainMenu,
    LoginBackToNickname,
}

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    input: InputState,
    world: ClientWorld,

    egui_ctx: egui::Context,
    egui_state: Option<egui_winit::State>,
    egui_renderer: Option<egui_wgpu::Renderer>,

    ui: UiState,

    /// Network bridge (alive while connected to server).
    net: Option<NetBridge>,

    /// Texture atlas (built once on first connect).
    atlas: Option<TextureAtlas>,

    /// Render pipeline (built after atlas).
    game_pipeline: Option<RenderPipeline>,

    /// First-person camera.
    camera: Camera,

    /// Player velocity (m/s), used for client-side physics.
    velocity: glam::Vec3,

    /// Whether player is on the ground.
    on_ground: bool,

    /// Accumulator for sending position packets (send every ~50ms).
    move_send_timer: f32,

    /// Hand model + animation.
    hand: Hand,

    /// Chunks that have been meshed (to avoid re-meshing).
    meshed_chunks: HashSet<ChunkPos>,

    /// Chunks that need remesh (e.g. neighbor arrived).
    dirty_chunks: HashSet<ChunkPos>,

    /// Whether inventory is open.
    inventory_open: bool,

    /// Set to true if init_game_rendering failed so we don't spam retries every frame.
    rendering_init_failed: bool,

    /// Receiver end of the asset download progress channel.
    /// None when no download is in progress (or already done).
    download_rx: Option<mpsc::Receiver<DownloadProgress>>,

    /// State for the download modal overlay.
    download_ui: AssetDownloadUi,

    /// Frame timing.
    last_frame_time: std::time::Instant,
}

impl App {
    pub fn new() -> Self {
        let (saved_token, nickname) = match session::load_session() {
            Some(s) => {
                info!("Loaded saved session for {}", s.nickname);
                (Some(s.token), s.nickname)
            }
            None => (None, String::new()),
        };

        // Check whether game assets exist. If not, start downloading immediately
        // so by the time the window appears the download is already in progress.
        let (download_rx, download_ui) = if asset_downloader::check_assets() {
            (None, AssetDownloadUi {
                show: false,
                status: String::new(),
                progress: 0.0,
                has_error: false,
            })
        } else {
            info!("Game assets missing — starting download...");
            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || asset_downloader::download_assets(tx));
            (Some(rx), AssetDownloadUi {
                show: true,
                status: "Connecting to GitHub...".to_string(),
                progress: -1.0, // indeterminate until first byte arrives
                has_error: false,
            })
        };

        Self {
            window: None,
            renderer: None,
            input: InputState::new(),
            world: ClientWorld::new(),
            egui_ctx: egui::Context::default(),
            egui_state: None,
            egui_renderer: None,
            net: None,
            atlas: None,
            game_pipeline: None,
            camera: Camera::new(8),
            velocity: glam::Vec3::ZERO,
            on_ground: true,
            move_send_timer: 0.0,
            hand: Hand::new(),
            meshed_chunks: HashSet::new(),
            dirty_chunks: HashSet::new(),
            inventory_open: false,
            rendering_init_failed: false,
            download_rx,
            download_ui,
            last_frame_time: std::time::Instant::now(),
            ui: UiState {
                screen: AppScreen::MainMenu,
                server_address: "127.0.0.1".to_string(),
                nickname,
                auth_code: String::new(),
                status_message: String::new(),
                saved_token,
                login_step: LoginStep::EnterNickname,
            },
        }
    }

    /// Build atlas and pipeline (called once after first connection).
    fn init_game_rendering(&mut self) {
        let Some(renderer) = &self.renderer else { return };
        if self.atlas.is_some() { return; } // already initialized
        if self.rendering_init_failed { return; } // don't retry after a fatal error

        // Resolve asset paths. In dev mode (cargo run from repo root) these point into
        // client/assets/textures; in release builds they point next to the exe.
        let textures_dir = asset_downloader::assets_dir();
        let block_textures = textures_dir.join("block");
        let steve_skin = textures_dir.join("entity/steve.png");

        // data dir: always relative to CWD in dev, or next to exe in release.
        let data_dir = if std::path::Path::new("common/data").exists() {
            std::path::PathBuf::from("common/data")
        } else if let Ok(exe) = std::env::current_exe() {
            exe.parent().unwrap_or(std::path::Path::new(".")).join("data")
        } else {
            std::path::PathBuf::from("data")
        };

        info!("Building texture atlas...");
        let atlas = match TextureAtlas::build(
            renderer.device(),
            renderer.queue(),
            data_dir.to_str().unwrap_or("common/data"),
            block_textures.to_str().unwrap_or("client/assets/textures/minecraft/textures/block"),
        ) {
            Ok(a) => a,
            Err(e) => {
                warn!("Failed to build atlas: {}", e);
                self.rendering_init_failed = true;
                return;
            }
        };

        info!("Creating render pipeline...");
        let pipeline = match RenderPipeline::new(
            renderer.device(),
            renderer.queue(),
            renderer.surface_format(),
            renderer.size().width,
            renderer.size().height,
            &atlas,
            steve_skin.to_str().unwrap_or("client/assets/textures/minecraft/textures/entity/steve.png"),
        ) {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to create pipeline: {}", e);
                self.rendering_init_failed = true;
                return;
            }
        };

        self.atlas = Some(atlas);
        self.game_pipeline = Some(pipeline);
        info!("Game rendering initialized");
    }

    /// Build meshes for any new chunks that arrived from server.
    fn unload_distant_chunks(&mut self) {
        const UNLOAD_DISTANCE: i32 = 12; // chunks beyond this get dropped

        let px = (self.world.player.position.x / 16.0).floor() as i32;
        let pz = (self.world.player.position.z / 16.0).floor() as i32;

        let to_unload: Vec<ChunkPos> = self.world.loaded_chunk_positions()
            .filter(|pos| (pos.x - px).abs() > UNLOAD_DISTANCE || (pos.z - pz).abs() > UNLOAD_DISTANCE)
            .collect();

        for pos in to_unload {
            self.world.unload_chunk(&pos);
            self.meshed_chunks.remove(&pos);
            self.dirty_chunks.remove(&pos);
            if let Some(pipeline) = &mut self.game_pipeline {
                pipeline.remove_chunk_mesh(pos);
            }
        }
    }

    fn mesh_new_chunks(&mut self) {
        let Some(atlas) = &self.atlas else { return };
        let Some(pipeline) = &mut self.game_pipeline else { return };
        let Some(renderer) = &self.renderer else { return };

        // Find chunks that exist in world but haven't been meshed
        let new_chunks: Vec<ChunkPos> = self.world.loaded_chunk_positions()
            .filter(|pos| !self.meshed_chunks.contains(pos) && !self.dirty_chunks.contains(pos))
            .collect();

        // When a new chunk arrives, mark neighbors dirty (once, not every frame)
        for chunk_pos in &new_chunks {
            for n in [
                ChunkPos::new(chunk_pos.x+1, chunk_pos.y, chunk_pos.z),
                ChunkPos::new(chunk_pos.x-1, chunk_pos.y, chunk_pos.z),
                ChunkPos::new(chunk_pos.x, chunk_pos.y, chunk_pos.z+1),
                ChunkPos::new(chunk_pos.x, chunk_pos.y, chunk_pos.z-1),
            ] {
                if self.meshed_chunks.contains(&n) {
                    self.meshed_chunks.remove(&n);
                    self.dirty_chunks.insert(n);
                }
            }
        }

        // Process new chunks + up to 4 dirty chunks per frame
        let dirty_batch: Vec<ChunkPos> = self.dirty_chunks.iter().take(4).copied().collect();
        for pos in &dirty_batch { self.dirty_chunks.remove(pos); }
        let chunks_to_mesh: Vec<ChunkPos> = new_chunks.into_iter().chain(dirty_batch).collect();

        for chunk_pos in chunks_to_mesh {
            let section = match self.world.get_chunk(&chunk_pos) {
                Some(s) => s,
                None => continue,
            };

            if section.is_empty() {
                self.meshed_chunks.insert(chunk_pos);
                continue;
            }

            // Build mesh with neighbor lookup
            let world_ref = &self.world;
            let chunk_mesh = mesh::build_chunk_mesh(
                chunk_pos,
                &section,
                atlas,
                &|wx, wy, wz| {
                    use skycraft_protocol::types::BlockPos;
                    world_ref.get_block(BlockPos::new(wx, wy, wz))
                },
            );

            if !chunk_mesh.is_empty() {
                pipeline.upload_chunk_mesh(renderer.device(), &chunk_mesh);
            }

            self.meshed_chunks.insert(chunk_pos);
        }
    }

    /// Update camera and hand each frame.
    fn update_game(&mut self, dt: f32) {
        // Toggle inventory
        if self.input.is_inventory_pressed() {
            self.inventory_open = !self.inventory_open;
            if let Some(window) = &self.window {
                if self.inventory_open {
                    let _ = window.set_cursor_grab(CursorGrabMode::None);
                    window.set_cursor_visible(true);
                } else {
                    let _ = window.set_cursor_grab(CursorGrabMode::Locked)
                        .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
                    window.set_cursor_visible(false);
                }
            }
        }

        // Mouse look (only when playing and inventory closed)
        if self.ui.screen == AppScreen::Playing && !self.inventory_open {
            self.camera.process_mouse(self.input.mouse_dx, self.input.mouse_dy);
        }

        // ── Client-side movement ─────────────────────────────────────────────
        const WALK_SPEED: f32 = 8.6;
        const SPRINT_SPEED: f32 = 34.4;

        let speed = if self.input.is_sprint() { SPRINT_SPEED } else { WALK_SPEED };

        // Horizontal movement direction from input
        let mut move_dir = glam::Vec3::ZERO;
        if self.input.is_forward()  { move_dir += self.camera.forward_xz(); }
        if self.input.is_backward() { move_dir -= self.camera.forward_xz(); }
        if self.input.is_right()    { move_dir -= self.camera.right_xz(); }
        if self.input.is_left()     { move_dir += self.camera.right_xz(); }
        if move_dir.length_squared() > 0.0 { move_dir = move_dir.normalize(); }

        let dx = move_dir.x * speed * dt;
        let dz = move_dir.z * speed * dt;

        let cur_x = self.world.player.position.x;
        let cur_y = self.world.player.position.y;
        let cur_z = self.world.player.position.z;
        const PLAYER_WIDTH: f64 = 0.3; // half-width
        const PLAYER_HEIGHT_COL: f64 = 1.8;

        // Helper: check if AABB at (cx, cy, cz) overlaps any solid block
        let is_blocked = |cx: f64, cy: f64, cz: f64| -> bool {
            use skycraft_protocol::types::BlockPos;
            let x0 = (cx - PLAYER_WIDTH).floor() as i32;
            let x1 = (cx + PLAYER_WIDTH).floor() as i32;
            let y0 = cy.floor() as i32;
            let y1 = (cy + PLAYER_HEIGHT_COL - 0.01).floor() as i32;
            let z0 = (cz - PLAYER_WIDTH).floor() as i32;
            let z1 = (cz + PLAYER_WIDTH).floor() as i32;
            for bx in x0..=x1 {
                for by in y0..=y1 {
                    for bz in z0..=z1 {
                        if self.world.get_block(BlockPos::new(bx, by, bz)) != 0 {
                            return true;
                        }
                    }
                }
            }
            false
        };

        let new_x = cur_x + dx as f64;
        let new_z = cur_z + dz as f64;

        let dest_chunk = skycraft_protocol::types::BlockPos { x: new_x.floor() as i32, y: 64, z: new_z.floor() as i32 }.to_chunk_pos();
        if self.world.is_chunk_loaded(&dest_chunk) {
            let can_x = !is_blocked(new_x, cur_y, cur_z);
            let can_z = !is_blocked(if can_x { new_x } else { cur_x }, cur_y, new_z);
            if can_x { self.world.player.position.x = new_x; }
            if can_z { self.world.player.position.z = new_z; }
        }

        // ── Gravity + jump ───────────────────────────────────────────────────
        const GRAVITY: f32 = -20.0;
        const JUMP_SPEED: f32 = 7.0;
        const PLAYER_HEIGHT: f64 = 1.8;

        // Jump
        if self.input.is_jump() && self.on_ground {
            self.velocity.y = JUMP_SPEED;
            self.on_ground = false;
        }

        // Apply gravity only if chunk below is loaded
        let px2 = self.world.player.position.x;
        let pz2 = self.world.player.position.z;
        let py_cur = self.world.player.position.y;
        let below_chunk = skycraft_protocol::types::BlockPos {
            x: px2.floor() as i32,
            y: (py_cur - 1.0).floor() as i32,
            z: pz2.floor() as i32,
        }.to_chunk_pos();
        let chunk_below_loaded = self.world.is_chunk_loaded(&below_chunk);

        if !self.on_ground && chunk_below_loaded {
            self.velocity.y += GRAVITY * dt;
        } else if !chunk_below_loaded {
            // Freeze until chunks load
            self.velocity.y = 0.0;
            self.on_ground = true;
        }

        // Move Y — copy coords to avoid borrow conflict
        let mut py = py_cur;
        py += self.velocity.y as f64 * dt as f64;

        let foot_block = self.world.get_block(skycraft_protocol::types::BlockPos {
            x: px2.floor() as i32,
            y: (py - 0.01).floor() as i32,
            z: pz2.floor() as i32,
        });
        if foot_block != 0 && self.velocity.y <= 0.0 {
            py = (py - 0.01).floor() + 1.0;
            self.velocity.y = 0.0;
            self.on_ground = true;
        } else if foot_block == 0 {
            self.on_ground = false;
        }

        let head_block = self.world.get_block(skycraft_protocol::types::BlockPos {
            x: px2.floor() as i32,
            y: (py + PLAYER_HEIGHT).floor() as i32,
            z: pz2.floor() as i32,
        });
        if head_block != 0 && self.velocity.y > 0.0 {
            self.velocity.y = 0.0;
        }

        self.world.player.position.y = py;
        // ────────────────────────────────────────────────────────────────────

        // Sync camera to player position
        self.camera.position = glam::Vec3::new(
            self.world.player.position.x as f32,
            self.world.player.position.y as f32,
            self.world.player.position.z as f32,
        );

        // Send position to server every 50ms
        self.move_send_timer += dt;
        if self.move_send_timer >= 0.05 {
            self.move_send_timer = 0.0;
            let px = self.world.player.position.x;
            let py = self.world.player.position.y;
            let pz = self.world.player.position.z;
            if let Some(net) = &self.net {
                net.send(skycraft_protocol::packets::ClientPacket::PlayerPositionAndLook(
                    skycraft_protocol::packets::C2SPlayerPositionAndLook {
                        x: px, y: py, z: pz,
                        yaw: self.camera.yaw,
                        pitch: self.camera.pitch,
                        on_ground: true,
                    }
                ));
            }
        }
        // ────────────────────────────────────────────────────────────────────

        // Walking state
        let is_walking = self.input.is_forward() || self.input.is_backward()
            || self.input.is_left() || self.input.is_right();
        self.camera.is_walking = is_walking;
        self.camera.is_sprinting = self.input.is_sprint() && is_walking;

        // Update camera (FOV smoothing, bobbing)
        self.camera.update(dt);

        // Sync hand with camera
        self.hand.is_walking = is_walking;
        self.hand.walk_cycle = self.camera.walk_cycle;
        self.hand.update(dt);

        // Left click -> swing hand
        if self.input.mouse_clicked.contains(&winit::event::MouseButton::Left) {
            self.hand.start_swing();
        }

        // Update camera aspect ratio
        if let Some(renderer) = &self.renderer {
            let size = renderer.size();
            self.camera.set_aspect(size.width, size.height);
        }

        // Update GPU uniforms
        if let (Some(pipeline), Some(renderer)) = (&self.game_pipeline, &self.renderer) {
            // Camera uniform
            let time_normalized = self.world.time_of_day as f32 / 24000.0;
            let camera_uniform = self.camera.build_uniform(time_normalized);
            pipeline.update_camera(renderer.queue(), &camera_uniform);

            // Hand uniform: rendered in view-space (fixed to screen)
            let hand_model = self.hand.model_matrix();
            // Hand uses simple orthographic-like projection for screen-space
            let hand_view_proj = glam::Mat4::perspective_rh(
                70.0_f32.to_radians(),
                self.camera.aspect,
                0.01,
                10.0,
            );
            let hand_uniform = crate::hand::HandUniform {
                model: hand_model.to_cols_array(),
                view_proj: hand_view_proj.to_cols_array(),
            };
            pipeline.update_hand(renderer.queue(), &hand_uniform);
        }

        // Rebuild mob meshes (upload per-entity cow geometry each frame)
        if let (Some(pipeline), Some(renderer)) = (&mut self.game_pipeline, &self.renderer) {
            // Remove meshes for entities that are no longer present
            let entity_ids: Vec<_> = pipeline.mob_meshes.keys().copied().collect();
            for eid in entity_ids {
                if !self.world.entities.contains_key(&eid) {
                    pipeline.remove_mob_mesh(eid);
                }
            }
            // Build/update mesh for each entity (type 11 = cow)
            for (eid, entity) in &self.world.entities {
                if entity.entity_type != 11 { continue; }
                let pos = glam::Vec3::new(
                    entity.position.x as f32,
                    entity.position.y as f32,
                    entity.position.z as f32,
                );
                let yaw = entity.rotation.yaw.to_radians();
                let (verts, idxs) = crate::cow::build_cow_mesh(pos, yaw);
                pipeline.upload_mob_mesh(renderer.device(), *eid, &verts, &idxs);
            }
        }
    }

    /// Drain all pending download progress messages and update download_ui accordingly.
    fn drain_download_progress(&mut self) {
        // Take ownership temporarily to avoid borrow issues.
        let rx = match self.download_rx.take() {
            Some(r) => r,
            None => return,
        };

        let mut done = false;

        while let Ok(msg) = rx.try_recv() {
            match msg {
                DownloadProgress::Downloading { downloaded_bytes, total_bytes } => {
                    if let Some(total) = total_bytes {
                        // First half of the overall progress bar is the download phase.
                        let frac = downloaded_bytes as f32 / total as f32;
                        self.download_ui.progress = frac * 0.5;
                        self.download_ui.status = format!(
                            "Downloading textures... {:.1} / {:.1} MB",
                            downloaded_bytes as f64 / 1_048_576.0,
                            total as f64 / 1_048_576.0,
                        );
                    } else {
                        self.download_ui.progress = -1.0; // indeterminate
                        self.download_ui.status = format!(
                            "Downloading textures... {:.1} MB",
                            downloaded_bytes as f64 / 1_048_576.0,
                        );
                    }
                }
                DownloadProgress::Extracting { current, total } => {
                    let frac = current as f32 / total as f32;
                    // Second half of progress bar is extraction phase.
                    self.download_ui.progress = 0.5 + frac * 0.5;
                    self.download_ui.status = format!(
                        "Extracting textures... {}/{}", current, total
                    );
                }
                DownloadProgress::Done => {
                    self.download_ui.progress = 1.0;
                    self.download_ui.status = "Done!".to_string();
                    self.download_ui.show = false;
                    done = true;
                    info!("Asset download complete");
                }
                DownloadProgress::Error(e) => {
                    self.download_ui.status = format!("Download failed: {}", e);
                    self.download_ui.has_error = true;
                    done = true;
                    warn!("Asset download error: {}", e);
                }
            }
        }

        // Put the receiver back unless the channel is finished.
        if !done {
            self.download_rx = Some(rx);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        info!("Creating window...");
        let attrs = WindowAttributes::default()
            .with_title("Sky Craft")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

        match event_loop.create_window(attrs) {
            Ok(window) => {
                let window = Arc::new(window);

                let renderer = pollster::block_on(Renderer::new(window.clone()));
                match renderer {
                    Ok(r) => {
                        let egui_state = egui_winit::State::new(
                            self.egui_ctx.clone(),
                            egui::ViewportId::ROOT,
                            &window,
                            Some(window.scale_factor() as f32),
                            None,
                            None,
                        );

                        let egui_renderer = egui_wgpu::Renderer::new(
                            r.device(),
                            r.surface_format(),
                            None,
                            1,
                            false,
                        );

                        self.egui_state = Some(egui_state);
                        self.egui_renderer = Some(egui_renderer);
                        self.renderer = Some(r);
                        info!("Renderer + egui initialized");
                    }
                    Err(e) => {
                        warn!("Failed to init renderer: {}", e);
                        event_loop.exit();
                        return;
                    }
                }
                self.window = Some(window);
            }
            Err(e) => {
                warn!("Failed to create window: {}", e);
                event_loop.exit();
            }
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        if self.ui.screen == AppScreen::Playing {
            if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
                self.input.mouse_dx += dx;
                self.input.mouse_dy += dy;
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // In playing mode, don't let egui consume mouse events (camera needs them)
        let egui_wants = if self.ui.screen == AppScreen::Playing {
            // Only pass keyboard events to egui when playing
            if matches!(event, WindowEvent::KeyboardInput { .. }) {
                if let Some(egui_state) = &mut self.egui_state {
                    let response = egui_state.on_window_event(self.window.as_ref().unwrap(), &event);
                    response.consumed
                } else {
                    false
                }
            } else if matches!(event, WindowEvent::Resized(_) | WindowEvent::CloseRequested | WindowEvent::RedrawRequested) {
                false
            } else {
                // Pass mouse to egui for HUD interaction, but also to input
                if let Some(egui_state) = &mut self.egui_state {
                    egui_state.on_window_event(self.window.as_ref().unwrap(), &event);
                }
                false // don't consume, input needs it too
            }
        } else {
            if let Some(egui_state) = &mut self.egui_state {
                let response = egui_state.on_window_event(self.window.as_ref().unwrap(), &event);
                response.consumed
            } else {
                false
            }
        };

        if !egui_wants {
            self.input.handle_event(&event);
        }

        match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size);
                }
                if let Some(pipeline) = &mut self.game_pipeline {
                    if let Some(renderer) = &self.renderer {
                        pipeline.resize_depth(renderer.device(), new_size.width, new_size.height);
                    }
                }
                self.camera.set_aspect(new_size.width, new_size.height);
            }
            WindowEvent::RedrawRequested => {
                self.do_frame();
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

impl App {
    fn do_frame(&mut self) {
        if self.window.is_none() || self.renderer.is_none()
            || self.egui_state.is_none() || self.egui_renderer.is_none() {
            return;
        }

        // Frame timing
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // ── Phase 1: Game update (no egui borrows) ──────────────────────

        // Drain asset download progress messages
        self.drain_download_progress();

        // Drain network packets
        if let Some(ref net) = self.net {
            for packet in net.drain_packets() {
                self.world.handle_server_packet(packet);
            }
        }

        // Init rendering, mesh chunks, update game
        if self.ui.screen == AppScreen::Playing {
            self.init_game_rendering();
            self.unload_distant_chunks();
            self.mesh_new_chunks();
            self.update_game(dt);
        }

        // ── Phase 2: egui frame ─────────────────────────────────────────

        let window = self.window.as_ref().unwrap();
        let egui_state = self.egui_state.as_mut().unwrap();

        let raw_input = egui_state.take_egui_input(window);
        self.egui_ctx.begin_pass(raw_input);

        let action = draw_ui(&self.egui_ctx, &mut self.ui, &self.world, &self.download_ui, self.inventory_open, self.world.player.held_slot);

        let full_output = self.egui_ctx.end_pass();

        let egui_state = self.egui_state.as_mut().unwrap();
        let window = self.window.as_ref().unwrap();
        egui_state.handle_platform_output(window, full_output.platform_output);

        let paint_jobs = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

        let renderer = self.renderer.as_mut().unwrap();
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [renderer.size().width, renderer.size().height],
            pixels_per_point: full_output.pixels_per_point,
        };

        let egui_renderer = self.egui_renderer.as_mut().unwrap();

        for (id, delta) in &full_output.textures_delta.set {
            egui_renderer.update_texture(renderer.device(), renderer.queue(), *id, delta);
        }

        let mut encoder = renderer.create_encoder();
        egui_renderer.update_buffers(
            renderer.device(),
            renderer.queue(),
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );
        renderer.queue().submit(std::iter::once(encoder.finish()));

        // ── Phase 3: Render ─────────────────────────────────────────────

        let screen = self.ui.screen.clone();
        match renderer.render_frame(
            self.game_pipeline.as_ref(),
            egui_renderer,
            &paint_jobs,
            &screen_descriptor,
            &screen,
        ) {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => {
                let size = self.window.as_ref().unwrap().inner_size();
                self.renderer.as_mut().unwrap().resize(size);
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                warn!("Out of GPU memory");
            }
            Err(e) => {
                warn!("Render error: {:?}", e);
            }
        }

        for id in &full_output.textures_delta.free {
            self.egui_renderer.as_mut().unwrap().free_texture(id);
        }

        // Clear per-frame input
        self.input.begin_frame();

        // Execute deferred action
        self.execute_action(action);
    }

    fn execute_action(&mut self, action: UiAction) {
        match action {
            UiAction::None => {}
            UiAction::Quit => std::process::exit(0),
            UiAction::Logout => {
                session::delete_session();
                self.ui.saved_token = None;
                self.ui.nickname.clear();
                self.ui.status_message = "Logged out".to_string();
            }
            UiAction::GoToLogin => {
                self.ui.screen = AppScreen::Login;
                self.ui.login_step = LoginStep::EnterNickname;
                self.ui.status_message.clear();
            }
            UiAction::GoToMainMenu => {
                self.ui.screen = AppScreen::MainMenu;
                self.ui.status_message.clear();
                // Release cursor when leaving game
                if let Some(window) = &self.window {
                    let _ = window.set_cursor_grab(CursorGrabMode::None);
                    window.set_cursor_visible(true);
                }
            }
            UiAction::LoginBackToNickname => {
                self.ui.login_step = LoginStep::EnterNickname;
                self.ui.auth_code.clear();
                self.ui.status_message.clear();
            }
            UiAction::RequestCode => {
                if self.ui.nickname.len() < 3 {
                    self.ui.status_message = "Nickname must be at least 3 characters".to_string();
                    return;
                }
                self.ui.status_message = "Requesting code...".to_string();
                match auth_client::request_code(&self.ui.nickname) {
                    Ok(msg) => {
                        self.ui.status_message = msg;
                        self.ui.login_step = LoginStep::EnterCode;
                    }
                    Err(e) => { self.ui.status_message = e; }
                }
            }
            UiAction::VerifyCode => {
                if self.ui.auth_code.len() != 6 {
                    self.ui.status_message = "Code must be 6 digits".to_string();
                    return;
                }
                self.ui.status_message = "Verifying...".to_string();
                match auth_client::verify_code(&self.ui.nickname, &self.ui.auth_code) {
                    Ok(token) => {
                        let sess = session::Session {
                            nickname: self.ui.nickname.clone(),
                            token: token.clone(),
                        };
                        let _ = session::save_session(&sess);
                        self.ui.saved_token = Some(token);
                        self.ui.status_message = "Login successful!".to_string();
                        info!("Logged in as {}", self.ui.nickname);
                        self.ui.screen = AppScreen::Connecting;
                        self.try_connect();
                    }
                    Err(e) => { self.ui.status_message = e; }
                }
            }
            UiAction::Connect => {
                if self.ui.saved_token.is_some() {
                    self.ui.screen = AppScreen::Connecting;
                    self.try_connect();
                } else {
                    self.ui.screen = AppScreen::Login;
                    self.ui.login_step = LoginStep::EnterNickname;
                }
            }
        }
    }

    fn try_connect(&mut self) {
        let token = match &self.ui.saved_token {
            Some(t) => t.clone(),
            None => {
                self.ui.status_message = "No auth token. Please login first.".to_string();
                self.ui.screen = AppScreen::Login;
                return;
            }
        };

        let addr = if self.ui.server_address.contains(':') {
            self.ui.server_address.clone()
        } else {
            format!("{}:{}", self.ui.server_address, skycraft_protocol::DEFAULT_PORT)
        };

        self.ui.status_message = format!("Connecting to {}...", addr);

        match NetBridge::connect(addr, token) {
            Ok((bridge, login_success)) => {
                info!("Connected! Player: {} UUID: {}", login_success.nickname, login_success.player_uuid);
                self.ui.nickname = login_success.nickname;
                self.ui.status_message.clear();
                self.ui.screen = AppScreen::Playing;
                self.net = Some(bridge);

                // Grab cursor for FPS-style mouse look
                if let Some(window) = &self.window {
                    let _ = window.set_cursor_grab(CursorGrabMode::Locked)
                        .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
                    window.set_cursor_visible(false);
                }

                // Reset game state for new connection
                self.world = ClientWorld::new();
                self.meshed_chunks.clear();
                self.atlas = None;
                self.game_pipeline = None;
                self.rendering_init_failed = false;
            }
            Err(e) => {
                let err_msg = e.to_string();
                warn!("Connection failed: {}", err_msg);
                if err_msg.contains("Invalid session") || err_msg.contains("401") {
                    session::delete_session();
                    self.ui.saved_token = None;
                    self.ui.status_message = "Session expired. Please login again.".to_string();
                    self.ui.screen = AppScreen::Login;
                } else {
                    self.ui.status_message = format!("Failed: {}", err_msg);
                    self.ui.screen = AppScreen::MainMenu;
                }
            }
        }
    }
}

// ─── UI Drawing ─────────────────────────────────────────────────────────────

fn draw_ui(ctx: &egui::Context, ui: &mut UiState, world: &ClientWorld, dl: &AssetDownloadUi, inventory_open: bool, held_slot: u8) -> UiAction {
    let action = match ui.screen {
        AppScreen::MainMenu => draw_main_menu(ctx, ui),
        AppScreen::Login => draw_login(ctx, ui),
        AppScreen::Connecting => draw_connecting(ctx, ui),
        AppScreen::Playing => draw_playing(ctx, ui, world, inventory_open, held_slot),
    };

    // Asset download modal: shown over any screen (typically MainMenu on first run).
    if dl.show || dl.has_error {
        draw_download_modal(ctx, dl);
    }

    action
}

fn draw_main_menu(ctx: &egui::Context, ui: &mut UiState) -> UiAction {
    let mut action = UiAction::None;
    egui::CentralPanel::default().show(ctx, |p| {
        p.vertical_centered(|p| {
            p.add_space(100.0);
            p.heading(egui::RichText::new("SKY CRAFT").size(48.0).strong());
            p.add_space(40.0);
            p.label("Server address:");
            p.add_space(4.0);
            p.add(egui::TextEdit::singleline(&mut ui.server_address)
                .hint_text("ip or ip:port").desired_width(300.0));
            p.add_space(16.0);
            if p.add_sized([200.0, 40.0], egui::Button::new("Connect")).clicked() {
                action = UiAction::Connect;
            }
            p.add_space(8.0);
            if p.add_sized([200.0, 30.0], egui::Button::new("Quit")).clicked() {
                action = UiAction::Quit;
            }
            if !ui.status_message.is_empty() {
                p.add_space(20.0);
                p.colored_label(egui::Color32::YELLOW, &ui.status_message);
            }
            if ui.saved_token.is_some() {
                p.add_space(20.0);
                p.colored_label(egui::Color32::GREEN, format!("Logged in as: {}", ui.nickname));
                if p.small_button("Logout").clicked() { action = UiAction::Logout; }
            }
            p.add_space(40.0);
            p.colored_label(egui::Color32::DARK_GRAY, format!("v{}", env!("CARGO_PKG_VERSION")));
        });
    });
    action
}

fn draw_login(ctx: &egui::Context, ui: &mut UiState) -> UiAction {
    let mut action = UiAction::None;
    egui::CentralPanel::default().show(ctx, |p| {
        p.vertical_centered(|p| {
            p.add_space(100.0);
            p.heading(egui::RichText::new("Login").size(36.0));
            p.add_space(8.0);
            p.label("Register via Telegram bot: @skycraftauth_bot");
            p.add_space(30.0);
            match ui.login_step {
                LoginStep::EnterNickname => {
                    p.label("Nickname:");
                    p.add_space(4.0);
                    p.add(egui::TextEdit::singleline(&mut ui.nickname)
                        .hint_text("your nickname").desired_width(300.0));
                    p.add_space(16.0);
                    if p.add_sized([200.0, 40.0], egui::Button::new("Request Code")).clicked() {
                        action = UiAction::RequestCode;
                    }
                }
                LoginStep::EnterCode => {
                    p.label(format!("Code sent to Telegram for: {}", ui.nickname));
                    p.add_space(12.0);
                    p.label("Enter 6-digit code:");
                    p.add_space(4.0);
                    p.add(egui::TextEdit::singleline(&mut ui.auth_code)
                        .hint_text("000000").desired_width(200.0));
                    p.add_space(16.0);
                    if p.add_sized([200.0, 40.0], egui::Button::new("Verify")).clicked() {
                        action = UiAction::VerifyCode;
                    }
                    if p.small_button("Back").clicked() { action = UiAction::LoginBackToNickname; }
                }
            }
            if !ui.status_message.is_empty() {
                p.add_space(20.0);
                p.colored_label(egui::Color32::YELLOW, &ui.status_message);
            }
            p.add_space(20.0);
            if p.small_button("Back to menu").clicked() { action = UiAction::GoToMainMenu; }
        });
    });
    action
}

fn draw_connecting(ctx: &egui::Context, ui: &mut UiState) -> UiAction {
    let mut action = UiAction::None;
    egui::CentralPanel::default().show(ctx, |p| {
        p.vertical_centered(|p| {
            p.add_space(200.0);
            p.heading("Connecting...");
            p.add_space(16.0);
            p.spinner();
            p.add_space(16.0);
            if !ui.status_message.is_empty() {
                p.colored_label(egui::Color32::YELLOW, &ui.status_message);
            }
            p.add_space(20.0);
            if p.button("Cancel").clicked() { action = UiAction::GoToMainMenu; }
        });
    });
    action
}

fn draw_slot(ui: &mut egui::Ui, slot: &skycraft_protocol::types::Slot, selected: bool) {
    let size = egui::vec2(40.0, 40.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();

    let bg = if selected { egui::Color32::from_rgb(180, 180, 60) } else { egui::Color32::from_rgba_unmultiplied(40, 40, 40, 200) };
    painter.rect_filled(rect, 3.0, bg);
    painter.rect_stroke(rect, 3.0, (1.0, egui::Color32::from_gray(120)), egui::StrokeKind::Outside);

    if let Some(item) = slot {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("{}", item.item_id),
            egui::FontId::monospace(10.0),
            egui::Color32::WHITE,
        );
        if item.count > 1 {
            painter.text(
                rect.right_bottom() - egui::vec2(2.0, 2.0),
                egui::Align2::RIGHT_BOTTOM,
                format!("{}", item.count),
                egui::FontId::monospace(9.0),
                egui::Color32::WHITE,
            );
        }
    }
}

fn draw_playing(ctx: &egui::Context, ui: &mut UiState, world: &ClientWorld, inventory_open: bool, held_slot: u8) -> UiAction {
    // Crosshair
    egui::Area::new(egui::Id::new("crosshair"))
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .interactable(false)
        .show(ctx, |ui| {
            let painter = ui.painter();
            let center = painter.clip_rect().center();
            let size = 10.0;
            let thickness = 2.0;
            let color = egui::Color32::WHITE;
            let shadow = egui::Color32::from_black_alpha(180);
            // Shadow
            painter.line_segment([egui::pos2(center.x - size + 1.0, center.y + 1.0), egui::pos2(center.x + size + 1.0, center.y + 1.0)], (thickness, shadow));
            painter.line_segment([egui::pos2(center.x + 1.0, center.y - size + 1.0), egui::pos2(center.x + 1.0, center.y + size + 1.0)], (thickness, shadow));
            // Cross
            painter.line_segment([egui::pos2(center.x - size, center.y), egui::pos2(center.x + size, center.y)], (thickness, color));
            painter.line_segment([egui::pos2(center.x, center.y - size), egui::pos2(center.x, center.y + size)], (thickness, color));
        });

    egui::TopBottomPanel::top("top_hud").show(ctx, |p| {
        p.horizontal(|p| {
            p.label(egui::RichText::new("Sky Craft").strong());
            p.separator();
            p.label(format!("Player: {}", ui.nickname));
            p.separator();
            p.label(format!("Chunks: {} | HP: {:.0} | Food: {}",
                world.loaded_chunk_count(), world.player.health, world.player.food));
            p.separator();
            p.label(format!("Pos: {:.1} {:.1} {:.1}",
                world.player.position.x, world.player.position.y, world.player.position.z));
            p.separator();
            p.label(format!("Ring: {}", world.current_ring));
        });
    });

    // Hotbar (bottom center, 9 slots)
    egui::Area::new(egui::Id::new("hotbar"))
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -8.0])
        .interactable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);
                for i in 0..9usize {
                    let slot = world.player.inventory.get(i + 36).cloned().flatten();
                    draw_slot(ui, &slot, i as u8 == held_slot);
                }
            });
        });

    // Full inventory (E key)
    if inventory_open {
        egui::Window::new("Inventory")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Inventory");
                ui.separator();
                // Main inventory: slots 0-35 (4 rows x 9)
                for row in 0..4usize {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
                        for col in 0..9usize {
                            let idx = row * 9 + col;
                            let slot = world.player.inventory.get(idx).cloned().flatten();
                            draw_slot(ui, &slot, false);
                        }
                    });
                }
                ui.separator();
                // Hotbar: slots 36-44
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
                    for i in 0..9usize {
                        let slot = world.player.inventory.get(i + 36).cloned().flatten();
                        draw_slot(ui, &slot, i as u8 == held_slot);
                    }
                });
            });
    }

    UiAction::None
}

fn draw_download_modal(ctx: &egui::Context, dl: &AssetDownloadUi) {
    egui::Window::new("Downloading game assets")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .min_width(380.0)
        .show(ctx, |p| {
            p.vertical_centered(|p| {
                p.add_space(8.0);

                if dl.has_error {
                    p.colored_label(egui::Color32::RED, &dl.status);
                    p.add_space(8.0);
                    p.label("Please run the game again to retry,");
                    p.label("Check your internet connection and restart the game.");
                } else if dl.progress < 0.0 {
                    // Indeterminate — no Content-Length from server yet.
                    p.spinner();
                    p.add_space(6.0);
                    p.label(&dl.status);
                } else {
                    p.add(
                        egui::ProgressBar::new(dl.progress)
                            .desired_width(340.0)
                            .animate(dl.progress < 1.0),
                    );
                    p.add_space(6.0);
                    p.label(&dl.status);
                }

                p.add_space(8.0);
            });
        });
}
