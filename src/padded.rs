use super::internal;
use std::fmt::{self, Debug};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Z85p {
    payload: Vec<u8>,
}

impl fmt::Display for Z85p {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Z85p {
    /// Creates Z85 from any byte slice
    pub fn encode(input: &[u8]) -> Z85p {
        let length = input.len();
        let tail_size = length % 4;
        let has_tail = tail_size != 0;
        let chunked_size = length - tail_size;
        let mut payload = Vec::with_capacity(length / 4 * 5 + 5);
        for chunk in input[..chunked_size].chunks(4) {
            let z85_chunk = internal::encode_chunk(chunk);
            payload.extend_from_slice(&z85_chunk);
        }
        if has_tail {
            let tail = &input[chunked_size..];
            let z85_tail = internal::encode_tail(tail);
            payload.extend_from_slice(&z85_tail);
        }
        Z85p { payload }
    }

    /// Converts data back to original byte vector.
    pub fn decode(&self) -> Vec<u8> {
        let input = &self.payload;
        let length = input.len();
        if length == 0 {
            return vec![];
        }
        let has_tail = input[length - 5] == b'#';
        let mut chunked_size = length;
        if has_tail {
            chunked_size -= 5;
        }
        let mut out = Vec::with_capacity(length / 5 * 4);
        for chunk in input[..chunked_size].chunks(5) {
            let binchunk = internal::decode_chunk(chunk);
            out.extend_from_slice(&binchunk);
        }
        if has_tail {
            let last_chunk = &input[chunked_size..];
            let bintail = internal::decode_tail(last_chunk);
            out.extend_from_slice(&bintail);
        }
        out
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
