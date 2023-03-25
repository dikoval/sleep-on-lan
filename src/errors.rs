use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::net::SocketAddr;

use mac_address::MacAddressError;

// exhaustive list of all possible daemon errors
#[derive(Debug)]
pub enum DaemonError {
    ConfigParseError {
        config_path: String,
        source: String
    },

    SocketBindError {
        address: SocketAddr,
        source: std::io::Error
    },

    SocketReadError {
        source: std::io::Error
    },

    NoMacAddress {
        iface: String
    },

    MacReadError {
        iface: String,
        source: MacAddressError
    },

    SleepError {
        command: String,
        source: std::io::Error
    }
}

// user-friendly error description
impl Display for DaemonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigParseError {config_path, source} =>
                write!(f, "Failed to read config file {}: {}", config_path, source),

            Self::SocketBindError {address, source} =>
                write!(f, "Failed to bind to address {}: {}", address, source),

            Self::SocketReadError {source} =>
                write!(f, "Failed to read magic package from socket {}", source),

            Self::NoMacAddress {iface} =>
                write!(f, "No MAC address found for interface {}", iface),

            Self::MacReadError {iface, source} =>
                write!(f, "Failed to determine MAC address of interface {}: {}", iface, source),

            Self::SleepError {command, source} =>
                write!(f, "Failure during sleep command '{}' invocation: {}", command, source)
        }
    }
}

// make our DaemonError an actual error type
impl Error for DaemonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SocketBindError { source, .. } => Some(source),
            Self::SocketReadError { source, .. } => Some(source),
            Self::MacReadError { source, .. } => Some(source),
            Self::SleepError { source, .. } => Some(source),
            _ => None
        }
    }
}