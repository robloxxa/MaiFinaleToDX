use crate::config::{Config, CLI};
use crate::error::Result;
use clap::Parser;
use flexi_logger::{colored_opt_format, Logger};
use log::{error, info};

use crate::helper_funcs::log_error;
use anyhow::Context;
use std::io;
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
    let cli = CLI::parse();

    // Set timer resolution to lower value possible. This is done for increasing reading speed of COM ports.
    unsafe {
        timeapi::timeBeginPeriod(1);
    }

    let logger = Logger::try_with_str("debug")?
        .format(colored_opt_format)
        .start()?;

    let config = Config::init(&cli)?;

    if let Some(level) = cli.log_level.as_ref() {
        logger.parse_new_spec(level)?;
    }

    let exit_sig = Arc::new(AtomicBool::new(false));

    let handles = init_handles(&config, &exit_sig)?;

    ctrlc::set_handler(move || {
        info!("Got CTRL+C, exiting...");
        exit_sig.store(true, Ordering::Relaxed);
    })
    .context("Failed to setup CTRL+C handler")?;

    for handle in handles.into_iter() {
        handle.join().unwrap()?;
    }

    Ok(())
}

fn init_handles(
    cfg: &Config,
    exit_sig: &Arc<AtomicBool>,
) -> Result<Vec<JoinHandle<io::Result<()>>>> {
    let mut handles = Vec::with_capacity(4);

    #[cfg(feature = "touch")]
    if cfg.touch.enabled {
        touch::setup(&cfg.touch, &mut handles, exit_sig.clone())
            .map_err(log_error)
            .ok();
    };

    #[cfg(feature = "jvs")]
    if cfg.jvs.enabled {
        jvs::init(&cfg.jvs, &mut handles, exit_sig.clone())
            .map_err(log_error)
            .ok();
    }

    #[cfg(feature = "reader")]
    if cfg.reader.enabled {
        card_reader::init(&cfg.reader, &mut handles, exit_sig.clone())
            .map_err(log_error)
            .ok();
    }

    Ok(handles)
}
