use super::MiSession;
use super::commands::{ScooterCommand, Direction, Attribute, ReadWrite};

use anyhow::Result;

impl MiSession {
  /**
   * Get travel distance left in kilometers
   */
  pub async fn distance_left(&mut self) -> Result<f32> {
    tracing::debug!("Reading distance left");

    let cmd = ScooterCommand {
      direction: Direction::MasterToMotor,
      read_write: ReadWrite::Read,
      attribute: Attribute::DistanceLeft,
      payload: vec![0x02]
    };

    self.send(&cmd).await?;

    let mut payload = self.read(2).await?;
    payload.pop_head()?;

    let distance_left = payload.pop_u16()?;
    let distance_left = distance_left as f32 / 100.0;
    tracing::debug!("Distance left: {}km", distance_left);

    Ok(distance_left)
  }

  /**
   * Get current speed in kilometers per hour
   */
  pub async fn speed(&mut self) -> Result<f32> {
    tracing::debug!("Reading speed");

    let cmd = ScooterCommand {
      direction: Direction::MasterToMotor,
      read_write: ReadWrite::Read,
      attribute: Attribute::Speed,
      payload: vec![0x02]
    };

    self.send(&cmd).await?;

    let mut payload = self.read(2).await?;
    payload.pop_head()?;

    let speed = payload.pop_i16()?;
    let speed = speed as f32 / 1000.0;
    tracing::debug!("speed: {}km/h", speed);

    Ok(speed)
  }

  /**
   * Read current travel distance in meters
   */
  pub async fn trip_distance(&mut self) -> Result<u16> {
    tracing::debug!("Reading distance");

    let cmd = ScooterCommand {
      direction: Direction::MasterToMotor,
      read_write: ReadWrite::Read,
      attribute: Attribute::TripDistance,
      payload: vec![0x02]
    };

    self.send(&cmd).await?;

    let mut payload = self.read(3).await?;
    payload.pop_head()?;

    let trip_distance = payload.pop_u16()?;

    Ok(trip_distance)
  }
}
