//! Rust implementation of ZeroMQ's Z85 encoding mechanism.

//! `rfc` module does just what ZeroMQ's RFC says.
//! `padded` module adds padding to eliminate
//! the need of 4-byte aligned input data.
//! `padded` module hasn't been tested enough yet
//!  but there won't be any breaking changes.
mod encdec;
mod internal;

/// [Experimental] RFC incompatible code
/// that can encode everything
pub mod padded;

/// Compatible with ZeroMQ's RFC.
pub mod rfc;
