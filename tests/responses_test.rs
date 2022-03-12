use hex_literal::hex;

#[test]
fn it_guess_what_distance_is_left() {
  let bytes = hex!("230125320a6af89411");
  let distance_bytes : [u8; 2] = bytes[3..5].try_into().unwrap();
  let distance_left_meters = u16::from_le_bytes(distance_bytes);
  assert_eq!(distance_left_meters, 2610);
}
