use core::fmt::Debug;
use uuid::Uuid;
use btleplug::api::ValueNotification;
use pretty_hex::*;

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Registers {
  /**
   * Universal Asynchronous Receiver and Transmitter
   */
  UART,
  TX,
  RX,
  AUTH,
  /**
   * Universal Plug and Play
   */
  UPNP,
  /**
   * Audio/video data transport protocol
   */
  AVDTP
}

impl Registers {
  pub fn to_uuid(&self) -> Uuid {
    let uuid = match self {
      Self::UART => "6e400001-b5a3-f393-e0a9-e50e24dcca9e",
      Self::TX => "6e400002-b5a3-f393-e0a9-e50e24dcca9e",
      Self::RX => "6e400003-b5a3-f393-e0a9-e50e24dcca9e",
      Self::AUTH => "0000fe95-0000-1000-8000-00805f9b34fb",
      Self::UPNP => "00000010-0000-1000-8000-00805f9b34fb",
      Self::AVDTP => "00000019-0000-1000-8000-00805f9b34fb",
    };

    Uuid::parse_str(uuid).expect("Invalid uuid for register")
  }
}

#[allow(non_camel_case_types)]
pub enum MiCommands {
  CMD_GET_INFO,
  CMD_SET_KEY,

  CMD_AUTH,
  CMD_LOGIN,

  CMD_SEND_DATA,
  CMD_SEND_DID,
  CMD_SEND_KEY,
  CMD_SEND_INFO,

  RCV_RDY,
  RCV_OK,

  RCV_AUTH_OK,
  RCV_AUTH_ERR,

  RCV_LOGIN_OK,
  RCV_LOGIN_ERR,
}

impl MiCommands {
  pub fn to_bytes(&self) -> Vec<u8> {
    match self {
      Self::CMD_GET_INFO => vec!(0xa2,0x00,0x00,0x00),
      Self::CMD_SET_KEY => vec!(0x15, 0x00, 0x00, 0x00),
      Self::CMD_SEND_DATA => vec!(0x00, 0x00, 0x00, 0x03, 0x04, 0x00),
      Self::CMD_SEND_DID => vec!(0x00, 0x00, 0x00, 0x00, 0x02, 0x00),
      Self::RCV_RDY => vec!(0x00, 0x00, 0x01, 0x01),
      Self::RCV_OK => vec!(0x00, 0x00, 0x01, 0x00),
      Self::CMD_AUTH => vec!(0x13, 0x00, 0x00, 0x00),
      Self::RCV_AUTH_OK => vec!(0x11, 0x00, 0x00, 0x00),
      Self::RCV_AUTH_ERR => vec!(0x12, 0x00, 0x00, 0x00),
      Self::RCV_LOGIN_OK => vec!(0x21, 0x00, 0x00, 0x00),
      Self::RCV_LOGIN_ERR => vec!(0x23, 0x00, 0x00, 0x00),
      Self::CMD_LOGIN => vec!(0x24, 0x00, 0x00, 0x00),
      Self::CMD_SEND_KEY => vec!(0x00, 0x00, 0x00, 0x0b, 0x01, 0x00),
      Self::CMD_SEND_INFO => vec!(0x00, 0x00, 0x00, 0x0a, 0x02, 0x00)
    }
  }
}

impl Debug for MiCommands {
  fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    //TODO: make this DRY
    match self {
      Self::CMD_GET_INFO => write!(fmt, "CMD_GET_INFO {}", self.to_bytes().hex_dump()),
      Self::CMD_SET_KEY => write!(fmt, "CMD_SET_KEY {}", self.to_bytes().hex_dump()),
      Self::CMD_SEND_DATA => write!(fmt, "CMD_SEND_DATA {}", self.to_bytes().hex_dump()),
      Self::RCV_RDY => write!(fmt, "RCV_RDY {}", self.to_bytes().hex_dump()),
      Self::RCV_OK => write!(fmt, "RCV_OK {}", self.to_bytes().hex_dump()),
      Self::CMD_AUTH => write!(fmt, "CMD_AUTH {}", self.to_bytes().hex_dump()),
      Self::CMD_SEND_DID => write!(fmt, "CMD_SEND_DID {}", self.to_bytes().hex_dump()),
      Self::RCV_AUTH_OK => write!(fmt, "RCV_AUTH_OK {}", self.to_bytes().hex_dump()),
      Self::RCV_AUTH_ERR => write!(fmt, "RCV_AUTH_ERR {}", self.to_bytes().hex_dump()),
      Self::RCV_LOGIN_OK => write!(fmt, "RCV_LOGIN_OK {}", self.to_bytes().hex_dump()),
      Self::RCV_LOGIN_ERR => write!(fmt, "RCV_LOGIN_ERR {}", self.to_bytes().hex_dump()),
      Self::CMD_LOGIN => write!(fmt, "CMD_LOGIN {}", self.to_bytes().hex_dump()),
      Self::CMD_SEND_KEY => write!(fmt, "CMD_SEND_KEY {}", self.to_bytes().hex_dump()),
      Self::CMD_SEND_INFO => write!(fmt, "CMD_SEND_KEY {}", self.to_bytes().hex_dump()),
    }
  }
}

impl TryFrom<ValueNotification> for MiCommands {
  type Error = &'static str;
  fn try_from(data: ValueNotification) -> std::result::Result<Self, <Self as std::convert::TryFrom<ValueNotification>>::Error> {
    if data.value == Self::RCV_RDY.to_bytes() {
      return Ok(Self::RCV_RDY)
    }

    if data.value == Self::CMD_SEND_DATA.to_bytes() {
      return Ok(Self::CMD_SEND_DATA)
    }

    if data.value == Self::RCV_OK.to_bytes() {
      return Ok(Self::RCV_OK)
    }

    if data.value == Self::RCV_AUTH_OK.to_bytes() {
      return Ok(Self::RCV_AUTH_OK)
    }

    if data.value == Self::RCV_AUTH_ERR.to_bytes() {
      return Ok(Self::RCV_AUTH_ERR)
    }

    if data.value == Self::RCV_LOGIN_OK.to_bytes() {
      return Ok(Self::RCV_LOGIN_OK)
    }

    if data.value == Self::RCV_LOGIN_ERR.to_bytes() {
      return Ok(Self::RCV_LOGIN_ERR)
    }

    Err("This is not response")
  }
}
