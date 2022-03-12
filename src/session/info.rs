use super::{MiSession, Payload};
use super::commands::{ScooterCommand, Direction, Attribute, ReadWrite};

use std::time::Duration;
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GeneralInfo {
  serial: String,
  pin: String,
  version: String
}

#[derive(Debug, Serialize)]
pub struct MotorInfo {
  /**
   * Percent value between 0 and 100
   */
  pub battery_percent: u16,
  /**
   * Speed in kilometers per hour
   */
  pub speed_kmh: f32,
  /**
   * Speed in kilometers per hour
   */
  pub speed_average_kmh: f32,
  /**
   * Distance is in meters
   */
  pub total_distance_m: u32,
    /**
   * Distance is in meters
   */
  pub trip_distance_m: i16,
  pub uptime: Duration,
  /**
   * Temperature in celsius
   */
  pub frame_temperature: f32
}

impl TryFrom<Payload> for MotorInfo {
  type Error = anyhow::Error;

  fn try_from(payload: Payload) -> Result<Self, Self::Error> {
    let mut payload = payload;
    payload.pop_head()?;
    payload.pad_bytes(8)?; // ---Var179=¿workmode?=0x0000

    let battery_percent = payload.pop_u16()?; // ---Var180=%batt=0x003d=61%
    let speed_kmh = payload.pop_i16()? as f32 / 1000.0; // ---Var181=¿velocidad metros/h?=0x0000=0km/h
    let speed_average_kmh = payload.pop_u16()? as f32 / 1000.0; // ---Var182=¿velocidad prom m/h?=0x4650=18km/h
    let total_distance_m = payload.pop_u32()?; // ---Var183-184=m-total=0x0000088a=2.1km
    let trip_distance_m = payload.pop_i16()?; // ---Var185=¿?=0x0005=5
    let uptime_s = payload.pop_i16()?; // ---Var186=¿?=0x027c=636
    let frame_temperature = payload.pop_i16()? as f32 / 10.0; // 	---Var187=temp*10=0x0118=28°C

    Ok(
      MotorInfo {
        battery_percent,
        speed_kmh,
        speed_average_kmh,
        total_distance_m,
        trip_distance_m,
        uptime: Duration::from_secs(uptime_s as u64),
        frame_temperature
      }
    )
  }
}

impl MiSession {
  pub async fn general_info(&mut self) -> Result<GeneralInfo> {
    tracing::debug!("Reading general information");

    let cmd = ScooterCommand {
      direction: Direction::MasterToMotor,
      read_write: ReadWrite::Read,
      attribute: Attribute::GeneralInfo,
      payload: vec![0x16]
    };

    self.send(&cmd).await?;
    //          [                      SERIAL                          ][          PIN         ][ VER  ]
    // payload: /x31/x36/x31/x33/x32/x2f/x30/x30/x30/x39/x35/x32/x39/x32/x30/x30/x30/x30/x30/x30/x38/x01
    let mut payload = self.read(2).await?;

    payload.pop_head()?;

    let serial = payload.pop_string_utf8(11)?;
    let pin = payload.pop_string_utf8(6)?;
    let version = payload.pop_string_utf8(2)?;

    Ok(GeneralInfo { serial, pin, version })
  }

  /**
   * Read scooter serial number
   */
  pub async fn serial_number(&mut self) -> Result<String> {
    tracing::debug!("Reading serial number");
    let cmd = ScooterCommand {
      direction: Direction::MasterToMotor,
      read_write: ReadWrite::Read,
      attribute: Attribute::GeneralInfo,
      payload: vec![0x0e]
    };

    self.send(&cmd).await?;
    let mut payload = self.read(2).await?;
    payload.pop_head()?;

    let serial = payload.pop_string_utf8(14)?;

    Ok(serial)
  }

  pub async fn motor_info(&mut self) -> Result<MotorInfo> {
    tracing::debug!("Reading motor info");

    self.send(&ScooterCommand {
      direction: Direction::MasterToMotor,
      read_write: ReadWrite::Read,
      attribute: Attribute::MotorInfo,
      payload: vec![0x20]
    }).await?;

    let payload = self.read(3).await?;

    MotorInfo::try_from(payload)
  }
}
