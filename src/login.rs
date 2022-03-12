use crate::mi_crypto::{
  AuthToken, RandKey, LoginKeychain,
  gen_rand_key, calc_login_did
};
use crate::session::MiSession;
use crate::consts::{MiCommands, Registers};
use crate::protocol::MiProtocol;
use anyhow::Result;
use pretty_hex::*;
use btleplug::platform::Peripheral;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoginError {
  #[error("Failed at creating session for scooter connection")] //TODO: less retarded error message
  LoginFailed,
  #[error("Scooter sent invalid remote key")]
  InvalidDid,
  #[error("Login failed: {0}")]
  Other(anyhow::Error)
}

impl From<anyhow::Error> for LoginError {
  fn from(other: anyhow::Error) -> Self {
    LoginError::Other(other)
  }
}

/**
 * Login to scooter. All communication over bluetooth is encrypted with special keys, to retrieve
 * these keys you need first to login using auth token, which you get using RegistrationRequest.
 * If everything goes right, you will receive MiSession which allows you to send commands and read
 * read responses back from the scooter
 */
pub struct LoginRequest {
  protocol: MiProtocol,
  auth_token: AuthToken,
  rand_key: RandKey,
  device: Peripheral,
  remote_info: Option<[u8; 32]>,
  keys: Option<LoginKeychain>,
  remote_key: Option<Vec<u8>>,
}

impl LoginRequest {
  pub async fn new(device : &Peripheral, token: &AuthToken) -> Result<Self> {
    let protocol = MiProtocol::new(device).await?;
    let rand_key = gen_rand_key();

    Ok(
      Self {
        remote_info: None,
        remote_key: None,
        keys: None,
        rand_key,
        protocol,
        device: device.clone(),
        auth_token: token.clone()
      }
    )
  }

  pub async fn start(&mut self) -> Result<MiSession> {
    self.send_key().await?;
    self.read_remote_key().await?;
    self.read_remote_info().await?;
    self.validate_remote_key_and_send_did().await?;
    self.confirm().await?;

    self.protocol.dispose().await?;
    let keys = self.keys.as_ref().unwrap();
    let session = MiSession::new(&self.device, keys).await?;
    Ok(session)
  }

  async fn send_key(&mut self) -> Result<bool> {
    self.protocol.write(&Registers::UPNP, MiCommands::CMD_LOGIN).await?;
    self.protocol.write(&Registers::AVDTP, MiCommands::CMD_SEND_KEY).await?;

    self.protocol.wait_for_scooter_to_receive_data().await?;
    self.protocol.write_mi_parcel(&Registers::AVDTP, &self.rand_key).await?;
    self.protocol.wait_for_scooter_to_ack_data().await?;

    Ok(true)
  }

  async fn read_remote_key(&mut self) -> Result<bool> {
    tracing::debug!("<- remote_key");
    let remote_key = self.protocol.read_mi_parcel(&Registers::AVDTP).await?;
    self.remote_key = Some(remote_key);

    Ok(true)
  }

  async fn read_remote_info(&mut self) -> Result<bool> {
    tracing::debug!("<- remote_info");
    let remote_info = self.protocol.read_mi_parcel(&Registers::AVDTP).await?;
    self.remote_info = Some(remote_info.try_into().unwrap());

    Ok(true)
  }

  async fn validate_remote_key_and_send_did(&mut self) -> Result<bool, LoginError> {
    tracing::info!("Validating did");

    let rand_key = self.rand_key.as_mut();
    let remote_key = self.remote_key.as_mut().unwrap();
    let remote_info = self.remote_info.unwrap();

    let (info, expected_remote_info, keys) = calc_login_did(rand_key, remote_key, &self.auth_token);
    if remote_info == expected_remote_info {
      tracing::debug!("Remote info is as expected, sending did");

      self.protocol.write(&Registers::AVDTP, MiCommands::CMD_SEND_INFO).await?;

      self.protocol.wait_for_scooter_to_receive_data().await?;
      self.protocol.write_mi_parcel(&Registers::AVDTP, &info).await?;
      self.protocol.wait_for_scooter_to_ack_data().await?;
      self.keys = Some(keys);

      return Ok(true)
    }

    tracing::error!("Scooter send invalid remote key:");
    tracing::error!("   Expected: {:?}", expected_remote_info.hex_dump());
    tracing::error!("   Received: {:?}", remote_info.hex_dump());

    Err(LoginError::InvalidDid)
  }

  async fn confirm(&mut self) -> Result<bool, LoginError> {
    match self.protocol.next_mi_response().await {
      Some(MiCommands::RCV_LOGIN_OK) => {
        tracing::info!("Logged in!");
      },

      Some(error) => {
        tracing::error!("Login failed: {:?}", error);
        return Err(LoginError::LoginFailed)
      },

      None => {
        tracing::error!("Login failed, scooter did not respond");
        return Err(LoginError::LoginFailed)
      }
    }

    Ok(true)
  }
}
