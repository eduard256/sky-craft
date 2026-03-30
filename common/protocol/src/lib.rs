// Sky Craft Protocol v0.0.1
// Shared types and packet definitions for client-server communication.
// Transport: QUIC (quinn crate). Serialization: bincode + serde.
// Port: 35565. No compression in v0.0.1.

pub mod types;
pub mod packets;
pub mod codec;

/// Protocol version. Both client and server must match this exactly.
pub const PROTOCOL_VERSION: u32 = 1;

/// Default server port.
pub const DEFAULT_PORT: u16 = 35565;

/// Max packet size in bytes (1 MB). Packets larger than this are rejected.
pub const MAX_PACKET_SIZE: u32 = 1_048_576;

/// Server tick rate (ticks per second).
pub const TICKS_PER_SECOND: u32 = 20;

/// Milliseconds per tick.
pub const MS_PER_TICK: u32 = 1000 / TICKS_PER_SECOND;

/// Default view distance in chunks.
pub const DEFAULT_VIEW_DISTANCE: u8 = 8;

/// Max view distance in chunks.
pub const MAX_VIEW_DISTANCE: u8 = 32;

/// Keep-alive interval in seconds.
pub const KEEP_ALIVE_INTERVAL_SECS: u32 = 15;

/// Keep-alive timeout in seconds. Disconnect if no response within this.
pub const KEEP_ALIVE_TIMEOUT_SECS: u32 = 30;

/// Max chat message length in characters.
pub const MAX_CHAT_LENGTH: usize = 256;

/// Max nickname length.
pub const MAX_NICKNAME_LENGTH: usize = 16;

/// Min nickname length.
pub const MIN_NICKNAME_LENGTH: usize = 3;
