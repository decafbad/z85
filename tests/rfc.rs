use proptest::prelude::*;
use z85::rfc::*;

const BS: &[u8] = &[0x86, 0x4F, 0xD2, 0x6F, 0xB5, 0x59, 0xF7, 0x5B];
const LS: &[u8] = b"HelloWorld";

#[test]
fn encode_simple() {
    let z85 = Z85::encode(BS).unwrap();
    assert_eq!(LS, z85.as_bytes());
}

#[test]
fn decode_simple() {
    let z85 = unsafe { Z85::wrap_bytes_unchecked(LS.to_vec()) };
    assert_eq!(z85.decode(), BS)
}

proptest! {
    #[test]
    fn z85_prop(input: Vec<u8>) {
        let mut pbs=input;
        pbs.truncate(pbs.len()/4*4);
        let z85=Z85::encode(pbs.as_slice()).unwrap();
        let z85_data=z85.as_bytes().to_vec();
        if let Err(_)=Z85::wrap_bytes(z85_data) {
            panic!("Z85::wrap_bytes incorrectly returned error");
        }
        let newbs=z85.decode();
        prop_assert_eq!(pbs,newbs);
    }
    #[test]
    fn vec_prop(input: Vec<u8>) {
        let mut pbs=input;
        pbs.truncate(pbs.len()/4*4);
        let z85_vec=encode(pbs.as_slice()).unwrap();
        let newbs=decode(&z85_vec).unwrap();
        prop_assert_eq!(pbs,newbs.as_slice());
    }
}
