use crate::config::{Cli, Config};
use crate::error::Result;
use clap::Parser;
use flexi_logger::{colored_opt_format, opt_format, FileSpec, Logger};
use log::{error, info, warn};

use anyhow::Context;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use winapi::um::timeapi;

mod config;
mod error;
mod helper_funcs;
mod keyboard;

#[cfg(feature = "touch")]
mod touch;

#[cfg(feature = "jvs")]
mod jvs;

#[cfg(feature = "reader")]
mod card_reader;

fn main() {
    if let Err(e) = setup() {
        error!("Error: {}", e);
    }

    use std::process::Command;
    let _ = Command::new("cmd.exe").arg("/c").arg("pause").status();
}

fn setup() -> Result<()> {
    let mut cli = Cli::parse();

    // Set timer resolution to lower value possible. This is done for increasing reading speed of COM ports.
    unsafe {
        let result = timeapi::timeBeginPeriod(1);
        if result != 0 {
            warn!("Failed to set timer resolution to 1ms (error code: {}). This may affect COM port performance.", result);
        }
    }

    let log_level = cli.log_level.take().unwrap_or_else(|| "info".to_string());

    let mut logger = Logger::try_with_str(log_level)?.format(colored_opt_format);

    if cli.log_to_file {
        let file_spec = FileSpec::default().directory("./logs");
        logger = logger
            .format_for_files(opt_format)
            .log_to_file(file_spec)
            .duplicate_to_stderr(flexi_logger::Duplicate::All);
    }

    logger.start()?;

    let config = Config::init(&cli)?;

    let exit_sig = Arc::new(AtomicBool::new(false));

    let handles = init_handles(&config, &exit_sig)?;

    ctrlc::set_handler(move || {
        info!("Got CTRL+C, exiting...");
        exit_sig.store(true, Ordering::Release);
    })
    .context("Failed to setup CTRL+C handler")?;

    for handle in handles.into_iter() {
        handle.join().unwrap()?;
    }

    Ok(())
}

fn init_handles(cfg: &Config, exit_sig: &Arc<AtomicBool>) -> Result<Vec<JoinHandle<Result<()>>>> {
    let mut handles = Vec::with_capacity(4);

    #[cfg(feature = "touch")]
    if cfg.touch.enabled {
        if let Err(e) = touch::setup(&cfg.touch, &mut handles, exit_sig.clone()) {
            error!("Failed to initialize Touch module: {}", e);
            error!("Touch functionality will be disabled");
        }
    };

    #[cfg(feature = "jvs")]
    if cfg.jvs.enabled {
        if let Err(e) = jvs::setup(&cfg.jvs, &mut handles, exit_sig.clone()) {
            error!("Failed to initialize JVS module: {}", e);
            error!("JVS functionality will be disabled");
        }
    }

    #[cfg(feature = "reader")]
    if cfg.reader.enabled {
        if let Err(e) = card_reader::setup(&cfg.reader, &mut handles, exit_sig.clone()) {
            error!("Failed to initialize Card Reader module: {}", e);
            error!("Card Reader functionality will be disabled");
        }
    }

    Ok(handles)
}
