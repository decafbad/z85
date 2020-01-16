use proptest::prelude::*;
use z85::padded::*;

const BS: [u8; 8] = [0x86, 0x4F, 0xD2, 0x6F, 0xB5, 0x59, 0xF7, 0x5B];
const LS: [u8; 10] = *b"HelloWorld";

#[test]
fn encode_simple() {
    let z85p = Z85p::encode(&BS);
    assert_eq!(&LS, z85p.as_bytes());
}

#[test]
fn decode_simple() {
    let z85p = unsafe { Z85p::wrap_bytes_unchecked(LS.to_vec()) };
    assert_eq!(z85p.decode(), BS)
}

proptest! {
    #[test]
    fn prop(input: Vec<u8>) {
        let z85p=Z85p::encode(&input);
        let newbs=z85p.decode();
        prop_assert_eq!(input,newbs);
    }
}
