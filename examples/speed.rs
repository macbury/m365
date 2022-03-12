use anyhow::Result;
use btleplug::api::{BDAddr};
use tracing_subscriber;
use tokio::io::AsyncReadExt;
use std::path::Path;
use tokio::fs::File;
use tracing::Level;
use std::env;
use tokio::time;
use std::time::Duration;
use tracing_subscriber::fmt::format::FmtSpan;
use m365::{
  AuthToken,
  ScooterScanner,
  LoginRequest,
  ConnectionHelper,
  MiSession
};

async fn load_token() -> Result<AuthToken> {
  let path = Path::new(".mi-token");
  tracing::debug!("Opening token: {:?}", path);

  let mut f = File::open(path).await?;
  let mut buffer : AuthToken = [0; 12];

  f.read(&mut buffer).await?;

  Ok(buffer)
}

async fn read(session : &mut MiSession) -> Result<()> {
  tracing::info!("  Current Speed {} km/h", session.speed().await?);
  tracing::info!("  Motor info: {:?}", session.motor_info().await?);
  tracing::info!("  Battery info: {:?}", session.battery_info().await?);
  Ok(())
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

  let token = load_token().await?;

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

  loop {
    read(&mut session).await;
    time::sleep(Duration::from_millis(1000)).await;
  }
}
