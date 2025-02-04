use proptest::prelude::*;
use z85::*;

const BINARR: [u8; 8] = [0x86, 0x4F, 0xD2, 0x6F, 0xB5, 0x59, 0xF7, 0x5B];
const Z85ARR: [u8; 10] = *b"HelloWorld";

#[test]
fn encode_simple() {
    let z85_dat = encode(&BINARR);
    assert_eq!(&Z85ARR, z85_dat.as_bytes());
}

#[test]
fn decode_simple() {
    let bin_dat = decode(&Z85ARR).unwrap();
    assert_eq!(&BINARR, bin_dat.as_slice());
}

#[test]
fn encode_tail_only() {
    const BINTAIL: [u8; 2] = [0x86, 0x4F];
    let z85_dat = encode(&BINTAIL);
    let out_tail = decode(&z85_dat).unwrap();
    assert_eq!(&BINTAIL, out_tail.as_slice());
}

#[test]
fn decode_too_big_tail() {
    let z85str = "HelloWorld#####";
    let decoded = decode(z85str);
    assert_eq!(decoded, Err(DecodeError::InvalidTail));
}

proptest! {
    #[test]
    fn encode_prop(input: Vec<u8>) {
        let z85_dat = encode(&input);
        let ans_bin_dat = decode(z85_dat).unwrap();
        assert_eq!(ans_bin_dat, input);
    }
}
