extern crate byteorder;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

use byteorder::{BigEndian, ByteOrder};
use std::{error, fmt, str};

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

fn encode_chunk(input: &[u8]) -> [u8; 5] {
    let mut num = BigEndian::read_u32(input) as usize;
    let mut ls = [0_u8; 5];
    for i in (0..5).rev() {
        ls[i] = LETTERS[num % 85];
        num /= 85;
    }
    ls
}

/// Returned when encode functions gets a Vec with
/// siz isn't multiple of four.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidInputSizeError;

impl fmt::Display for InvalidInputSizeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Input size must be multiple of four.")
    }
}

impl error::Error for InvalidInputSizeError {
    fn description(&self) -> &str {
        "invalid input size"
    }
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

/// Encode arbitrary octets as base64. Returns a Result with String.
/// Input size must be multiple of four.
pub fn encode<T: ?Sized + AsRef<[u8]>>(input: &T) -> Result<String, InvalidInputSizeError> {
    let input = input.as_ref();
    let len = input.len();
    if len % 4 != 0 {
        return Err(InvalidInputSizeError);
    }
    let mut out = Vec::with_capacity(len / 4 * 5);
    for chunk in input.chunks(4) {
        out.extend_from_slice(&encode_chunk(chunk));
    }
    unsafe { Ok(String::from_utf8_unchecked(out)) }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum FromDecodeChunk {
    Fine([u8; 4]),
    Bollocks(usize),
}

fn decode_chunk(input: &[u8]) -> FromDecodeChunk {
    use FromDecodeChunk::*;
    let mut num: u32 = 0;
    for i in 0..5 {
        num *= 85;
        let l = input[i];
        if l < 0x20 || 0x7F < l {
            return Bollocks(i);
        }
        let b = OCTETS[l as usize - 32];
        if b == 0xFF {
            return Bollocks(i);
        }
        num += b as u32;
    }
    let mut out = [0_u8; 4];
    BigEndian::write_u32(&mut out, num);
    Fine(out)
}
/*
fn decode_chunk(input: &[u8]) -> FromDecodeChunk {
    use FromDecodeChunk::*;
    let mut num: u32 = 0;
    for i in 0..5 {
        num *= 85;
        let mut found = false;
        for j in 0..85 {
            if LETTERS[j] == input[i] {
                num += j as u32;
                found = true;
            }
        }
        if !found {
            return Bollocks(i);
        }
    }
    let mut out = [0_u8; 4];
    BigEndian::write_u32(&mut out, num);
    Fine(out)
}
*/

/// Errors that can occur while decoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// The length of the input is invalid.
    InvalidLength,
    /// An invalid byte was found in the input. The offset and offending byte are provided.
    InvalidByte(usize, u8),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DecodeError::*;
        match *self {
            InvalidLength => write!(f, "Encoded text length must be multiple of five."),
            InvalidByte(index, byte) => write!(f, "Invalid byte 0x{:2X}, offset {}.", byte, index),
        }
    }
}

impl error::Error for DecodeError {
    fn description(&self) -> &str {
        match *self {
            DecodeError::InvalidByte(_, _) => "invalid byte",
            DecodeError::InvalidLength => "invalid length",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

/// Decode from string reference as octets. Returns a Result containing a Vec.
pub fn decode<T: ?Sized + AsRef<[u8]>>(input: &T) -> Result<Vec<u8>, DecodeError> {
    use DecodeError::*;
    use FromDecodeChunk::*;
    let input = input.as_ref();
    let len = input.len();
    if len % 5 != 0 {
        return Err(InvalidLength);
    }
    let mut out = Vec::with_capacity(len / 5 * 4);
    for (i, chunk) in input.chunks(5).enumerate() {
        match decode_chunk(chunk) {
            Bollocks(pos) => return Err(InvalidByte(i * 5 + pos, chunk[pos])),
            Fine(out_chunk) => out.extend_from_slice(&out_chunk),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use FromDecodeChunk::*;
    const LS1: [u8; 5] = *b"Hello";
    const LS2: [u8; 5] = *b"World";
    const BS1: [u8; 4] = [0x86, 0x4F, 0xD2, 0x6F];
    const BS2: [u8; 4] = [0xB5, 0x59, 0xF7, 0x5B];

    #[test]
    fn ec_simple() {
        let exp_ls1 = encode_chunk(&BS1);
        let exp_ls2 = encode_chunk(&BS2);
        assert_eq!(exp_ls1, LS1);
        assert_eq!(exp_ls2, LS2);
    }

    #[test]
    fn dc_simple() {
        let exp_bs1 = decode_chunk(&LS1);
        let exp_bs2 = decode_chunk(&LS2);
        assert_eq!(Fine(BS1), exp_bs1);
        assert_eq!(Fine(BS2), exp_bs2);
    }

    quickcheck! {
        fn ec_quick(num: u32) -> bool {
            let mut bs = [0_u8; 4];
            BigEndian::write_u32(&mut bs, num);
            let exp_ls = encode_chunk(&bs);
            if let Fine(exp_bs)=decode_chunk(&exp_ls) {
                if exp_bs==bs {
                    return true;
                }
            }
            return false;
        }
    }

    #[test]
    fn seven_bit_letters() {
        for &l in LETTERS.iter() {
            assert!(l < 0x80)
        }
    }
}
