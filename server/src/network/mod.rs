// QUIC network layer. Accepts connections, handles login, routes packets.

mod connection;

use std::sync::Arc;
use quinn::{Endpoint, ServerConfig as QuinnServerConfig};
use tracing::{info, warn};

use crate::config::ServerConfig;
use crate::game::GameState;

/// Generate self-signed TLS cert for QUIC. In production, use real certs.
fn generate_self_signed_cert() -> Result<(rustls::pki_types::CertificateDer<'static>, rustls::pki_types::PrivateKeyDer<'static>), Box<dyn std::error::Error>> {
    let cert = rcgen::generate_simple_self_signed(vec!["skycraft".to_string()])?;
    let cert_der = rustls::pki_types::CertificateDer::from(cert.cert);
    let key_der = rustls::pki_types::PrivateKeyDer::try_from(cert.key_pair.serialize_der())
        .map_err(|e| format!("key conversion error: {}", e))?;
    Ok((cert_der, key_der))
}

/// Start the QUIC server and listen for connections.
pub async fn start_server(
    config: Arc<ServerConfig>,
    game_state: Arc<GameState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (cert, key) = generate_self_signed_cert()?;

    let mut tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?;
    tls_config.alpn_protocols = vec![b"skycraft".to_vec()];

    let quinn_config = QuinnServerConfig::with_crypto(Arc::new(
        quinn::crypto::rustls::QuicServerConfig::try_from(tls_config)?,
    ));

    let addr = format!("0.0.0.0:{}", config.port).parse()?;
    let endpoint = Endpoint::server(quinn_config, addr)?;

    info!("Server listening on 0.0.0.0:{}", config.port);

    // Accept connections loop
    while let Some(incoming) = endpoint.accept().await {
        let game_state = game_state.clone();
        let config = config.clone();
        tokio::spawn(async move {
            match incoming.await {
                Ok(conn) => {
                    let remote = conn.remote_address();
                    info!("New connection from {}", remote);
                    if let Err(e) = connection::handle_connection(conn, config, game_state).await {
                        warn!("Connection {} error: {}", remote, e);
                    }
                }
                Err(e) => {
                    warn!("Connection failed: {}", e);
                }
            }
        });
    }

    Ok(())
}
