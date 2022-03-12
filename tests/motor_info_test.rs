use hex_literal::hex;

use std::time::Duration;
use m365::{
  Payload,
  MotorInfo,
  BatteryInfo
};

#[test]
fn it_transform_payload_into_motor_info() {
  let bytes = hex!("2301b00000000000080000400000000000e3ed130000005800fa000000000000000000676598f0");
  let payload = Payload::from(&bytes[0..]);
  let motor_info = MotorInfo::try_from(payload).unwrap();

  assert_eq!(motor_info.battery_percent, 64);
  assert_eq!(motor_info.speed_kmh, 0.0);
  assert_eq!(motor_info.speed_average_kmh, 0.0);
  assert_eq!(motor_info.total_distance_m, 1306083);
  assert_eq!(motor_info.trip_distance_m, 0);
  assert_eq!(motor_info.uptime, Duration::from_secs(88));
  assert_eq!(motor_info.frame_temperature, 25.0);
}

#[test]
fn it_transform_payload_into_battery_info() {
  let bytes = hex!("250131f91c3f0001005c0e2d2d1178f518");
  let payload = Payload::from(&bytes[0..]);
  let battery = BatteryInfo::try_from(payload).unwrap();

  assert_eq!(battery.capacity, 7417);
  assert_eq!(battery.percent, 63);
  assert_eq!(battery.current, 0.01);
  assert_eq!(battery.voltage, 36.76);
  assert_eq!(battery.temperature_1, 45);
  assert_eq!(battery.temperature_2, 45);
}
