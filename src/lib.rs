extern crate uuid;

pub mod mi_crypto;
pub mod protocol;
pub mod consts;

mod register;
mod scanner;
mod connection;
mod login;
mod session;

pub use register::RegistrationRequest as RegistrationRequest;
pub use register::RegistrationError as RegistrationError;
pub use mi_crypto::AuthToken as AuthToken;
pub use login::LoginRequest as LoginRequest;
pub use scanner::ScooterScanner as ScooterScanner;
pub use scanner::ScannerError as ScannerError;
pub use scanner::ScannerEvent as ScannerEvent;
pub use scanner::TrackedDevice as TrackedDevice;
pub use connection::ConnectionHelper as ConnectionHelper;

pub use session::{
  MiSession as MiSession,
  Payload,
  MotorInfo,
  GeneralInfo,
  TailLight,
  BatteryInfo
};
