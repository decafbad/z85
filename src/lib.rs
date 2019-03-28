extern crate byteorder;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

use byteorder::{BigEndian, ByteOrder};

static LETTERS: [u8; 85] =
    *b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

fn encode_chunk(input: &[u8]) -> [u8; 5] {
    let mut num = BigEndian::read_u32(input) as usize;
    let mut ls = [0_u8; 5];
    for i in (0..5).rev() {
        ls[i] = LETTERS[num % 85];
        num /= 85;
    }
    ls
}

pub struct InvalidInputSize;

pub fn encode<T: ?Sized + AsRef<[u8]>>(input: &T) -> Result<String, InvalidInputSize> {
    let input = input.as_ref();
    let len = input.len();
    if len % 4 != 0 {
        return Err(InvalidInputSize);
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

/// Errors that can occur while decoding.
pub enum DecodeError {
    /// The length of the input is invalid.
    InvalidLength,
    /// An invalid byte was found in the input. The offset and offending byte are provided.
    InvalidByte(usize, u8),
}

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
    use super::FromDecodeChunk::*;
    use super::*;
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
}
