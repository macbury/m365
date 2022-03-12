![Scooter](./doc/logo.jpeg)
# About

![MIT license](https://img.shields.io/github/license/macbury/m365)
![Crates.io version](https://img.shields.io/crates/v/m365)

Get info about your Xiaomi Mijia 365 Scooter via BLE (Bluetooth Low Energy). This is using [btleplug](https://crates.io/crates/btleplug) which is an async Rust BLE library, supporting Windows 10, macOS, Linux, and iOS. I did test this only on my linux laptop and It should in theory work with other systems.

# How to use
Each example shows you how to interact with your scooter. Before you can read any information, you need to get auth token.

### Supported scooters:
This protocol is used in m365, mi-lite-1-s, mi-pro, mi-pro2 and mi-pro3.

### Note
Registering / pairing with devices unpairs the device from all other apps! If you want to use your device with other apps after pairing, either reinstall or remove / re-add the device inside the app.

## Find MAC address for scooter
This package includes the example to scan and list nearby M365 Scooters. Simply execute the package as such:

```bash
$ cargo run --example scanner
```
```
2022-03-01T20:24:52.433166Z DEBUG m365::scanner: Starting scanning for new devices
2022-03-01T20:24:52.445873Z DEBUG m365::scanner: Watching for events in background
2022-03-01T20:25:25.786072Z  INFO scanner: Found scooter nearby: MIScooter7353 with mac: D5:01:45:37:ED:FD
```

## Register

To get auth token from scooter run example register and pass mac address of your scooter. Registration token will be persisted as file `.mi-token`
```bash
$ cargo run --example register D5:01:45:37:ED:FD
```

## Login

You can check how you can login and read serial number using this example

```bash
$ cargo run --example login D5:01:45:37:ED:FD
```

## About

Read all information from scooter

```bash
$ cargo run --example about D5:01:45:37:ED:FD
```

```
2022-03-12T18:35:47.751207Z  INFO about: Searching scooter with address: D5:01:45:37:ED:FD
2022-03-12T18:35:47.769772Z  INFO m365::scanner: Found your scooter
2022-03-12T18:36:02.811852Z  INFO m365::login: Validating did
2022-03-12T18:36:02.930741Z  INFO m365::login: Logged in!
2022-03-12T18:36:03.350317Z  INFO about: Logged in with success, reading data...
2022-03-12T18:36:03.410690Z  INFO about:   Battery info: BatteryInfo { capacity: 7392, percent: 63, current: 0.01, voltage: 36.74, temperature_1: 44, temperature_2: 44 }
2022-03-12T18:36:03.502199Z  INFO about:   Battery cells (V): [36.71, 36.73, 36.71, 36.75, 36.74, 36.76, 36.78, 36.78, 36.8, 36.77]
2022-03-12T18:36:03.561323Z  INFO about:   Serial number 26354/00467353
2022-03-12T18:36:03.652204Z  INFO about:   Motor info: MotorInfo { battery_percent: 63, speed_kmh: 0, speed_average_kmh: 0, total_distance_m: 1306083, trip_distance_m: 0, uptime: 260s, frame_temperature: 24.0 }
2022-03-12T18:36:03.710897Z  INFO about:   Supplementary info SupplementaryInfo { kers: Weak, is_cruise: false, tail_light: Off }
2022-03-12T18:36:03.771259Z  INFO about:   General info GeneralInfo { serial: "26354/00467", pin: "353000", version: "00" }
2022-03-12T18:36:03.830117Z  INFO about:   Distance left 28.35 km
2022-03-12T18:36:03.890595Z  INFO about:   Trip distance 0 km
2022-03-12T18:36:03.951109Z  INFO about:   Current Speed 0 km/h
2022-03-12T18:36:04.011104Z  INFO about:   Cruise enabled: false
2022-03-12T18:36:04.070594Z  INFO about:   Tail light enabled: Off
2022-03-12T18:36:04.130628Z  INFO about:   Battery 36.74 V
2022-03-12T18:36:04.221106Z  INFO about:           0 A
2022-03-12T18:36:04.311157Z  INFO about:           63 %
2022-03-12T18:36:04.370719Z  INFO about:           BatteryInfo { capacity: 7392, percent: 63, current: 0.0, voltage: 36.74, temperature_1: 44, temperature_2: 44 }
```

## Settings

You can check how to change tail light and cruise mode with this example

```bash
$ cargo run --example settings D5:01:45:37:ED:FD
```

# License
See LICENSE.md

# Disclaimer
I'm in no way affiliated with Xiaomi or any of their subsidiaries and products. This code has been provided for research purposes only.

# References
* https://github.com/danielkucera/mi-standardauth
* https://learning-rust.github.io/docs/a4.cargo,crates_and_basic_project_structure.html
* https://github.com/CamiAlfa/M365-BLE-PROTOCOL
* https://github.com/AntonHakansson/m365py
* https://github.com/michaljach/m365-info/blob/master/M365Info.swift
* https://learn.adafruit.com/introduction-to-bluetooth-low-energy/gatt
* https://github.com/drogue-iot/burrboard/blob/9e10c87a596fe29055e2a302c2753963bcc545cd/gatt-client/src/board.rs
* https://github.com/deviceplug/btleplug
* https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md#2-if-t-static-then-t-must-be-valid-for-the-entire-program
* https://github.com/cheetahdotcat/m365-mi
* https://github.com/dnandha/miauth/tree/main/lib/java
* https://github.com/dnandha/miauth
* https://pypi.org/project/miauth/#files
* https://github.com/cheetahdotcat/m365-mi/blob/master/mim365mi/m365scooter.py
* https://github.com/stepanbenes/doorkeeper/blob/a70c08ad738fe4390e374c925e779fc6b9e67433/src/main.rs
* https://github.com/tokio-rs/tokio/discussions/3362
* https://depth-first.com/articles/2020/06/22/returning-rust-iterators/
* https://github.com/Emeryth/ReadM365/blob/master/readM365.py
* https://github.com/heardrwt/Xiaomi-M365-Display/blob/5414aa9ef6a7d6d76c8b88deedc90e148b1812f4/ninebot_module.c#L175
