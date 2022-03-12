use std::hash::Hash;
use anyhow::Result;
use tokio::sync::mpsc;
use std::collections::HashSet;
use futures::stream::StreamExt;
use btleplug::platform::{Adapter, Manager, PeripheralId, Peripheral};
use btleplug::api::{Central, Manager as _, ScanFilter, BDAddr, Peripheral as _, CentralEvent};
use thiserror::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

type Devices = Arc<RwLock<HashSet<TrackedDevice>>>;

/**
 * All xiaomi scooters start with name MIScooter and random numbers after tha
 */
const XIAOMI_SCOOTER_NAME : &str = "MIScooter";

#[derive(Error, Debug)]
pub enum ScannerError {
  #[error("Could not find scooter with addr: {0}")]
  WaitForScooterFailed(BDAddr),
  #[error("Could not find working bluetooth adapter")]
  MissingCentral,
  #[error("Bluetooth error: {0}")]
  BluetoothError(btleplug::Error),
  #[error("Registration failed: {0}")]
  Other(anyhow::Error)
}

impl From<anyhow::Error> for ScannerError {
  fn from(other: anyhow::Error) -> Self {
    ScannerError::Other(other)
  }
}

impl From<btleplug::Error> for ScannerError {
  fn from(other: btleplug::Error) -> Self {
    ScannerError::BluetoothError(other)
  }
}

#[derive(Clone, Debug)]
pub enum ScannerEvent {
  DiscoveredScooter(TrackedDevice)
}

#[derive(Clone, Debug, Hash, Eq)]
pub struct TrackedDevice {
  pub id: PeripheralId,
  pub addr: BDAddr,
  pub name: Option<String>,
}

impl TrackedDevice {
  /**
   * Check if current device is possible the scooter
   */
  pub fn is_scooter(&self) -> bool {
    if let Some(name) = &self.name {
      return name.starts_with(XIAOMI_SCOOTER_NAME);
    }
    return false;
  }
}

impl PartialEq for TrackedDevice {
  fn eq(&self, other: &Self) -> bool {
    self.addr == other.addr
  }
}

/**
 * Use scooter scanner to find scooter.
 * By default all Xiaomi scooter names start with MIScooter and then have few digits after name.
 * If you already know bluetooth mac address of scooter you wan't to connect, you can skip using this scanner
 */
#[derive(Clone)]
pub struct ScooterScanner {
  devices: Devices,
  pub central: Adapter,
}

impl ScooterScanner {
  pub async fn new() -> Result<Self, ScannerError> {
    let manager  = Manager::new().await?;
    let central  = find_central(&manager).await?;
    let devices  = Arc::new(RwLock::new(HashSet::new()));

    Ok(Self { central, devices })
  }

  /**
   * Wait for scooter with mac address to appear and return it.
   */
  pub async fn wait_for(&mut self, scooter_with_address: &BDAddr) -> Result<TrackedDevice, ScannerError> {
    let mut rx = self.start().await?;
    while let Some(event) = rx.recv().await {
      match event {
        ScannerEvent::DiscoveredScooter(scooter) => {
          if scooter.addr == *scooter_with_address {
            tracing::info!("Found your scooter");
            return Ok(scooter)
          } else {
            tracing::info!("Found scooter nearby: {} with mac: {}", scooter.name.unwrap(), scooter.addr);
          }
        }
      }
    }

    Err(ScannerError::WaitForScooterFailed(*scooter_with_address))
  }

  /**
   * Get bluetooth Peripheral/Device using TrackedDevice struct
   */
  pub async fn peripheral(&self, tracked_device : &TrackedDevice) -> Result<Peripheral> {
    Ok(self.central.peripheral(&tracked_device.id).await?)
  }

  /**
   * Start scanning for scooters. This method returns receiver which emits
   * events every time a scooter is visible by bluetooth adapter
   */
  pub async fn start(&mut self) -> Result<mpsc::Receiver<ScannerEvent>> {
    let (tx, rx) = mpsc::channel::<ScannerEvent>(32);
    tracing::debug!("Starting scanning for new devices");
    self.central.start_scan(ScanFilter::default()).await?;

    tracing::debug!("Watching for events in background");
    let central = self.central.clone();
    let devices = self.devices.clone();

    tokio::spawn(async move {
      if let Err(e) = CentralEventsProcessor::new(tx, central, devices).run().await {
        tracing::error!("Stopped processed events {}", e);
      }
    });

    Ok(rx)
  }

  /**
   * Get list of scooters nearby you
   */
  pub async fn scooters(&self) -> Vec<TrackedDevice> {
    self.devices
      .read()
      .await
      .iter()
      .filter(|tracked_device| tracked_device.is_scooter())
      .map(|tracked_device| tracked_device.clone())
      .collect::<Vec<TrackedDevice>>()
  }

  /**
   * Get list of scooters nearby you
   */
  pub async fn devices(&self) -> Vec<TrackedDevice> {
    self.devices
      .read()
      .await
      .iter()
      .map(|tracked_device| tracked_device.clone())
      .collect::<Vec<TrackedDevice>>()
  }
}

struct CentralEventsProcessor {
  central: Adapter,
  tx: mpsc::Sender<ScannerEvent>,
  devices: Devices
}

impl CentralEventsProcessor {
  pub fn new(tx: mpsc::Sender<ScannerEvent>, central: Adapter, devices: Devices) -> Self {
    Self {
      central,
      tx,
      devices
    }
  }

  pub async fn run(&mut self) -> Result<()> {
    let mut events = self.central.events().await?;

    while let Some(event) = events.next().await {
      match event {
        CentralEvent::DeviceDiscovered(peer_id) => {
          if let Some(tracked_device) = self.track_device(&peer_id).await? {
            if tracked_device.is_scooter() {
              self.tx.send(ScannerEvent::DiscoveredScooter(tracked_device)).await?;
            }
          }
        },
        _ => {}
      }
    }
    Ok(())
  }

  async fn track_device(&mut self, peer_id: &PeripheralId) -> Result<Option<TrackedDevice>> {
    tracing::debug!("Discovered peer: {:?}", peer_id);
    let device = self.central.peripheral(peer_id).await?;

    let mut tracked_device = TrackedDevice {
      id: peer_id.clone(),
      addr: device.address(),
      name: None,
    };

    let mut devices = self.devices.write().await;

    if devices.contains(&tracked_device) {
      tracing::debug!("Already discovered: {}", tracked_device.addr);
      Ok(None)
    } else {
      let props = device.properties().await?.unwrap();
      tracing::debug!("Props: {:?}", props);

      let name = props.local_name.unwrap_or("(peripheral name unknown)".to_owned());
      tracing::debug!("Device name: {}", name);
      tracked_device.name = Some(name);

      devices.insert(tracked_device.clone());
      Ok(Some(tracked_device))
    }
  }
}

async fn find_central(manager: &Manager) -> Result<Adapter, ScannerError> {
  let adapters = manager.adapters().await?;

  if let Some(adapter) = adapters.into_iter().nth(0) {
    Ok(adapter)
  } else {
    Err(ScannerError::MissingCentral)
  }
}
