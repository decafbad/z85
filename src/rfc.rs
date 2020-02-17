use std::error::Error;
use std::fmt::{self, Debug};

use super::encdec;
pub use encdec::ParserError;

/// Main type. Input data length must be
/// multiple of four.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Z85 {
    payload: Vec<u8>,
}

impl fmt::Display for Z85 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
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

/// Converts byte slice to Z85 ancoded vector
/// if its length is multiple of four
pub fn encode(input: &[u8]) -> Result<Vec<u8>, EncoderError> {
    let len = input.len();
    if len % 4 != 0 {
        return Err(EncoderError(len));
    }
    Ok(encdec::encode_z85_unchecked(input))
}

/// Converts z85 data back to original vector
/// if it's valid.
pub fn decode(input: &[u8]) -> Result<Vec<u8>, ParserError> {
    // TODO: too many cpu cycles here
    encdec::validate_z85(input).map(|_| encdec::decode_z85_unchecked(input))
}

impl Z85 {
    /// Creates Z85 from any byte slice with length
    /// multiple of four.
    pub fn encode(input: &[u8]) -> Result<Z85, EncoderError> {
        encode(input).map(|payload| Z85 { payload })
    }

    /// Converts data back to original byte vector.
    pub fn decode(&self) -> Vec<u8> {
        encdec::decode_z85_unchecked(self.payload.as_slice())
    }

    /// Takes in and owns Z85 data if it's valid.
    pub fn wrap_bytes(input: Vec<u8>) -> Result<Z85, ParserError> {
        encdec::validate_z85(&input).map(|_| Z85 { payload: input })
    }

    /// Owns any byte vector as Z85 and assumes it's valid.
    /// # Safety
    /// This can lead to crashes with wrong error messages.
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
