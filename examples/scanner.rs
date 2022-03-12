use tracing_subscriber;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use anyhow::Result;
use m365::{ScooterScanner, ScannerEvent};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
  tracing_subscriber::fmt()
    .with_max_level(Level::DEBUG)
    .with_span_events(FmtSpan::CLOSE)
    .init();

  let scanner = ScooterScanner::new().await?;
  let mut rx = scanner.clone().start().await?;

  while let Some(event) = rx.recv().await {
    match event {
      ScannerEvent::DiscoveredScooter(scooter) => {
        tracing::info!("Found scooter nearby: {} with mac: {}", scooter.name.unwrap(), scooter.addr);
        tracing::debug!("All devices: {:?}", scanner.devices().await);
      }
    }
  }

  Ok(())
}
