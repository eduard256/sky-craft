// Network bridge: runs QUIC connection on background thread,
// communicates with main thread via channels.

use std::sync::mpsc;
use skycraft_protocol::packets::{ServerPacket, ClientPacket};
use skycraft_protocol::codec;
use tracing::{info, warn};

/// Handle to the background network thread.
pub struct NetBridge {
    /// Receive server packets (background -> main thread).
    rx: mpsc::Receiver<ServerPacket>,
    /// Send client packets (main thread -> background).
    tx: mpsc::Sender<ClientPacket>,
}

/// Messages from main thread to network thread.
enum NetCommand {
    Send(ClientPacket),
    Disconnect,
}

impl NetBridge {
    /// Connect to server on background thread, return bridge handle.
    /// Blocks until login completes (or fails).
    pub fn connect(
        addr: String,
        token: String,
    ) -> Result<(Self, skycraft_protocol::packets::S2CLoginSuccess), Box<dyn std::error::Error>> {
        // Channel for server packets (net thread -> main)
        let (server_tx, server_rx) = mpsc::channel::<ServerPacket>();
        // Channel for client packets (main -> net thread)
        let (client_tx, client_rx) = mpsc::channel::<ClientPacket>();
        // Channel for login result
        let (login_tx, login_rx) = mpsc::sync_channel::<Result<skycraft_protocol::packets::S2CLoginSuccess, String>>(1);

        let addr_clone = addr.clone();
        let token_clone = token.clone();

        // Spawn background thread with its own tokio runtime
        std::thread::Builder::new()
            .name("net_worker".into())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                rt.block_on(async move {
                    net_worker(addr_clone, token_clone, server_tx, client_rx, login_tx).await;
                });
            })?;

        // Wait for login result (blocking)
        let login_result = login_rx.recv()
            .map_err(|_| "Network thread died during login")?;

        let login_success = login_result.map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

        Ok((
            Self {
                rx: server_rx,
                tx: client_tx,
            },
            login_success,
        ))
    }

    /// Try to receive a server packet (non-blocking). Call each frame.
    pub fn try_recv(&self) -> Option<ServerPacket> {
        self.rx.try_recv().ok()
    }

    /// Drain all available server packets.
    pub fn drain_packets(&self) -> Vec<ServerPacket> {
        let mut packets = Vec::new();
        while let Ok(packet) = self.rx.try_recv() {
            packets.push(packet);
        }
        packets
    }

    /// Send a packet to the server.
    pub fn send(&self, packet: ClientPacket) {
        let _ = self.tx.send(packet);
    }
}

/// Background network worker. Runs on separate thread with own tokio runtime.
async fn net_worker(
    addr: String,
    token: String,
    server_tx: mpsc::Sender<ServerPacket>,
    client_rx: mpsc::Receiver<ClientPacket>,
    login_tx: mpsc::SyncSender<Result<skycraft_protocol::packets::S2CLoginSuccess, String>>,
) {
    // Connect
    let mut net_client = crate::network::NetworkClient::new();
    match net_client.connect(&addr, &token).await {
        Ok(login_success) => {
            info!("Net worker: logged in as {}", login_success.nickname);
            let _ = login_tx.send(Ok(login_success));
        }
        Err(e) => {
            warn!("Net worker: login failed: {}", e);
            let _ = login_tx.send(Err(e.to_string()));
            return;
        }
    }

    // Main loop: poll server + forward client packets
    loop {
        // Poll server for incoming packets
        if let Err(e) = net_client.poll().await {
            warn!("Net worker: connection lost: {}", e);
            break;
        }

        // Forward all received server packets to main thread
        while let Some(packet) = net_client.try_recv() {
            if server_tx.send(packet).is_err() {
                // Main thread dropped receiver, exit
                return;
            }
        }

        // Check for outbound client packets (non-blocking)
        while let Ok(packet) = client_rx.try_recv() {
            if let Err(e) = net_client.send(packet).await {
                warn!("Net worker: send error: {}", e);
                break;
            }
        }

        // Small yield to not spin CPU
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }
}
