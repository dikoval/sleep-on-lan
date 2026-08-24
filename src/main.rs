use std::process::ExitCode;

use clap::{Arg, ArgAction, Command};
use configparser::ini::Ini;
use log::{LevelFilter, SetLoggerError, debug, error, warn};
use systemd_journal_logger::{JournalLog, connected_to_journal};

use crate::config::DaemonConfig;
use crate::errors::DaemonError;
use crate::server::Server;

mod config;
mod errors;
mod server;

fn main() -> ExitCode {
    // read CLI options
    let cli_args = cli().get_matches();

    let config_path = cli_args.get_one::<String>("config");
    let dry_run = cli_args.get_flag("dry-run");
    let log_level = if cli_args.get_flag("verbose") {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    // init logging
    if let Err(e) = init_logging(log_level) {
        eprint!("Failed to init application logging: {}", e);
        return ExitCode::FAILURE;
    }

    return match run_daemon(config_path, dry_run) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            error!("Failed to start application: {}", error);
            return ExitCode::FAILURE;
        }
    };
}

fn run_daemon(config_path: Option<&String>, dry_run: bool) -> Result<(), DaemonError> {
    // read config
    let mut config = match config_path {
        Some(path) => read_config_file(path)?,
        None => {
            let default_config_path = "/etc/sleep-on-lan.conf";
            match std::fs::exists(default_config_path) {
                Ok(true) => read_config_file(default_config_path)?,
                _ => {
                    debug!(
                        "Config file not found - starting application with default built-in configuration..."
                    );
                    DaemonConfig::default()
                }
            }
        }
    };

    if dry_run {
        warn!("Starting application in DRY-RUN mode!");
        config.sleep_cmd = String::from("echo '[DRY RUN] Shutting down...'");
    }

    let server = Server::new(config);
    server.run()
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

fn init_logging(log_level: LevelFilter) -> Result<(), SetLoggerError> {
    if connected_to_journal() {
        JournalLog::new()
            .unwrap()
            .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
            .install()?;

        log::set_max_level(log_level);
    } else {
        env_logger::builder().filter_level(log_level).try_init()?;
    }

    Ok(())
}

fn read_config_file(config_path: &str) -> Result<DaemonConfig, DaemonError> {
    // parse ini file
    let mut config = Ini::new();
    let _ = config.load(config_path).map_err(
        |source| DaemonError::ConfigParseError { config_path: config_path.to_string(), source }
    )?;

    // parse config
    let interface = config
        .get("main", "interface")
        .unwrap_or("eth0".to_string());
    let port = config
        .getuint("main", "port")
        .map_err(|source| DaemonError::ConfigParseError {
            config_path: config_path.to_string(),
            source,
        })?
        .unwrap_or(9) as u16;
    let sleep_cmd = config
        .get("main", "sleep-cmd")
        .unwrap_or("systemctl hibernate".to_string());

    debug!("Using config file '{}' for application", config_path);

    Ok(DaemonConfig {
        interface,
        port,
        sleep_cmd,
    })
}
