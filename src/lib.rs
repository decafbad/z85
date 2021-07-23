//! Rust implementation of ZeroMQ's Z85 encoding mechanism.

use std::error::Error;
use std::fmt::{self, Display};

mod internal;
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DecodeError {
    InvalidByte(usize, u8),
    InvalidChunk(usize),
    InvalidLength(usize),
    InvalidTail,
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodeError::InvalidLength(length) => {
                write!(f, "Z85 data length ({}) is not multiple of five", length)
            }
            DecodeError::InvalidByte(position, byte) => write!(
                f,
                "Z85 data has an invalid byte (0x{:02X}) at ({}) ",
                byte, position
            ),
            DecodeError::InvalidChunk(position) => write!(
                f,
                "Z85 data has an invalid 5-bytes chunk at ({}) ",
                position
            ),
            DecodeError::InvalidTail => write!(f, "Z85 data has an invalid padding chunk"),
        }
    }
}

impl Error for DecodeError {}

/// Errors that can occur while decoding.
impl DecodeError {
    fn add_offset(&self, chunk_count: usize) -> Self {
        use DecodeError::*;
        let offset = chunk_count * 5;
        match self {
            InvalidByte(index, byte) => InvalidByte(index + offset, *byte),
            InvalidChunk(index) => InvalidChunk(index + offset),
            _ => *self,
        }
    }
}

/// Encode arbitrary octets as z85. Returns a String.
pub fn encode<T: AsRef<[u8]>>(input: T) -> String {
    let input = input.as_ref();
    let length = input.len();
    if length == 0 {
        return String::with_capacity(0);
    }
    let tail_size = length % 4;
    let chunked_size = length - tail_size;
    let mut out = Vec::with_capacity(length / 4 * 5 + 5);
    for chunk in input[..chunked_size].chunks(4) {
        let z85_chunk = internal::encode_chunk(chunk);
        out.extend_from_slice(&z85_chunk);
    }
    if tail_size > 0 {
        let bintail = &input[chunked_size..];
        let tail = internal::encode_tail(bintail);
        out.extend_from_slice(&tail);
    }

    // all bytes in internal::LETTERS have their most significant
    // bit set to zero. confirmed with unit test.
    unsafe { String::from_utf8_unchecked(out) }
}

/// Decode from string reference as octets. Returns a Result containing a Vec.
pub fn decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, DecodeError> {
    let input = input.as_ref();
    let length = input.len();
    if length == 0 {
        return Ok(Vec::with_capacity(0));
    }
    if length % 5 != 0 {
        return Err(DecodeError::InvalidLength(length));
    }
    let has_tail = input[length - 5] == b'#';
    let chunked_size = if has_tail { length - 5 } else { length };
    let mut out = Vec::with_capacity(length / 5 * 4);
    for (chunk_count, chunk) in input[..chunked_size].chunks(5).enumerate() {
        match internal::decode_chunk(chunk) {
            Err(decode_error) => return Err(decode_error.add_offset(chunk_count)),
            Ok(binchunk) => out.extend_from_slice(&binchunk),
        }
    }
    if has_tail {
        let last_chunk = &input[chunked_size..];
        match internal::decode_tail(last_chunk) {
            Err(decode_error) => return Err(decode_error.add_offset(length - 5)),
            Ok(bintail) => bintail.append_to_vec(&mut out),
        }
    }
    Ok(out)
}
