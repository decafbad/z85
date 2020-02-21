use proptest::prelude::*;
use z85::padded::*;

const BS: &[u8] = &[0x86, 0x4F, 0xD2, 0x6F, 0xB5, 0x59, 0xF7, 0x5B];
const LS: &[u8] = b"HelloWorld";

#[test]
fn encode_simple() {
    let z85p = Z85p::encode(BS);
    assert_eq!(LS, z85p.as_bytes());
}

#[test]
fn decode_simple() {
    let z85p = unsafe { Z85p::wrap_bytes_unchecked(LS.to_vec()) };
    assert_eq!(z85p.decode(), BS)
}

proptest! {
    #[test]
    fn z85p_prop(input: Vec<u8>) {
        let z85p=Z85p::encode(&input);
        let z85_vec=z85p.clone().as_bytes().to_vec();
        let wrapped=Z85p::wrap_bytes(z85_vec);
        if let Err(_) = wrapped {
            panic!("Z85p::wrap_bytes incorrectly returned error");
        }
        let newbs=z85p.decode();
        prop_assert_eq!(input,newbs);
    }

    #[test]
    fn vec_prop(input: Vec<u8>) {
        let z85_vec=encode(input.as_slice());
        let newbs=decode(&z85_vec).unwrap();
        prop_assert_eq!(input,newbs.as_slice());
    }
}
