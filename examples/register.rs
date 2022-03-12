use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

use btleplug::platform::{Peripheral};
use btleplug::api::BDAddr;
use tokio::io::{BufWriter, AsyncWriteExt};
use tokio::fs::File;
use pretty_hex::*;
use std::env;
use tracing_subscriber;
use std::path::Path;
use anyhow::Result;

use m365::{
  ScooterScanner, ScannerEvent,
  RegistrationRequest, RegistrationError,
  ConnectionHelper, AuthToken
};

async fn save_token(token : &AuthToken) -> Result<()> {
  let path = Path::new(".mi-token");
  tracing::info!("Saving token at {:?} with content {:?}", path, token.hex_dump());
  let f = File::create(path).await?;
  {
    let mut writer = BufWriter::new(f);
    writer.write(token).await?;
    writer.flush().await?;
  }
  Ok(())
}

async fn register(device: &Peripheral) -> Result<()> {
  let connection = ConnectionHelper::new(&device);

  loop {
    tracing::info!(">>> Press power button up to 5 seconds after beep!");
    connection.reconnect().await?;
    let mut request = RegistrationRequest::new(&device).await?;

    match request.start().await {
      Ok(token) => {
        save_token(&token).await?;
        break;
      },
      Err(RegistrationError::RestartNeeded) => {
        tracing::debug!("Restarting...");
        continue;
      },
      Err(_) => {
        tracing::error!("Unhandled error...");
      }
    }
  }

  Ok(())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
  tracing_subscriber::fmt()
    .with_max_level(Level::DEBUG)
    .with_span_events(FmtSpan::CLOSE)
    .init();

  let args: Vec<String> = env::args().collect();
  if args.len() < 2 || args[1].is_empty() {
    panic!("First argument is scooter mac address");
  }

  let mac = BDAddr::from_str_delim(&args[1]).expect("Invalid mac address");
  tracing::info!("Searching scooter with address: {}", mac);

  let mut scanner = ScooterScanner::new().await?;
  let mut rx = scanner.start().await?;

  while let Some(event) = rx.recv().await {
    match event {
      ScannerEvent::DiscoveredScooter(scooter) => {
        if scooter.addr == mac {
          tracing::info!("Found your scooter, starting registration");
          let device = scanner.peripheral(&scooter).await?;
          register(&device).await?;
          break;
        } else {
          tracing::info!("Found scooter nearby: {} with mac: {}", scooter.name.unwrap(), scooter.addr);
        }
      }
    }
  }

  Ok(())
}
