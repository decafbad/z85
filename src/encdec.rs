use super::internal::{CVResult, Chunk};
use std::error::Error;
use std::fmt::{self, Debug};

pub fn encode_z85_unchecked(input: &[u8]) -> Vec<u8> {
    // one chunk more for possible tail
    let z85_length = input.len() / 4 * 5 + 5;
    let mut out = Vec::with_capacity(z85_length);
    for chunk in input.chunks(4) {
        let z85_chunk = Chunk::from_binary(chunk).to_z85();
        out.extend_from_slice(&z85_chunk);
    }
    out
}

pub fn encode_z85_padded(input: &[u8]) -> Vec<u8> {
    let length = input.len();
    let tail_size = length % 4;
    let has_tail = tail_size != 0;
    let chunked_size = length - tail_size;
    let mut out = encode_z85_unchecked(&input[..chunked_size]);
    if has_tail {
        let tail = &input[chunked_size..];
        let z85_tail = Chunk::from_bintail(tail).to_z85_tail();
        out.extend_from_slice(&z85_tail);
    }
    out
}

pub fn validate_z85(input: &[u8]) -> Result<(), ParserError> {
    use CVResult::*;
    use ParserError::*;
    let length = input.len();
    if length % 5 != 0 {
        return Err(InvalidInputSize(length));
    }
    for (cpos, chunk) in input.chunks(5).enumerate() {
        match Chunk::from_z85_checked(chunk) {
            Err(WrongChunk) => return Err(InvalidChunk(cpos * 5)),
            Err(WrongByte(pos, l)) => return Err(InvalidByte(cpos * 5 + pos, l)),
            _ => (),
        }
    }
    Ok(())
}

pub fn validate_z85_padded(input: &[u8]) -> Result<(), ParserError> {
    use CVResult::*;
    use ParserError::*;
    let length = input.len();
    let has_tail = length >= 5 && input[length - 5] == b'#';
    let chunked_size = if has_tail { length - 5 } else { length };

    if let err @ Err(_) = validate_z85(&input[..chunked_size]) {
        return err;
    }

    if has_tail {
        match Chunk::from_z85_tail_checked(&input[chunked_size..]) {
            Err(WrongChunk) => return Err(InvalidChunk(length - 5)),
            Err(WrongByte(pos, l)) => return Err(InvalidByte(length - 5 + pos, l)),
            _ => (),
        }
    }
    Ok(())
}

pub fn decode_z85(input: &[u8]) -> Result<Vec<u8>, ParserError> {
    use CVResult::*;
    use ParserError::*;
    let length = input.len();
    if length % 5 != 0 {
        return Err(InvalidInputSize(length));
    }
    // one chunk more for possible tail
    let z85_length = input.len() / 4 * 5 + 5;
    let mut out = Vec::with_capacity(z85_length);
    for (cpos, chunk) in input.chunks(5).enumerate() {
        match Chunk::from_z85_checked(chunk) {
            Err(WrongChunk) => return Err(InvalidChunk(cpos * 5)),
            Err(WrongByte(pos, l)) => return Err(InvalidByte(cpos * 5 + pos, l)),
            Ok(chunk) => out.extend_from_slice(&chunk.to_binary()),
        }
    }
    Ok(out)
}

pub fn decode_z85_padded(input: &[u8]) -> Result<Vec<u8>, ParserError> {
    use CVResult::*;
    use ParserError::*;
    let length = input.len();
    let has_tail = length >= 5 && input[length - 5] == b'#';
    let chunked_size = if has_tail { length - 5 } else { length };

    let mut out: Vec<u8>;
    match decode_z85(&input[..chunked_size]) {
        Err(e) => return Err(e),
        Ok(v) => out = v,
    }
    if has_tail {
        match Chunk::from_z85_tail_checked(&input[chunked_size..]) {
            Err(WrongChunk) => return Err(InvalidChunk(length - 5)),
            Err(WrongByte(pos, l)) => return Err(InvalidByte(length - 5 + pos, l)),
            Ok(c) => out.extend_from_slice(&c.to_bintail()),
        }
    }
    Ok(out)
}

pub fn decode_z85_unchecked(input: &[u8]) -> Vec<u8> {
    let length = input.len();
    // one chunk more for possible tail
    let mut out = Vec::with_capacity(length / 5 * 4 + 5);
    for chunk in input.chunks(5) {
        let binchunk = Chunk::from_z85(chunk).to_binary();
        out.extend_from_slice(&binchunk);
    }
    out
}

pub fn decode_z85_padded_unchecked(input: &[u8]) -> Vec<u8> {
    let length = input.len();
    let has_tail = length != 0 && input[length - 5] == b'#';
    let chunked_size = if has_tail { length - 5 } else { length };
    let mut out = decode_z85_unchecked(&input[..chunked_size]);
    if has_tail {
        let last_chunk = &input[chunked_size..];
        let bintail = Chunk::from_z85_tail(last_chunk).to_bintail();
        out.extend_from_slice(&bintail);
    }
    out
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn encode_z85_prop(input: Vec<u8>) {
            let mut bs=input;
            let proper_length=bs.len()/4*4;
            bs.truncate(proper_length);
            let ls=encode_z85_unchecked(&bs);
            if validate_z85(&ls).is_err() {
                panic!("validate_z85 shouldn't have returned error here");
            }
            let new_bs=decode_z85_unchecked(&ls);
            prop_assert_eq!(bs,new_bs);
        }

        #[test]
        fn encode_z85p_prop(input: Vec<u8>) {
            let ls=encode_z85_padded(&input);
            if validate_z85_padded(&ls).is_err() {
                panic!("validate_z85_padded shouldn't have returned error here");
            }
            let new_bs=decode_z85_padded_unchecked(&ls);
            prop_assert_eq!(input,new_bs);
        }

    }
}
