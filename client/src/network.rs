// Network client. Connects to server via QUIC, sends/receives packets.

use std::collections::VecDeque;
use quinn::{Connection, Endpoint, ClientConfig};
use skycraft_protocol::codec;
use skycraft_protocol::packets::*;
use skycraft_protocol::PROTOCOL_VERSION;
use tracing::{info, warn};

/// QUIC network client for connecting to a Sky Craft server.
pub struct NetworkClient {
    connection: Option<Connection>,
    send: Option<quinn::SendStream>,
    recv: Option<quinn::RecvStream>,
    inbound_queue: VecDeque<ServerPacket>,
    read_buf: Vec<u8>,
}

impl NetworkClient {
    pub fn new() -> Self {
        Self {
            connection: None,
            send: None,
            recv: None,
            inbound_queue: VecDeque::new(),
            read_buf: Vec::new(),
        }
    }

    /// Connect to a server and perform login.
    pub async fn connect(
        &mut self,
        addr: &str,
        auth_token: &str,
    ) -> Result<S2CLoginSuccess, Box<dyn std::error::Error>> {
        // Create QUIC endpoint with insecure TLS (self-signed cert accepted)
        let mut crypto = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(InsecureCertVerifier))
            .with_no_client_auth();
        crypto.alpn_protocols = vec![b"skycraft".to_vec()];

        let client_config = ClientConfig::new(Arc::new(
            quinn::crypto::rustls::QuicClientConfig::try_from(crypto)?,
        ));

        let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
        endpoint.set_default_client_config(client_config);

        info!("Connecting to {}...", addr);
        let connection = endpoint.connect(addr.parse()?, "skycraft")?.await?;
        info!("QUIC connection established");

        // Open bidirectional stream
        let (mut send, mut recv) = connection.open_bi().await?;

        // Send login packet
        let login = ClientPacket::Login(C2SLogin {
            protocol_version: PROTOCOL_VERSION,
            auth_token: auth_token.to_string(),
        });
        let bytes = codec::encode_client_packet(&login)?;
        send.write_all(&bytes).await?;

        // Read login response
        let mut buf = vec![0u8; 65536];
        let n = recv.read(&mut buf).await?
            .ok_or("Connection closed during login")?;

        let (response, _) = codec::decode_server_packet(&buf[..n])?
            .ok_or("Incomplete login response")?;

        match response {
            ServerPacket::LoginSuccess(success) => {
                info!("Logged in as {} ({})", success.nickname, success.player_uuid);
                self.connection = Some(connection);
                self.send = Some(send);
                self.recv = Some(recv);
                Ok(success)
            }
            ServerPacket::Disconnect(d) => {
                Err(format!("Login rejected: {}", d.reason).into())
            }
            _ => {
                Err("Unexpected response during login".into())
            }
        }
    }

    /// Send a packet to the server.
    pub async fn send(&mut self, packet: ClientPacket) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut send) = self.send {
            let bytes = codec::encode_client_packet(&packet)?;
            send.write_all(&bytes).await?;
        }
        Ok(())
    }

    /// Try to receive a packet from the inbound queue (non-blocking).
    pub fn try_recv(&mut self) -> Option<ServerPacket> {
        self.inbound_queue.pop_front()
    }

    /// Poll for incoming data and decode packets. Call this regularly.
    pub async fn poll(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut recv) = self.recv {
            let mut buf = vec![0u8; 65536];

            // Non-blocking read attempt
            tokio::select! {
                result = recv.read(&mut buf) => {
                    match result {
                        Ok(Some(n)) => {
                            self.read_buf.extend_from_slice(&buf[..n]);
                        }
                        Ok(None) => {
                            return Err("Server closed connection".into());
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(1)) => {
                    // No data ready
                }
            }

            // Decode all complete packets
            loop {
                match codec::decode_server_packet(&self.read_buf) {
                    Ok(Some((packet, consumed))) => {
                        self.read_buf.drain(..consumed);
                        self.inbound_queue.push_back(packet);
                    }
                    Ok(None) => break,
                    Err(e) => {
                        warn!("Packet decode error: {}", e);
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// Check if connected.
    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }
}

// ── TLS: accept self-signed certs (dev only) ────────────────────────────────

use std::sync::Arc;

#[derive(Debug)]
struct InsecureCertVerifier;

impl rustls::client::danger::ServerCertVerifier for InsecureCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}
