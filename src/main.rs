use std::convert::Into;
use std::process::ExitCode;

use clap::{Arg, ArgAction, Command};
use configparser::ini::Ini;
use env_logger::Builder;
use log::{debug, error, LevelFilter, warn};

use crate::errors::DaemonError;
use crate::server::Server;

mod server;
mod errors;

struct DaemonConfig {
    log_level: LevelFilter,
    config_path: String,
    interface: String,
    port: u16,
    sleep_cmd: String,
    dry_run: bool
}

fn main() -> ExitCode {
    let config = match parse_config() {
        Ok(c) => c,
        Err(error) => {
            env_logger::init();
            error!("Failed to read application config: {}", error);
            return ExitCode::FAILURE;
        }
    };

    Builder::from_default_env().filter_level(config.log_level).init();

    if config.dry_run {
        warn!("Starting application in DRY-RUN mode with config {}", config.config_path);
    } else {
        debug!("Starting application with config {}", config.config_path);
    }

    let server = Server::new(config.interface, config.port, config.sleep_cmd);
    return match server.run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            error!("Failed to start application: {}", error);
            return ExitCode::FAILURE;
        }
    };
}

fn cli() -> Command {
    return Command::new("Sleep-On-LAN daemon")
        .about(
            "Triggers system sleep on magic package receival.\n\n\
             Application works with the exact same magic packet format as used for Wake-On-LAN, \
             with the only difference is that the MAC address has to be written in reverse order."
        )
        .args([
            Arg::new("config")
                .short('c').long("config")
                .help("Config file to use. Default: /etc/sleep-on-lan.conf"),
            Arg::new("verbose")
                .short('v').long("verbose")
                .action(ArgAction::SetTrue)
                .help("Enable verbose logging"),
            Arg::new("dry-run")
                .long("dry-run")
                .action(ArgAction::SetTrue)
                .help("Start in dry-run mode, where receival of magic package would not trigger actual server sleep")
        ]);
}

fn parse_config() -> Result<DaemonConfig, DaemonError> {
    let cli_args = cli().get_matches();

    // read CLI options
    let default_config_path = &String::from("/etc/sleep-on-lan.conf");
    let config_path = cli_args.get_one::<String>("config").unwrap_or(default_config_path);
    let dry_run = cli_args.get_flag("dry-run");
    let log_level = if cli_args.get_flag("verbose") {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    // read config
    let mut config = Ini::new();
    let _ = config.load(config_path.clone()).map_err(
        |source| DaemonError::ConfigParseError { config_path: config_path.clone(), source }
    )?;

    let interface = config.get("main", "interface").unwrap_or("eth0".into());
    let port = config.getuint("main", "port").unwrap().unwrap_or(9) as u16;
    let sleep_cmd = if dry_run {
        String::from("echo '[DRY RUN] Shutting down...'")
    } else {
        let default_cmd = String::from("systemctl hibernate");
        config.get("main", "sleep-cmd").unwrap_or(default_cmd)
    };

    return Ok(DaemonConfig {
        log_level,
        config_path: config_path.clone(),
        interface,
        port,
        sleep_cmd: sleep_cmd.into(),
        dry_run
    });
}
