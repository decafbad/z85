//! Rust implementation of ZeroMQ's Z85 encoding mechanism

//! rfc module does just what ZeroMQ'rfc says.
//! padded module adds padding to eliminate
//! the need of 4-byte aligned input data
mod internal;

/// RFC incompatible code can encode everything
pub mod padded;
/// Compatible with ZeroMQ's RFC
pub mod rfc;
