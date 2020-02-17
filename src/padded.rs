use super::encdec;
use std::fmt::{self, Debug};

/// Converts any byte slice to Z85p data
pub fn encode(input: &[u8]) -> Vec<u8> {
    encdec::encode_z85_padded(input)
}

/// Converts z85p data back to original vector
/// if it's valid.
pub fn decode(input: &[u8]) -> Result<Vec<u8>, ParserError> {
    // TODO: no need to iterate twice
    encdec::validate_z85_padded(input).map(|_| encdec::decode_z85_padded_unchecked(input))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Z85p {
    payload: Vec<u8>,
}

pub use encdec::ParserError;

impl fmt::Display for Z85p {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Z85p {
    /// Creates Z85p from any byte slice
    pub fn encode(input: &[u8]) -> Z85p {
        let payload = encode(input);
        Z85p { payload }
    }

    /// Converts data back to original byte vector.
    pub fn decode(&self) -> Vec<u8> {
        encdec::decode_z85_padded_unchecked(&self.payload)
    }

    /// Takes in and owns Z85p data if it's valid.
    pub fn wrap_bytes(input: Vec<u8>) -> Result<Z85p, ParserError> {
        encdec::validate_z85_padded(&input).map(|_| Z85p { payload: input })
    }

    /// Owns any byte vector as Z85p and assumes it's valid.
    /// # Safety
    /// This can lead to crashes with wrong error messages.
    pub unsafe fn wrap_bytes_unchecked(input: Vec<u8>) -> Self {
        Z85p { payload: input }
    }

    /// Returns Z85p data as a str.
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
