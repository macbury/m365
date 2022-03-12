use crate::consts::{MiCommands, Registers};
pub use crate::mi_crypto::AuthToken;
use crate::protocol::MiProtocol;
use crate::mi_crypto;

use pretty_hex::*;
use btleplug::platform::Peripheral;
use p256::{PublicKey, ecdh::EphemeralSecret, EncodedPoint};
use anyhow::{Result, anyhow};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegistrationError {
  #[error("There was problem registering scooter")]
  RegistrationFailed,
  #[error("Please restart connection and try again")]
  RestartNeeded,
  #[error("Registration failed: {0}")]
  Other(anyhow::Error)
}

impl From<anyhow::Error> for RegistrationError {
  fn from(other: anyhow::Error) -> Self {
    RegistrationError::Other(other)
  }
}

pub struct RegistrationRequest {
  protocol: MiProtocol,
  my_secret_key: EphemeralSecret,
  my_public_key: PublicKey,
  remote_info: Option<Vec<u8>>,
  token: Option<AuthToken>
}

impl RegistrationRequest {
  /**
   * Create new registration request for device. It is important that device is a M365 scooter, and you did already connect to it
   */
  pub async fn new(device : &Peripheral) -> Result<Self> {
    let protocol = MiProtocol::new(device).await?;

    let (my_secret_key, my_public_key) = mi_crypto::gen_key_pair();
    tracing::debug!("Public key: {:?}", my_public_key);

    let request = Self {
      protocol,
      my_secret_key,
      my_public_key,
      remote_info: None,
      token: None
    };

    Ok(request)
  }

  /**
   * Starting registration process. In some cases there will be RegistrationError.
   * For this error please disconnect and connect again to scooter and ask user to press power button. Remember to create new instance of
   * RegistrationRequest and start process again. I know this sucks but this is how it works.
   */
  pub async fn start(&mut self) -> Result<AuthToken, RegistrationError> {
    self.read_remote_info().await?;
    self.send_public_key().await?;
    self.send_did().await?;
    self.perform_auth().await?;

    Ok(self.token.unwrap())
  }

  /**
   * Get remote info, this is used for generating token and did that is sent to scooter
   */
  async fn read_remote_info(&mut self) -> Result<bool> {
    self.protocol.write(&Registers::UPNP, MiCommands::CMD_GET_INFO).await?;

    tracing::debug!("<- remote_info");
    let remote_info = self.protocol.read_mi_parcel(&Registers::AVDTP).await?;
    self.remote_info = Some(remote_info);

    Ok(true)
  }

  /**
   * Send public key to scooter and then wait for scooter with
   */
  async fn send_public_key(&mut self) -> Result<bool, RegistrationError> {
    self.protocol.write(&Registers::UPNP, MiCommands::CMD_SET_KEY).await?;
    self.protocol.write(&Registers::AVDTP, MiCommands::CMD_SEND_DATA).await?;

    let notification = self.protocol.wait_for_notification().await;

    if notification.is_err() {
      return Err(RegistrationError::RestartNeeded)
    }

    let notification = notification.unwrap();

    match MiCommands::try_from(notification) {
      Ok(MiCommands::RCV_RDY) => {
        tracing::debug!("<- {:?}", MiCommands::RCV_RDY);
        let public_key_bytes = EncodedPoint::from(self.my_public_key);
        tracing::debug!("-> Mi ready to receive key, uploading my public key: {:?}", public_key_bytes.as_bytes().hex_dump());
        self.protocol.write_mi_parcel(&Registers::AVDTP, &public_key_bytes.as_bytes()[1..]).await?;
      },
      Ok(other) => {
        tracing::debug!("Could not match: {:?}", other);
        return Err(RegistrationError::Other(anyhow!("Scooter responded with: {:?} instead RCV_RDY", other)))
      }
      Err(err) => {
        return Err(RegistrationError::Other(anyhow!(err)))
      }
    }

    if let Some(MiCommands::RCV_OK) = self.protocol.next_mi_response().await {
      tracing::debug!("Mi confirmed key receive");

      return Ok(true)
    }

    Err(RegistrationError::Other(anyhow!("Sending public key failed...")))
  }

  async fn send_did(&mut self) -> Result<bool> {
    let remote_key_bytes = self.protocol.read_mi_parcel(&Registers::AVDTP).await?;
    let remote_info = self.remote_info.as_ref().unwrap();
    let remote_key_bytes = [&[0x04], remote_key_bytes.as_slice()].concat();
    let (did_ct, token) = mi_crypto::calc_did(&self.my_secret_key, &remote_key_bytes, &remote_info);

    self.token = Some(token);
    self.protocol.write(&Registers::AVDTP, MiCommands::CMD_SEND_DID).await?;

    loop {
      match self.protocol.next_mi_response().await {
        Some(MiCommands::RCV_RDY) => {
          tracing::debug!("Mi ready to receive, Sending did");
          self.protocol.write_mi_parcel(&Registers::AVDTP, &did_ct).await?;
        },
        Some(MiCommands::RCV_OK) => {
          tracing::debug!("Mi confirmed receiving did");
          break;
        }
        _ => {
          tracing::error!("Scooter did not receive public key");
          return Err(anyhow!("Scooter did not receive public key"));
        }
      }
    }

    Ok(true)
  }

  async fn perform_auth(&mut self) -> Result<bool, RegistrationError> {
    self.protocol.write(&Registers::UPNP, MiCommands::CMD_AUTH).await?;
    match self.protocol.next_mi_response().await {
      Some(MiCommands::RCV_AUTH_OK) => {
        tracing::info!("Registered token: {:?}", self.token.unwrap().hex_dump());
      },

      Some(error) => {
        // something bad happened, error
        tracing::error!("Registration failed: {:?}", error);
        return Err(RegistrationError::RegistrationFailed)
      },

      None => {
        tracing::error!("Registration failed, scooter did not respond");
        return Err(RegistrationError::RegistrationFailed)
      }
    }

    Ok(true)
  }
}
