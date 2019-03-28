#[macro_use]
extern crate quickcheck;

extern crate z85;

use z85::{decode, encode, DecodeError};


quickcheck! {
    fn encode_quick(input: Vec<u8>) -> bool {
        let mut input = input.clone();
        input.extend_from_slice(&input.clone());
        input.extend_from_slice(&input.clone());
        if let Ok(ls) = encode(&input) {
            if let Ok(exp_bs) = decode(&ls) {
                return exp_bs == input;
            }
        }
        false
    }
}