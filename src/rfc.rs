//! Compatible with ZeroMQ's RFC.

use std::convert::TryInto;
use std::error::Error;
use std::fmt::{self, Debug};

static LETTERS: [u8; 85] = [
    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66,
    0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76,
    0x77, 0x78, 0x79, 0x7A, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C,
    0x4D, 0x4E, 0x4F, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x2E, 0x2D,
    0x3A, 0x2B, 0x3D, 0x5E, 0x21, 0x2F, 0x2A, 0x3F, 0x26, 0x3C, 0x3E, 0x28, 0x29, 0x5B, 0x5D, 0x7B,
    0x7D, 0x40, 0x25, 0x24, 0x23,
];

static OCTETS: [u8; 96] = [
    0xFF, 0x44, 0xFF, 0x54, 0x53, 0x52, 0x48, 0xFF, 0x4B, 0x4C, 0x46, 0x41, 0xFF, 0x3F, 0x3E, 0x45,
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x40, 0xFF, 0x49, 0x42, 0x4A, 0x47,
    0x51, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32,
    0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x4D, 0xFF, 0x4E, 0x43, 0xFF,
    0xFF, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
    0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x4F, 0xFF, 0x50, 0xFF, 0xFF,
];

static SORTED_LETTERS: [u8; 85] = [
    0x21, 0x23, 0x24, 0x25, 0x26, 0x28, 0x29, 0x2A, 0x2B, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33,
    0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3C, 0x3D, 0x3E, 0x3F, 0x40, 0x41, 0x42, 0x43, 0x44,
    0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50, 0x51, 0x52, 0x53, 0x54,
    0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5D, 0x5E, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67,
    0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77,
    0x78, 0x79, 0x7A, 0x7B, 0x7D,
];

/// Main type. Input data length must be
/// multiple of four.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Z85 {
    payload: Vec<u8>,
}

impl fmt::Display for Z85 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.payload.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParserError {
    InvalidInputSize(usize),
    InvalidByte(usize, u8),
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParserError::*;
        match self {
            InvalidInputSize(size) => write!(
                f,
                "Z85 parser input size ({}) is not multiple of five.",
                size
            ),
            InvalidByte(pos, b) => write!(
                f,
                "Z85 data has an invalid byte (0x{:02X}) at ({}) ",
                b, pos
            ),
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
            let chunk = chunk.try_into().unwrap();
            payload.extend_from_slice(&encode_chunk(chunk));
        }
        Ok(Z85 { payload })
    }

    /// Returns Z85 data as a slice.
    pub fn as_bytes(&self) -> &[u8] {
        self.payload.as_slice()
    }

    /// Converts data back to original byte vector.
    pub fn decode(&self) -> Vec<u8> {
        let pl = &self.payload;
        let mut out = Vec::with_capacity(pl.len() / 4 * 5);
        for chunk in pl.chunks(5) {
            let chunk = chunk.try_into().unwrap();
            out.extend_from_slice(&decode_chunk(chunk));
        }
        out
    }

    /// Takes in and owns Z85 data if it's valid.
    pub fn wrap_bytes(input: Vec<u8>) -> Result<Z85, ParserError> {
        use ParserError::*;
        let len = input.len();
        if len % 5 != 0 {
            return Err(InvalidInputSize(len));
        }
        for (i, l) in input.iter().enumerate() {
            if (&SORTED_LETTERS).binary_search(&l).is_err() {
                return Err(InvalidByte(i, *l));
            }
        }
        Ok(Z85 { payload: input })
    }

    /// Owns any byte vector as Z85 and assumes it's valid.
    /// This can lead to crashes.
    pub unsafe fn wrap_bytes_unchecked(input: Vec<u8>) -> Self {
        Z85 { payload: input }
    }
}

fn encode_chunk(binchunk: [u8; 4]) -> [u8; 5] {
    let mut full_num = u32::from_be_bytes(binchunk);
    let mut z85_chunk = [0; 5];
    for i in (0..5).rev() {
        z85_chunk[i] = LETTERS[(full_num % 85) as usize];
        full_num /= 85;
    }
    z85_chunk
}

fn decode_chunk(lschunk: [u8; 5]) -> [u8; 4] {
    let mut full_num: u32 = 0;
    for &l in &lschunk {
        full_num = full_num * 85 + OCTETS[(l as usize) - 32] as u32;
    }
    u32::to_be_bytes(full_num)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const BS1: [u8; 4] = [0x86, 0x4F, 0xD2, 0x6F];
    const BS2: [u8; 4] = [0xB5, 0x59, 0xF7, 0x5B];
    const LS1: [u8; 5] = *b"Hello";
    const LS2: [u8; 5] = *b"World";

    #[test]
    fn test_encode_chunk_simple() {
        assert_eq!(encode_chunk(BS1), LS1);
        assert_eq!(encode_chunk(BS2), LS2);
    }

    #[test]
    fn test_decode_chunk_simple() {
        assert_eq!(decode_chunk(LS1), BS1);
        assert_eq!(decode_chunk(LS2), BS2);
    }

    proptest! {
        #[test]
        fn test_encode_chunk_prop(bs: [u8; 4]) {
            let ls = encode_chunk(bs);
            let new_bs = decode_chunk(ls);
            prop_assert_eq!(new_bs ,bs);
        }
    }
}
