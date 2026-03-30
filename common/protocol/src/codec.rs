// Packet encoding/decoding over byte streams.
//
// Wire format (per packet):
//   [u32 payload_length] [payload bytes]
//
// Payload is bincode-serialized ClientPacket or ServerPacket enum.
// The enum variant tag is included in bincode serialization automatically.
//
// No compression in v0.0.1. Max packet size enforced on decode.

use crate::packets::{ClientPacket, ServerPacket};
use crate::MAX_PACKET_SIZE;

/// Errors during packet encode/decode.
#[derive(Debug)]
pub enum CodecError {
    /// Packet exceeds MAX_PACKET_SIZE.
    PacketTooLarge(u32),
    /// bincode serialization failed.
    SerializeError(String),
    /// bincode deserialization failed.
    DeserializeError(String),
    /// Not enough bytes to read the length header.
    InsufficientData,
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodecError::PacketTooLarge(size) => {
                write!(f, "packet too large: {} bytes (max {})", size, MAX_PACKET_SIZE)
            }
            CodecError::SerializeError(e) => write!(f, "serialize error: {}", e),
            CodecError::DeserializeError(e) => write!(f, "deserialize error: {}", e),
            CodecError::InsufficientData => write!(f, "insufficient data"),
        }
    }
}

impl std::error::Error for CodecError {}

// ─── Encode ─────────────────────────────────────────────────────────────────

/// Encode a ClientPacket to bytes (length-prefixed).
pub fn encode_client_packet(packet: &ClientPacket) -> Result<Vec<u8>, CodecError> {
    let payload = bincode::serialize(packet)
        .map_err(|e| CodecError::SerializeError(e.to_string()))?;

    let len = payload.len() as u32;
    if len > MAX_PACKET_SIZE {
        return Err(CodecError::PacketTooLarge(len));
    }

    let mut buf = Vec::with_capacity(4 + payload.len());
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(&payload);
    Ok(buf)
}

/// Encode a ServerPacket to bytes (length-prefixed).
pub fn encode_server_packet(packet: &ServerPacket) -> Result<Vec<u8>, CodecError> {
    let payload = bincode::serialize(packet)
        .map_err(|e| CodecError::SerializeError(e.to_string()))?;

    let len = payload.len() as u32;
    if len > MAX_PACKET_SIZE {
        return Err(CodecError::PacketTooLarge(len));
    }

    let mut buf = Vec::with_capacity(4 + payload.len());
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(&payload);
    Ok(buf)
}

// ─── Decode ─────────────────────────────────────────────────────────────────

/// Try to decode a ClientPacket from a byte buffer.
/// Returns the packet and the number of bytes consumed.
/// Returns None if not enough data is available yet (partial read).
pub fn decode_client_packet(buf: &[u8]) -> Result<Option<(ClientPacket, usize)>, CodecError> {
    if buf.len() < 4 {
        return Ok(None);
    }

    let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    if len > MAX_PACKET_SIZE {
        return Err(CodecError::PacketTooLarge(len));
    }

    let total = 4 + len as usize;
    if buf.len() < total {
        return Ok(None);
    }

    let packet: ClientPacket = bincode::deserialize(&buf[4..total])
        .map_err(|e| CodecError::DeserializeError(e.to_string()))?;

    Ok(Some((packet, total)))
}

/// Try to decode a ServerPacket from a byte buffer.
/// Returns the packet and the number of bytes consumed.
/// Returns None if not enough data is available yet (partial read).
pub fn decode_server_packet(buf: &[u8]) -> Result<Option<(ServerPacket, usize)>, CodecError> {
    if buf.len() < 4 {
        return Ok(None);
    }

    let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    if len > MAX_PACKET_SIZE {
        return Err(CodecError::PacketTooLarge(len));
    }

    let total = 4 + len as usize;
    if buf.len() < total {
        return Ok(None);
    }

    let packet: ServerPacket = bincode::deserialize(&buf[4..total])
        .map_err(|e| CodecError::DeserializeError(e.to_string()))?;

    Ok(Some((packet, total)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packets::*;
    use crate::types::*;

    #[test]
    fn roundtrip_client_packet() {
        let packet = ClientPacket::ChatMessage(C2SChatMessage {
            message: "Hello Sky Craft!".to_string(),
        });
        let bytes = encode_client_packet(&packet).unwrap();
        let (decoded, consumed) = decode_client_packet(&bytes).unwrap().unwrap();
        assert_eq!(consumed, bytes.len());
        match decoded {
            ClientPacket::ChatMessage(msg) => assert_eq!(msg.message, "Hello Sky Craft!"),
            _ => panic!("wrong packet type"),
        }
    }

    #[test]
    fn roundtrip_server_packet() {
        let packet = ServerPacket::UpdateHealth(S2CUpdateHealth {
            health: 20.0,
            food: 18,
            saturation: 5.0,
        });
        let bytes = encode_server_packet(&packet).unwrap();
        let (decoded, consumed) = decode_server_packet(&bytes).unwrap().unwrap();
        assert_eq!(consumed, bytes.len());
        match decoded {
            ServerPacket::UpdateHealth(h) => {
                assert_eq!(h.health, 20.0);
                assert_eq!(h.food, 18);
            }
            _ => panic!("wrong packet type"),
        }
    }

    #[test]
    fn partial_data_returns_none() {
        let packet = ClientPacket::KeepAliveResponse(C2SKeepAliveResponse { id: 42 });
        let bytes = encode_client_packet(&packet).unwrap();
        // Only send half the bytes
        let result = decode_client_packet(&bytes[..bytes.len() / 2]).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn chunk_section_basics() {
        let section = ChunkSection::empty();
        assert!(section.is_empty());
        assert_eq!(section.get_block(0, 0, 0), 0); // air
        assert_eq!(section.get_block(15, 15, 15), 0); // air
    }

    #[test]
    fn block_pos_chunk_conversion() {
        let pos = BlockPos::new(17, 65, -3);
        let chunk = pos.to_chunk_pos();
        assert_eq!(chunk.x, 1);  // 17 / 16 = 1
        assert_eq!(chunk.y, 4);  // 65 / 16 = 4
        assert_eq!(chunk.z, -1); // -3 / 16 = -1 (Euclidean division)

        let (lx, ly, lz) = pos.chunk_local();
        assert_eq!(lx, 1);  // 17 % 16 = 1
        assert_eq!(ly, 1);  // 65 % 16 = 1
        assert_eq!(lz, 13); // -3 % 16 = 13 (Euclidean remainder)
    }

    #[test]
    fn entity_pos_distance() {
        let a = EntityPos::new(0.0, 0.0, 0.0);
        let b = EntityPos::new(3.0, 4.0, 0.0);
        assert!((a.distance_to(&b) - 5.0).abs() < 0.001);
    }

    #[test]
    fn ring_calculation() {
        let origin = EntityPos::new(0.0, 64.0, 0.0);
        assert_eq!((origin.horizontal_distance_to_origin() / 500.0) as u32, 0); // ring 0

        let ring3 = EntityPos::new(1500.0, 64.0, 0.0);
        assert_eq!((ring3.horizontal_distance_to_origin() / 500.0) as u32, 3); // ring 3
    }
}
