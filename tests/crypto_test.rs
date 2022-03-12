use rand_core::OsRng;
use p256::{EncodedPoint, ecdh::EphemeralSecret};
use m365::mi_crypto;

#[test]
fn it_calculates_did() {
  let scooter_secret = EphemeralSecret::random(&mut OsRng);
  let remote_secret = EphemeralSecret::random(&mut OsRng);
  let remote_public_key = EncodedPoint::from(remote_secret.public_key());

  let remote_info = [
    0x01, 0x00, 0x00, 0x00, 0x00, 0x62, 0x6c, 0x74, 0x2e, 0x33, 0x2e, 0x31, 0x36, 0x33, 0x39, 0x34, 0x74, 0x33, 0x67, 0x34, 0x6c, 0x63, 0x30, 0x30
  ];

  let (did_ct, token) = mi_crypto::calc_did(&scooter_secret, remote_public_key.as_bytes(), &remote_info);

  assert_eq!(24, did_ct.len());
  assert_eq!(12, token.len());
}
