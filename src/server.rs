use std::net::{SocketAddr, UdpSocket};
use std::process::{Command, ExitStatus};

use log::{debug, info, warn};
use mac_address::{MacAddress, mac_address_by_name};

use crate::config::DaemonConfig;
use crate::errors::DaemonError;
use crate::errors::DaemonError::{
    MacReadError, NoMacAddress, SleepError, SocketBindError, SocketReadError,
};

// 6 * 0xFF and 16 * MAC
const MAGIC_PACKAGE_SIZE: usize = 6 + 16 * 6;

pub struct Server {
    interface: String,
    port: u16,
    sleep_cmd: String,
}

impl Server {
    pub fn new(config: DaemonConfig) -> Server {
        Server {
            interface: config.interface,
            port: config.port,
            sleep_cmd: config.sleep_cmd,
        }
    }

    pub fn run(&self) -> Result<(), DaemonError> {
        let address = SocketAddr::from(([0, 0, 0, 0], self.port));
        let socket =
            UdpSocket::bind(address).map_err(|source| SocketBindError { address, source })?;
        debug!("Successfully bound to address {}", address);

        let mut buffer = [0; MAGIC_PACKAGE_SIZE];
        let device_mac = self.get_interface_mac()?;

        loop {
            info!(
                "Waiting for magic package for MAC {} on address {}",
                device_mac, address
            );
            let (read_count, sender) = socket
                .recv_from(&mut buffer)
                .map_err(|source| SocketReadError { source })?;

            let filled_buffer = &mut buffer[..read_count];
            if Self::is_magic_package(device_mac, filled_buffer) {
                info!(
                    "Magic package received from {} - initiating machine sleep...",
                    sender.ip()
                );
                self.initiate_sleep()?;
            } else {
                warn!(
                    "Invalid magic package received {:02x?} - skipping...",
                    filled_buffer
                );
            }
        }
    }

    fn initiate_sleep(&self) -> Result<ExitStatus, DaemonError> {
        Command::new("sh")
            .arg("-c")
            .arg(&self.sleep_cmd)
            .spawn()
            .and_then(|mut child| child.wait()) // wait for sleep command to return
            .map_err(|source| SleepError {
                command: self.sleep_cmd.clone(),
                source,
            })
    }

    #[allow(clippy::needless_range_loop)] // index-based solution is simpler to reason about
    fn is_magic_package(mac_address: MacAddress, package: &[u8]) -> bool {
        // check length
        if package.len() != MAGIC_PACKAGE_SIZE {
            return false;
        }

        // check first 6 bytes
        for i in 0..6 {
            if package[i] != 0xff {
                return false;
            }
        }

        // check presence of reversed MAC
        let mut mac_bytes = mac_address.bytes();
        mac_bytes.reverse();

        for i in 6..MAGIC_PACKAGE_SIZE {
            let mac_idx = (i - 6) % 6;
            if package[i] != mac_bytes[mac_idx] {
                return false;
            }
        }

        true
    }

    fn get_interface_mac(&self) -> Result<MacAddress, DaemonError> {
        let iface = self.interface.clone();
        match mac_address_by_name(&iface) {
            Ok(Some(address)) => Ok(address),
            Ok(None) => Err(NoMacAddress { iface }),
            Err(source) => Err(MacReadError { iface, source }),
        }
    }
}

#[cfg(test)]
#[rustfmt::skip]  // keep consistent alignment for test packages
mod tests {
    use std::error::Error;
    use std::str::FromStr;

    use mac_address::MacAddress;

    use super::*;

    #[test]
    fn is_magic_package_for_valid_package() -> Result<(), Box<dyn Error>> {
        let test_mac = MacAddress::from_str("11:22:33:44:55:66")?;
        // using reversed MAC
        let test_package: &[u8] = &[
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11
        ];

        let result = Server::is_magic_package(test_mac, test_package);

        assert!(result);
        Ok(())
    }

    #[test]
    fn is_magic_package_for_short_package() -> Result<(), Box<dyn Error>> {
        let test_mac = MacAddress::from_str("11:22:33:44:55:66")?;
        let test_package: &[u8] = &[11, 22, 33];

        let result = Server::is_magic_package(test_mac, test_package);

        assert_eq!(result, false);
        Ok(())
    }

    #[test]
    fn is_magic_package_for_invalid_header() -> Result<(), Box<dyn Error>> {
        let test_mac = MacAddress::from_str("11:22:33:44:55:66")?;
        let test_package: &[u8] = &[
            0x11, 0xff, 0xff, 0xff, 0xff, 0xff,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11
        ];

        let result = Server::is_magic_package(test_mac, test_package);

        assert_eq!(result, false);
        Ok(())
    }

    #[test]
    fn is_magic_package_for_straight_package() -> Result<(), Box<dyn Error>> {
        let test_mac = MacAddress::from_str("11:22:33:44:55:66")?;
        let test_package: &[u8] = &[
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
        ];

        let result = Server::is_magic_package(test_mac, test_package);

        assert_eq!(result, false);
        Ok(())
    }
}
