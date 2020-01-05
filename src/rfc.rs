//! Compatible with ZeroMQ's RFC.

use super::internal;
use std::error::Error;
use std::fmt::{self, Debug};

/// Main type. Input data length must be
/// multiple of four.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Z85 {
    payload: Vec<u8>,
}

impl fmt::Display for Z85 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncoderError(usize);

impl fmt::Display for EncoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Z85 encoder input size ({}) is not multiple of four.",
            self.0
        )
    }
}

impl Error for EncoderError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserError {
    InvalidInputSize(usize),
    InvalidByte(usize, u8),
    InvalidChunk(usize),
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParserError::*;
        match self {
            InvalidInputSize(size) => {
                write!(f, "Z85 input length ({}) is not multiple of five.", size)
            }
            InvalidByte(pos, b) => write!(
                f,
                "Z85 data has an invalid byte (0x{:02X}) at ({}) ",
                b, pos
            ),
            InvalidChunk(pos) => write!(f, "Z85 data has an invalid 5 bytes chunk at ({}) ", pos),
        }
    }
}

impl Error for ParserError {}

impl Z85 {
    /// Creates Z85 from any byte slice with length
    /// multiple of four.
    pub fn encode(input: &[u8]) -> Result<Z85, EncoderError> {
        let len = input.len();
        if len % 4 != 0 {
            return Err(EncoderError(len));
        }
        let z85_length = input.len() / 4 * 5;
        let mut payload = Vec::with_capacity(z85_length);
        for chunk in input.chunks(4) {
            let z85_chunk = internal::encode_chunk(chunk);
            payload.extend_from_slice(&z85_chunk);
        }
        Ok(Z85 { payload })
    }

    /// Converts data back to original byte vector.
    pub fn decode(&self) -> Vec<u8> {
        let pl = &self.payload;
        let mut out = Vec::with_capacity(pl.len() / 4 * 5);
        for chunk in pl.chunks(5) {
            let binchunk = internal::decode_chunk(chunk);
            out.extend_from_slice(&binchunk);
        }
        out
    }

    /// Takes in and owns Z85 data if it's valid.
    pub fn wrap_bytes(input: Vec<u8>) -> Result<Z85, ParserError> {
        use internal::CVResult::*;
        use ParserError::*;
        let len = input.len();
        if len % 5 != 0 {
            return Err(InvalidInputSize(len));
        }
        for (cpos, chunk) in input.chunks(5).enumerate() {
            match internal::validate_chunk(chunk) {
                WrongChunk => return Err(InvalidChunk(cpos * 5)),
                WrongByte(pos, l) => return Err(InvalidByte(cpos * 5 + pos, l)),
                Fine => (),
            }
        }
        Ok(Z85 { payload: input })
    }

    /// # Safety
    /// This can lead to crashes.
    /// Owns any byte vector as Z85 and assumes it's valid.
    pub unsafe fn wrap_bytes_unchecked(input: Vec<u8>) -> Self {
        Z85 { payload: input }
    }

    /// Returns Z85 data as a str.
    pub fn as_str(&self) -> &str {
        // SAFETY: We know (through checking or constructing ourselves) that the payload
        //         only contains valid Z85 encoding characters.
        unsafe { std::str::from_utf8_unchecked(&self.payload) }
    }

    /// Returns Z85 data as a slice.
    pub fn as_bytes(&self) -> &[u8] {
        self.payload.as_slice()
    }
}
