pub use super::payload::Payload;
use super::commands::ScooterCommand;
use crate::protocol::MiProtocol;
use crate::mi_crypto::{encrypt_uart, decrypt_uart, LoginKeychain};
use crate::consts::Registers;

use anyhow::Result;
use btleplug::platform::Peripheral;

pub struct MiSession {
  protocol: MiProtocol,
  keys: LoginKeychain,
}

impl MiSession {
  pub async fn new(device: &Peripheral, keys: &LoginKeychain) -> Result<Self> {
    let protocol = MiProtocol::new(device).await?;
    let keys = keys.clone();

    Ok(Self { protocol, keys })
  }

  /**
   * Serialize, encrypt and send command to scooter
   */
  pub async fn send(&mut self, cmd: &ScooterCommand) -> Result<bool> {
    let bytes = encrypt_uart(&self.keys.app, &cmd.as_bytes(), 0, None); // encrypt bytes
    self.protocol.write_nb_parcel(&Registers::TX, &bytes).await?;
    Ok(true)
  }

  /**
   * Wait for response from scooter. You can specify number of frames that you expect to receive
   */
  pub async fn read(&mut self, frames: u8) -> Result<Payload> {
    let data = self.protocol.read_nb_parcel(frames).await?;
    let response = decrypt_uart(&self.keys.dev, &data)?;
    let payload = Payload::from(response);
    Ok(payload)
  }
}
