use proptest::prelude::*;
use z85::rfc::*;

const BS: [u8; 8] = [0x86, 0x4F, 0xD2, 0x6F, 0xB5, 0x59, 0xF7, 0x5B];
const LS: [u8; 10] = *b"HelloWorld";

#[test]
fn test_encode_simple() {
    let z85 = Z85::encode(&BS).unwrap();
    assert_eq!(&LS, z85.as_bytes());
}

#[test]
fn test_decode_simple() {
    let z85 = Z85::wrap_bytes((&LS).to_vec()).unwrap();
    assert_eq!(z85.decode(), BS)
}

proptest! {
    #[test]
    fn test_prop(input: Vec<u8>) {
        let mut pbs=input.clone();
        pbs.truncate(pbs.len()/4*4);
        let z85=Z85::encode(pbs.as_slice()).unwrap();
        let newbs=z85.decode();
        assert_eq!(pbs,newbs);
    }
}
