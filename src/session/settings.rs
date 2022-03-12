use super::{MiSession, Payload};
use super::commands::{ScooterCommand, Direction, Attribute, ReadWrite};

use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum Kers {
  Weak,
  Medium,
  Strong,
  Unknown
}

#[derive(Debug, Serialize)]
pub enum TailLight {
  Off,
  OnBrake,
  Always,
  Unknown
}

impl From<u16> for TailLight {
  fn from(byte: u16) -> Self {
    match byte {
      0x0 => TailLight::Off,
      0x1 => TailLight::OnBrake,
      0x2 => TailLight::Always,
      _   => TailLight::Unknown
    }
  }
}

impl From<u16> for Kers {
  fn from(byte: u16) -> Self {
    match byte {
      0x0 => Kers::Weak,
      0x1 => Kers::Medium,
      0x2 => Kers::Strong,
      _   => Kers::Unknown
    }
  }
}

#[derive(Debug, Serialize)]
pub struct SupplementaryInfo {
  kers: Kers,
  is_cruise: bool,
  tail_light: TailLight
}

impl TryFrom<Payload> for SupplementaryInfo {
  type Error = anyhow::Error;

  fn try_from(payload: Payload) -> Result<Self, Self::Error> {
    let mut payload = payload;
    payload.pop_head()?;

    Ok(
      SupplementaryInfo {
        kers: payload.pop_u16()?.into(),
        is_cruise: payload.pop_bool()?,
        tail_light: TailLight::from(payload.pop_u16()?),
      }
    )
  }
}

impl MiSession {
  pub async fn supplementary_info(&mut self) -> Result<SupplementaryInfo> {
    tracing::debug!("Reading supplementary information");

    self.send(&ScooterCommand {
      direction: Direction::MasterToBattery,
      read_write: ReadWrite::Read,
      attribute: Attribute::Supplementary,
      payload: vec![0x06]
    }).await?;

    let payload = self.read(2).await?;

    Ok(SupplementaryInfo::try_from(payload)?)
  }

  pub async fn is_cruise_on(&mut self) -> Result<bool> {
    tracing::debug!("Reading cruise state");

    self.send(&ScooterCommand {
      direction: Direction::MasterToMotor,
      read_write: ReadWrite::Read,
      attribute: Attribute::Cruise,
      payload: vec![0x02]
    }).await?;

    let mut payload = self.read(2).await?;
    payload.pop_head()?;

    Ok(payload.pop_bool()?)
  }

  pub async fn tail_light(&mut self) -> Result<TailLight> {
    tracing::debug!("Reading tail light state");

    self.send(&ScooterCommand {
      direction: Direction::MasterToMotor,
      read_write: ReadWrite::Read,
      attribute: Attribute::TailLight,
      payload: vec![0x02]
    }).await?;

    let mut payload = self.read(2).await?;
    payload.pop_head()?;

    Ok(
      TailLight::from(payload.pop_u16()?)
    )
  }

  pub async fn set_tail_light(&mut self, mode : TailLight) -> Result<()> {
    tracing::debug!("Setting tail light: {:?}", mode);

    let mode : u8 = match mode {
      TailLight::OnBrake => 0x01,
      TailLight::Always => 0x02,
      _ => 0x00
    };

    let payload = vec![mode, 0x00];

    self.send(&ScooterCommand {
      direction: Direction::MasterToMotor,
      read_write: ReadWrite::Write,
      attribute: Attribute::TailLight,
      payload
    }).await?;

    Ok(())
  }

  pub async fn set_cruise(&mut self, on : bool) -> Result<()> {
    tracing::debug!("Setting cruise enabled: {}", on);

    let payload = if on {
      vec![0x01, 0x00]
    } else {
      vec![0x00, 0x00]
    };

    self.send(&ScooterCommand {
      direction: Direction::MasterToMotor,
      read_write: ReadWrite::Write,
      attribute: Attribute::Cruise,
      payload
    }).await?;

    Ok(())
  }
}
