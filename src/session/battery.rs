use super::{MiSession, Payload};
use super::commands::{ScooterCommand, Direction, Attribute, ReadWrite};

use anyhow::Result;
use serde::Serialize;

pub type BatteryCellsVoltage = [f32; 10];

#[derive(Debug, Serialize)]
pub struct BatteryInfo {
  /**
   * Charge left in scooter, in Milliamps (mA)
   */
  pub capacity: u16,
  pub percent: u16,

  /**
   * In Ampers, current current going through battery, you can use it with voltage to calculate wats
   */
  pub current: f32,
  /**
   * Current measured voltage for all batteries, in Volts
   */
  pub voltage: f32,
  pub temperature_1: u8,
  pub temperature_2: u8,
}

impl TryFrom<Payload> for BatteryInfo {
  type Error = anyhow::Error;

  fn try_from(payload: Payload) -> Result<Self, Self::Error> {
    let mut payload = payload;
    payload.pop_head()?;

    Ok(
      BatteryInfo {
        capacity: payload.pop_u16()?,
        percent: payload.pop_u16()?,
        current: payload.pop_i16()? as f32 / 10.0,
        voltage: payload.pop_u16()? as f32 / 100.0,
        temperature_1: payload.pad_byte()?,
        temperature_2: payload.pad_byte()?,
      }
    )
  }
}

impl MiSession {
  /**
   * Battery voltage in volts
   */
  pub async fn battery_voltage(&mut self) -> Result<f32> {
    tracing::debug!("Reading battery voltage");

    self.send(&ScooterCommand {
      direction: Direction::MasterToBattery,
      read_write: ReadWrite::Read,
      attribute: Attribute::BatteryVoltage,
      payload: vec![0x02]
    }).await?;

    let mut payload = self.read(2).await?;
    payload.pop_head()?;

    let voltage = payload.pop_u16()? as f32 / 100.0;

    Ok(voltage)
  }

  /**
   * Return amperage in Ampere
   */
  pub async fn battery_amperage(&mut self) -> Result<f32> {
    tracing::debug!("Reading battery amperage");

    self.send(&ScooterCommand {
      direction: Direction::MasterToBattery,
      read_write: ReadWrite::Read,
      attribute: Attribute::BatteryCurrent,
      payload: vec![0x02]
    }).await?;

    let mut payload = self.read(2).await?;
    payload.pop_head()?;

    let amperage = payload.pop_i16()? as f32 / 10.0;

    Ok(amperage)
  }

  /**
   * Return amperage in Ampere
   */
  pub async fn battery_percentage(&mut self) -> Result<f32> {
    tracing::debug!("Reading battery amperage");

    self.send(&ScooterCommand {
      direction: Direction::MasterToBattery,
      read_write: ReadWrite::Read,
      attribute: Attribute::BatteryPercent,
      payload: vec![0x02]
    }).await?;

    let mut payload = self.read(2).await?;
    payload.pop_head()?;

    let percent = payload.pop_u16()? as f32;

    Ok(percent)
  }

  pub async fn battery_cell_voltages(&mut self) -> Result<BatteryCellsVoltage> {
    tracing::debug!("Reading battery cell voltages");

    self.send(&ScooterCommand {
      direction: Direction::MasterToBattery,
      read_write: ReadWrite::Read,
      attribute: Attribute::BatteryCellVoltages,
      payload: vec![0x1B]
    }).await?;

    let mut payload = self.read(3).await?;
    payload.pop_head()?;

    let voltages : BatteryCellsVoltage = [
      payload.pop_u16()? as f32 / 100.0,
      payload.pop_u16()? as f32 / 100.0,
      payload.pop_u16()? as f32 / 100.0,
      payload.pop_u16()? as f32 / 100.0,
      payload.pop_u16()? as f32 / 100.0,
      payload.pop_u16()? as f32 / 100.0,
      payload.pop_u16()? as f32 / 100.0,
      payload.pop_u16()? as f32 / 100.0,
      payload.pop_u16()? as f32 / 100.0,
      payload.pop_u16()? as f32 / 100.0,
    ];

    Ok(voltages)
  }

  pub async fn battery_info(&mut self) -> Result<BatteryInfo> {
    self.send(&ScooterCommand {
      direction: Direction::MasterToBattery,
      read_write: ReadWrite::Read,
      attribute: Attribute::BatteryInfo,
      payload: vec![0x0A]
    }).await?;

    let payload = self.read(2).await?;

    Ok(
      BatteryInfo::try_from(payload)?
    )
  }
}
