use crate::consts::{MiCommands, Registers};
use uuid::Uuid;
use futures::Stream;
use futures::stream::StreamExt;
use pretty_hex::*;
use std::{pin::Pin, boxed::Box};
use btleplug::platform::{Peripheral};
use tokio::time::timeout;
use std::time::Duration;
use btleplug::api::{Peripheral as _, Characteristic, WriteType, ValueNotification};
use anyhow::{Context, Result, anyhow};

const NB_CHUNK_SIZE : usize = 20;
const MI_CHUNK_SIZE : usize = 18;

/**
 * This structs hides all bluetooth shenanigans under easy to use commands.
 */
pub struct MiProtocol {
  device: Peripheral,
  avdtp: Characteristic,
  upnp: Characteristic,
  tx: Characteristic,
  rx: Characteristic,
  stream: Pin<Box<dyn Stream<Item = ValueNotification>>>,
}

impl MiProtocol {
  pub async fn new(device: &Peripheral) -> Result<Self> {
    let (avdtp, upnp, tx, rx) = setup_channels(&device).await?;
    let stream : Pin<Box<dyn Stream<Item = ValueNotification>>> = device.notifications().await
      .with_context(|| format!("Could not load notifications stream"))?;
    let device = device.clone();

    let instance = Self {
      device,
      stream,
      avdtp,
      upnp,
      tx,
      rx
    };

    Ok(instance)
  }

  pub async fn dispose(&self) -> Result<bool> {
    self.device.unsubscribe(&self.avdtp).await?;
    self.device.unsubscribe(&self.upnp).await?;
    self.device.unsubscribe(&self.rx).await?;

    Ok(true)
  }

  fn reg_to_channel(&self, reg : &Registers) -> Option<&Characteristic> {
    match reg {
      Registers::RX => Some(&self.rx),
      Registers::TX => Some(&self.tx),
      Registers::AVDTP => Some(&self.avdtp),
      Registers::UPNP => Some(&self.upnp),
      _ => None
    }
  }

  /**
   * Read next notification
   */
  pub async fn next(&mut self) -> Option<ValueNotification> {
    tracing::debug!("Waiting for notifications...");
    self.stream.next().await
  }

  pub async fn wait_for_scooter_to_receive_data(&mut self) -> Result<bool> {
    match self.next_mi_response().await {
      Some(MiCommands::RCV_RDY) => Ok(true),
      Some(state) => Err(anyhow!("Expected state: {:?}, but received: {:?}", MiCommands::RCV_RDY, state)),
      None => Err(anyhow!("Invalid response received from scooter"))
    }
  }

  pub async fn wait_for_scooter_to_ack_data(&mut self) -> Result<bool> {
    match self.next_mi_response().await {
      Some(MiCommands::RCV_OK) => Ok(true),
      Some(state) => Err(anyhow!("Expected state: {:?}, but received: {:?}", MiCommands::RCV_RDY, state)),
      None => Err(anyhow!("Invalid response received from scooter"))
    }
  }

  /**
   * Try to read next notification as MiCommand response
   */
  pub async fn next_mi_response(&mut self) -> Option<MiCommands> {
    if let Some(data) = self.next().await {
      if let Ok(cmd) = MiCommands::try_from(data.clone()) {
        tracing::debug!("<- {:?}", cmd);
        return Some(cmd)
      } else {
        tracing::debug!("These bytes don't look like mi response: {:?}", data);
      }
    }

    None
  }

  /**
   * Try to read next notification, If nothing comes in specified duration throw error
   */
  pub async fn wait_for_notification_with_timeout(&mut self, duration : Duration) -> Result<ValueNotification> {
    let response = timeout(duration, self.next()).await?;//TODO: map to timeout error

    if let Some(notification) = response {
      return Ok(notification)
    }

    Err(anyhow!("Received empty message from mi scooter..."))
  }

  /**
   * Try to read next notification, If nothing comes in 2 seconds raise error.
   */
  pub async fn wait_for_notification(&mut self) -> Result<ValueNotification> {
    self.wait_for_notification_with_timeout(Duration::from_secs(2)).await
  }

  /**
   * Send mi command to register on scooter
   */
  pub async fn write(&self, reg: &Registers, command: MiCommands) -> Result<bool> {
    let channel = self.reg_to_channel(reg).unwrap();
    tracing::debug!("-> {:?} -> {:?}", command, &reg);

    self.device.write(&channel, &command.to_bytes(), WriteType::WithoutResponse).await
      .with_context(|| format!("Could not write command: {:?} to {:?}", command, &reg))?;

    Ok(true)
  }

  /**
   * Ninebot protocol sends multiple messages. I don't know how long they will be, but this is persistent per command, so you can specify it as arg
   */
  pub async fn read_nb_parcel(&mut self, frames: u8) -> Result<Vec<u8>> {
    let mut buffer : Vec<u8> = Vec::new();
    let mut frames_left = frames;
    let duration = Duration::from_secs(5);

    tracing::debug!("Reading nb frames: {}", frames_left);
    while frames_left > 0 {
      tracing::debug!("  Reading frame...");
      let notification = self.wait_for_notification_with_timeout(duration).await?;
      tracing::debug!("  Received data: {:?}", notification.value.hex_dump());
      buffer.extend_from_slice(notification.value.as_slice());
      frames_left -= 1;
    }

    tracing::debug!("  Finished reading: {:?}", buffer.hex_dump());
    Ok(buffer)
  }

  /**
   * Read parcel data send in multiple messages from scooter using mi protocol
   */
  pub async fn read_mi_parcel(&mut self, reg: &Registers) -> Result<Vec<u8>> {
    tracing::debug!("Reading parcel...");

    let mut total_frames : u16 = 0;
    let mut received_data : Vec<u8> = Vec::new();

    if let Some(data) = self.stream.next().await {
      total_frames = data.value[4] as u16 + 0x100 * data.value[5] as u16;
      tracing::debug!("Expecting {} frames: {:?}", total_frames, data.value.hex_dump());

      self.write(reg, MiCommands::RCV_RDY).await?;
    }

    while let Some(data) = self.stream.next().await {
      let current_frame : u16 = what_frame(&data.value);
      tracing::debug!("Current frame {}: {:?}", current_frame, data.value.hex_dump());

      for i in 2..data.value.len() {
        received_data.push(data.value[i]);
      }

      if current_frame == total_frames {
        break;
      }
    }

    tracing::debug!("All frames received: {:?}", received_data.hex_dump());
    self.write(reg, MiCommands::RCV_OK).await?;

    Ok(received_data)
  }

  pub async fn write_nb_parcel(&self, reg: &Registers, data: &[u8]) -> Result<bool> {
    let channel = self.reg_to_channel(reg).unwrap();

    for chunk in data.chunks(NB_CHUNK_SIZE) {
      tracing::debug!("Writing nb chunk to {:?}: {:?}", reg, chunk.hex_dump());
      self.device.write(&channel, &chunk, WriteType::WithoutResponse).await
        .with_context(|| format!("Could not write mi chunk: for channel: {:?}", channel))?;
    }

    Ok(true)
  }

  /**
   * Send big data parcel to scooter using mi protocol
   */
  pub async fn write_mi_parcel(&self, reg: &Registers, data: &[u8]) -> Result<bool> {
    let mut buffer : Vec<u8> = Vec::new();
    let mut chunk_index = 1;
    let channel = self.reg_to_channel(reg).unwrap();

    for chunk in data.chunks(MI_CHUNK_SIZE) {
      buffer.clear();
      buffer.push(chunk_index);
      buffer.push(0);

      for byte in chunk { // There should be better way of doing this...
        buffer.push(*byte);
      }

      tracing::debug!("Writing mi chunk {} to {:?}: {:?}", chunk_index, reg, buffer.hex_dump());
      self.device.write(&channel, &buffer, WriteType::WithoutResponse).await
        .with_context(|| format!("Could not write mi chunk: {} for channel: {:?}", chunk_index, channel))?;
      chunk_index += 1;
    }

    Ok(true)
  }
}

async fn find_characteristic(device : &Peripheral, service_uuid: Uuid, char_uuid: Uuid) -> Result<Characteristic> {
  device.discover_services().await
    .with_context(|| format!("Could not enable discovering devices"))?;

  for ch in device.characteristics() {
    if ch.uuid == char_uuid && ch.service_uuid == service_uuid {
      tracing::debug!("Found Characteristic: {:?}", ch);
      return Ok(ch)
    } else {
      tracing::debug!("Skipped Characteristic: {:?}", ch);
    }
  }

  Err(anyhow!("Could not find characteristic: {}", char_uuid))
}

fn what_frame(bytes: &Vec<u8>) -> u16 {
  bytes[0] as u16 & 0xff + 0x100 * bytes[1] as u16 & 0xff
}

async fn setup_channels(device : &Peripheral) -> Result<(Characteristic, Characteristic, Characteristic, Characteristic)> {
  device.discover_services().await?;

  // Auth channels
  tracing::debug!("Setting up AUTH channels");
  let avdtp = find_characteristic(device, Registers::AUTH.to_uuid(), Registers::AVDTP.to_uuid()).await?;
  let upnp = find_characteristic(device, Registers::AUTH.to_uuid(), Registers::UPNP.to_uuid()).await?;

  // UART channels
  tracing::debug!("Setting up UART channels");
  let tx = find_characteristic(device, Registers::UART.to_uuid(), Registers::TX.to_uuid()).await?;
  let rx = find_characteristic(device, Registers::UART.to_uuid(), Registers::RX.to_uuid()).await?;

  tracing::debug!("Enabling notify for AVDTP");
  device.subscribe(&avdtp).await
    .with_context(|| format!("Could not subscribe to scooter AVDTP notifications"))?;

  tracing::debug!("Enabling notify for UPNP");
  device.subscribe(&upnp).await
    .with_context(|| format!("Could not subscribe to scooter UPNP notifications"))?;

  tracing::debug!("Enabling notify for RX");
  device.subscribe(&rx).await
    .with_context(|| format!("Could not subscribe to scooter RX notifications"))?;

  Ok((avdtp, upnp, tx, rx))
}
