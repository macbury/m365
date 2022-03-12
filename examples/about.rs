use anyhow::{Result, Context};
use btleplug::api::{BDAddr};
use tracing_subscriber;
use tokio::io::AsyncReadExt;
use std::path::Path;
use tokio::fs::File;
use tracing::Level;
use std::env;
use tracing_subscriber::fmt::format::FmtSpan;
use m365::{
  AuthToken,
  ScooterScanner,
  LoginRequest,
  ConnectionHelper
};

async fn load_token() -> Result<AuthToken> {
  let path = Path::new(".mi-token");
  tracing::debug!("Opening token: {:?}", path);

  let mut f = File::open(path).await?;
  let mut buffer : AuthToken = [0; 12];

  f.read(&mut buffer).await?;

  Ok(buffer)
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()>{
  tracing_subscriber::fmt()
    .with_max_level(Level::INFO)
    .with_span_events(FmtSpan::CLOSE)
    .init();

  let args: Vec<String> = env::args().collect();
  if args.len() < 2 || args[1].is_empty() {
    panic!("First argument is scooter mac address");
  }

  let token = load_token().await
    .with_context(|| "Could not load registration token")?;

  let mac = BDAddr::from_str_delim(&args[1]).expect("Invalid mac address");
  tracing::info!("Searching scooter with address: {}", mac);

  let mut scanner = ScooterScanner::new().await?;
  let scooter = scanner.wait_for(&mac).await?;
  let device = scanner.peripheral(&scooter).await?;
  let connection = ConnectionHelper::new(&device);
  connection.reconnect().await?;

  let mut request = LoginRequest::new(&device, &token).await?;
  let mut session = request.start().await?;

  tracing::info!("Logged in with success, reading data...");

  tracing::info!("  Battery info: {:?}", session.battery_info().await?);
  tracing::info!("  Battery cells (V): {:?}", session.battery_cell_voltages().await?);
  tracing::info!("  Serial number {}", session.serial_number().await?);
  tracing::info!("  Motor info: {:?}", session.motor_info().await?);
  tracing::info!("  Supplementary info {:?}", session.supplementary_info().await?);
  tracing::info!("  General info {:?}", session.general_info().await?);
  tracing::info!("  Distance left {} km", session.distance_left().await?);
  tracing::info!("  Trip distance {} km", session.trip_distance().await?);
  tracing::info!("  Current Speed {} km/h", session.speed().await?);
  tracing::info!("  Cruise enabled: {}", session.is_cruise_on().await?);
  tracing::info!("  Tail light enabled: {:?}", session.tail_light().await?);
  tracing::info!("  Battery {} V", session.battery_voltage().await?);
  tracing::info!("          {} A", session.battery_amperage().await?);
  tracing::info!("          {} %", session.battery_percentage().await?);
  tracing::info!("          {:?}", session.battery_info().await?);

  Ok(())
}
