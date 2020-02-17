use smallvec::{smallvec, SmallVec};

type Tail = SmallVec<[u8; 5]>;

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

pub fn encode_chunk(binchunk: &[u8]) -> [u8; 5] {
    let mut full_num = 0_u32;
    for &b in binchunk {
        full_num <<= 8;
        full_num |= b as u32;
    }
    let mut z85_chunk = [0; 5];
    for l in z85_chunk.iter_mut().rev() {
        let index = (full_num % 85) as usize;
        *l = LETTERS[index];
        full_num /= 85;
    }
    z85_chunk
}

pub fn encode_tail(input: &[u8]) -> [u8; 5] {
    let diff = 4 - input.len();
    let mut bintail: Tail = smallvec![0;diff];
    bintail.extend_from_slice(input);
    let mut out = encode_chunk(&bintail);
    for l in out.iter_mut().take(diff) {
        *l = b'#';
    }
    out
}

// chunk validator result
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CVResult {
    Fine,
    WrongChunk,
    WrongByte(usize, u8),
}

// assumes input data is valid
pub fn decode_chunk(lschunk: &[u8]) -> [u8; 4] {
    let mut full_num: u32 = 0;
    for &l in lschunk {
        let index = (l as usize) - 32;
        let octet = OCTETS[index] as u32;
        full_num = full_num * 85 + octet;
    }
    u32::to_be_bytes(full_num)
}

// assumes input data is valid
pub fn decode_tail(input: &[u8]) -> Tail {
    let (z85_tail, diff) = get_diff(input);
    let out = decode_chunk(&z85_tail);
    let out = &out[diff..];
    SmallVec::from_slice(out)
}

pub fn validate_chunk(lschunk: &[u8]) -> CVResult {
    use CVResult::*;
    const U32_MAX: u64 = std::u32::MAX as u64;
    let mut full_num = 0_u64;
    for (i, &l) in lschunk.iter().enumerate() {
        if l < 0x20 || 0x7f < l {
            return WrongByte(i, l);
        }
        let index = (l - 32) as usize;
        let b = OCTETS[index];
        if b == 0xFF {
            return WrongByte(i, l);
        }
        full_num *= 85;
        full_num += b as u64;
    }
    if full_num > U32_MAX {
        return WrongChunk;
    }
    Fine
}

pub fn validate_tail(input: &[u8]) -> CVResult {
    use CVResult::*;
    let (z85_tail, diff) = get_diff(input);
    if diff < 1 || 3 < diff {
        return WrongChunk;
    }
    match validate_chunk(&z85_tail) {
        Fine => (),
        wrong => return wrong,
    }
    let mut full_num: u64 = 0;
    for &l in z85_tail.as_slice() {
        let index = (l as usize) - 32;
        let octet = OCTETS[index] as u64;
        full_num = full_num * 85 + octet;
    }
    let digit_count = 4 - (diff as u32);
    let max_full_num = 0x100_u64.pow(digit_count) - 1;
    if full_num > max_full_num {
        return WrongChunk;
    }
    Fine
}

// counts leading b'#' in tails and turns them to b'0'
fn get_diff(input: &[u8]) -> (Tail, usize) {
    let mut z85_tail: Tail = SmallVec::from_slice(input);
    let mut diff = 0;
    for l in z85_tail.iter_mut() {
        if *l != b'#' {
            break;
        }
        *l = b'0';
        diff += 1;
    }
    (z85_tail, diff)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const BS1: &[u8] = &[0x86, 0x4F, 0xD2, 0x6F];
    const BS2: &[u8] = &[0xB5, 0x59, 0xF7, 0x5B];
    const LS1: &[u8; 5] = b"Hello";
    const LS2: &[u8; 5] = b"World";

    #[test]
    fn encode_chunk_simple() {
        assert_eq!(encode_chunk(BS1), *LS1);
        assert_eq!(encode_chunk(BS2), *LS2);
    }

    #[test]
    fn decode_chunk_simple() {
        assert_eq!(decode_chunk(LS1), BS1);
        assert_eq!(decode_chunk(LS2), BS2);
    }

    #[test]
    fn all_raw_data_made_of_seven_bit_bytes() {
        for &b in LETTERS.iter() {
            assert!(b < 0x80)
        }
    }

    #[test]
    fn encode_tail_one_byte() {
        for b in 0..=255 {
            let bintail = vec![b];
            let z85_tail = encode_tail(&bintail);
            assert_eq!(validate_tail(&z85_tail), CVResult::Fine);
            let newbt = decode_tail(&z85_tail);
            assert_eq!(bintail, newbt.to_vec());
        }
    }

    proptest! {
        #[test]
        fn encode_chunk_prop(bs: [u8; 4]) {
            let ls = encode_chunk(&bs);
            let new_bs = decode_chunk(&ls);
            prop_assert_eq!(new_bs ,bs);
        }

        #[test]
        fn encode_chunk_is_unicode_prop(bs: [u8; 4]) {
            let ls = encode_chunk(&bs);
            let ls_str_res = std::str::from_utf8(&ls);
            prop_assert!(ls_str_res.is_ok());
        }

        #[test]
        fn encode_tail_two_bytes_prop(bs: [u8;2]) {
            let z85_tail = encode_tail(&bs);
            prop_assert_eq!(validate_tail(&z85_tail), CVResult::Fine);
            let new_bs = decode_tail(&z85_tail);
            prop_assert_eq!(&bs,new_bs.as_slice());
        }

        #[test]
        fn encode_tail_three_bytes_prop(bs: [u8;3]) {
            let z85_tail = encode_tail(&bs);
            prop_assert_eq!(validate_tail(&z85_tail), CVResult::Fine);
            let new_bs = decode_tail(&z85_tail);
            prop_assert_eq!(&bs,new_bs.as_slice());
        }

        #[test]
        fn validate_chunk_all_fine_prop(bs: [u8;4]) {
            let ls = encode_chunk(&bs);
            let fine = CVResult::Fine;
            let res=validate_chunk(&ls);
            prop_assert_eq!(fine,res)
        }
    }
}
