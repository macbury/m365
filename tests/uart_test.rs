use hex_literal::hex;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use m365::mi_crypto::{EncryptionKey, encrypt_uart, crc16, decrypt_uart};

#[test]
fn it_crc16() {
  let bytes = [0xa1, 0x21, 0xf3, 4, 5, 6, 7, 8, 9];
  let crc = crc16(&bytes);

  assert_eq!(crc, hex!("23fe"));
}

#[test]
fn it_encrypts_uart() {
  let encryption_key = EncryptionKey {
    key: hex!("5066d82368375a1f6a0a3eba1317b525"),
    iv: hex!("28cee53e")
  };

  let rand : [u8; 4] = hex!("897045e7");
  let cmd : [u8; 5] = hex!("032001100e");
  let ct = encrypt_uart(&encryption_key, &cmd, 0, Some(rand));

  let expected_result = hex!("55ab03000016b2eddb0b680532a988c4f2dbf9");

  assert_eq!(ct.len(), expected_result.len());
  assert_eq!(ct, expected_result)
}


#[test]
fn it_decrypt_uart() {
  tracing_subscriber::fmt()
    .with_max_level(Level::DEBUG)
    .with_span_events(FmtSpan::CLOSE)
    .init();
  let encryption_key = EncryptionKey {
    key: hex!("462f3fcc74200ca5f77ee2a581c42af0"),
    iv: hex!("f8901a05")
  };

  let encrypted : [u8; 32] = hex!("55ab1001009a70888f3a27d8378bb07f7d8ce4cce88ab54a50595ad6c019c7f2");
  let decrypted = decrypt_uart(&encryption_key, &encrypted).unwrap();
  let bytes = &decrypted[3..decrypted.len()-4];
  let text = String::from_utf8_lossy(bytes);

  assert_eq!("26354/00467353", text)
}
