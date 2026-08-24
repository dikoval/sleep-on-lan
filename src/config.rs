pub struct DaemonConfig {
    pub(crate) interface: String,
    pub(crate) port: u16,
    pub(crate) sleep_cmd: String,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        DaemonConfig {
            interface: "wlan0".to_string(), //"eth0".to_string(),
            port: 10009, //9,
            sleep_cmd: "systemctl hibernate".to_string(),
        }
    }
}
