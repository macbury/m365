use core::fmt::Debug;
use pretty_hex::*;
use anyhow::{Result, anyhow, Context};

/**
 * Represents decrypted payload received from the scooter. Payload also have methods which helps to read each value encoded in payload
 */
pub struct Payload {
  /**
   * Bytes are stored in reverse
   */
  bytes: Vec<u8>
}

// H - unsigned short
// h - short
// I - unsigned int

impl Payload {
  pub fn pad_byte(&mut self) -> Result<u8> {
    if let Some(byte) = self.bytes.pop() {
      Ok(byte)
    } else {
      Err(anyhow!("You are out of bytes to pop"))
    }
  }

  pub fn pad_bytes(&mut self, num : usize) -> Result<()> {
    for _ in 0..num {
      self.pad_byte()
        .with_context(|| "Could not pad byte")?;
    }

    Ok(())
  }

  /**
   * Remove head bytes. Every payload contains 3 bytes for additional header
   */
  pub fn pop_head(&mut self) -> Result<bool> {
    self.pad_bytes(3)
      .with_context(|| "Could not pop 3 bytes header")?;

    Ok(true)
  }

  /**
   * Return unsigned short
   */
  pub fn pop_u16(&mut self) -> Result<u16> {
    let mut value_bytes : [u8; 2] = [0, 0];
    value_bytes[0] = self.pad_byte()?;
    value_bytes[1] = self.pad_byte()?;

    let value = u16::from_le_bytes(value_bytes);
    Ok(value)
  }


  /**
   * Pop unsigned short and checks if it is equal 1
   */
  pub fn pop_bool(&mut self) -> Result<bool> {
    let val = self.pop_u16()
      .with_context(|| "Could not read bool")?;

    Ok(val == 1)
  }

  /**
   * Return signed short
   */
  pub fn pop_i16(&mut self) -> Result<i16> {
    let mut value_bytes : [u8; 2] = [0, 0];
    value_bytes[0] = self.pad_byte()?;
    value_bytes[1] = self.pad_byte()?;

    let value = i16::from_le_bytes(value_bytes);
    Ok(value)
  }


  /**
   * Return unsigned int
   */
  pub fn pop_u32(&mut self) -> Result<u32> {
    let mut value_bytes : [u8; 4] = [0, 0, 0, 0];
    value_bytes[0] = self.pad_byte()?;
    value_bytes[1] = self.pad_byte()?;
    value_bytes[2] = self.pad_byte()?;
    value_bytes[3] = self.pad_byte()?;

    let value = u32::from_le_bytes(value_bytes);
    Ok(value)
  }

  /**
   * Return signed int
   */
  pub fn pop_i32(&mut self) -> Result<i32> {
    let mut value_bytes : [u8; 4] = [0, 0, 0, 0];
    value_bytes[0] = self.pad_byte()?;
    value_bytes[1] = self.pad_byte()?;
    value_bytes[2] = self.pad_byte()?;
    value_bytes[3] = self.pad_byte()?;

    let value = i32::from_le_bytes(value_bytes);
    Ok(value)
  }


  /**
   * Read utf string
   */
  pub fn pop_string_utf8(&mut self, characters: usize) -> Result<String> {
    let mut string_bytes : Vec<u8> = Vec::new();
    for _ in 0..characters {
      let byte = self.pad_byte()?;
      string_bytes.push(byte);
    }

    let string = String::from_utf8_lossy(&string_bytes);
    Ok(string.into_owned())
  }
}

impl From<Vec<u8>> for Payload {
  fn from(bytes: Vec<u8>) -> Self {
    let mut bytes = bytes.clone();
    bytes.reverse();

    Self {
      bytes
    }
  }
}

impl From<&[u8]> for Payload {
  fn from(bytes: &[u8]) -> Self {
    let mut vec : Vec<u8> = Vec::new();
    vec.extend_from_slice(bytes);
    vec.reverse();

    Self {
      bytes: vec
    }
  }
}

impl Debug for Payload {
  fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    let message = format!("Payload: {:?}", self.bytes.hex_dump());
    fmt.write_str(&message).unwrap();
    Ok(())
  }
}
