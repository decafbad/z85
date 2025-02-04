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

pub use super::DecodeError;
use std::convert::TryInto;

#[derive(Debug, Copy, Clone)]
pub struct BinTail([u8; 4]);

impl BinTail {
    fn new(mut binchunk: [u8; 4], diff: usize) -> Self {
        binchunk[0] = diff as u8;
        BinTail(binchunk)
    }

    pub fn append_to_vec(self, target: &mut Vec<u8>) {
        let binchunk = self.0;
        let diff = usize::from(binchunk[0]);
        let slice = &binchunk[diff..];
        target.extend_from_slice(slice);
    }
}

pub fn encode_chunk(input: &[u8]) -> [u8; 5] {
    let mut out = [0_u8; 5];
    let mut full_num: u32 = u32::from_be_bytes(input.try_into().unwrap());
    for letter in out.iter_mut().rev() {
        let index = (full_num % 85) as usize;
        *letter = LETTERS[index];
        full_num /= 85;
    }
    out
}

pub fn encode_tail(input: &[u8]) -> [u8; 5] {
    let mut input_padded = [0_u8; 4];
    let diff = 4 - input.len();
    input_padded[diff..].copy_from_slice(input);
    let mut out = encode_chunk(&input_padded);
    for l in out.iter_mut().take(diff) {
        *l = b'#';
    }
    out
}

pub fn decode_chunk(input: &[u8]) -> Result<[u8; 4], DecodeError> {
    const U32_MAX: u64 = 0x_FF_FF_FF_FF;
    let mut full_num = 0_u64;
    for (index, &letter) in input.iter().enumerate() {
        if letter <= 0x20 || 0x80 <= letter {
            return Err(DecodeError::InvalidByte(index, letter));
        }
        let octets_index = usize::from(letter - 32);
        let byte = OCTETS[octets_index];
        if byte == 0xFF {
            return Err(DecodeError::InvalidByte(index, letter));
        }
        full_num *= 85;
        full_num += u64::from(byte);
    }
    if full_num > U32_MAX {
        return Err(DecodeError::InvalidChunk(0));
    }
    let out = (full_num as u32).to_be_bytes();
    Ok(out)
}

pub fn decode_tail(input: &[u8]) -> Result<BinTail, DecodeError> {
    let diff = input.iter().take_while(|&&l| l == b'#').count();
    if diff > 3 {
        return Err(DecodeError::InvalidTail);
    }
    let binchunk = decode_chunk(&input[diff..])?;
    let max_full_num = 256_u32.pow(4 - diff as u32) - 1;
    if u32::from_be_bytes(binchunk) > max_full_num {
        return Err(DecodeError::InvalidTail);
    }
    Ok(BinTail::new(binchunk, diff))
}

#[cfg(test)]
mod tests {

    use super::*;
    use proptest::prelude::*;

    const BINCHUNK1: [u8; 4] = [0x86, 0x4F, 0xD2, 0x6F];
    const BINCHUNK2: [u8; 4] = [0xB5, 0x59, 0xF7, 0x5B];
    const Z85CHUNK1: [u8; 5] = *b"Hello";
    const Z85CHUNK2: [u8; 5] = *b"World";

    #[test]
    fn encode_chunk_simple() {
        let ans_z85_1 = encode_chunk(&BINCHUNK1);
        assert_eq!(ans_z85_1, Z85CHUNK1);
        let ans_z85_2 = encode_chunk(&BINCHUNK2);
        assert_eq!(ans_z85_2, Z85CHUNK2);
    }

    #[test]
    fn decode_chunk_simple() {
        let ans_bin_1 = decode_chunk(&Z85CHUNK1).unwrap();
        assert_eq!(ans_bin_1, BINCHUNK1);
        let ans_bin_2 = decode_chunk(&Z85CHUNK2).unwrap();
        assert_eq!(ans_bin_2, BINCHUNK2);
    }

    #[test]
    fn decode_chunk_full_hash() {
        let input = "#####".as_bytes();
        let answer = decode_chunk(input);
        assert_eq!(answer, Err(DecodeError::InvalidChunk(0)));
    }

    #[test]
    fn encode_tail_one_byte() {
        for b in 0..=255 {
            let input = &[b];
            let z85_tail = encode_tail(input);
            let bin_tail = decode_tail(&z85_tail).unwrap();
            let mut ans_vec = Vec::new();
            bin_tail.append_to_vec(&mut ans_vec);
            assert_eq!(ans_vec, input);
        }
    }

    #[test]
    fn all_letters_seven_bits_printable() {
        for &letter in LETTERS.iter() {
            assert!(0x20 < letter && letter < 0x80)
        }
    }

    proptest! {
        #[test]
        fn encode_chunk_prop(input: [u8; 4]) {
            let ans_z85 = encode_chunk(&input);
            let ans_bin = decode_chunk(&ans_z85).unwrap();
            assert_eq!(ans_bin, input);
        }

        #[test]
        fn encode_tail_two_bytes(input: [u8; 2]) {
            let z85_tail = encode_tail(&input);
            let bin_tail = decode_tail(&z85_tail).unwrap();
            let mut ans_vec = Vec::new();
            bin_tail.append_to_vec(&mut ans_vec);
            assert_eq!(ans_vec, input);
        }
        #[test]
        fn encode_tail_three_bytes(input: [u8; 3]) {
            let z85_tail = encode_tail(&input);
            let bin_tail = decode_tail(&z85_tail).unwrap();
            let mut ans_vec = Vec::new();
            bin_tail.append_to_vec(&mut ans_vec);
            assert_eq!(ans_vec, input);
        }
    }
}
