use crate::config::Config;
use crate::error::Result;
use clap::Parser;
use clap_serde_derive::ClapSerde;
use config::Settings;
use flexi_logger::{colored_opt_format, Logger};
use log::{debug, error, info, warn};

use crate::helper_funcs::log_error;
use anyhow::Context;
use anyhow::__private::kind::TraitKind;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use winapi::um::timeapi;

// mod card_reader;
mod card_reader;
mod config;
mod error;
mod helper_funcs;
mod jvs;
mod keyboard;
mod touch;

fn main() {
    // let mut handles: Vec<JoinHandle<io::Result<()>>> = Vec::new();

    // let running = Arc::new(AtomicBool::new(true));
    // if !args.settings.disable_touch {
    //     match touch::spawn_thread(&args.settings, &running) {
    //         Ok((finale, deluxe)) => {
    //             handles.push(finale);
    //             handles.push(deluxe);
    //         }
    //         Err(err) => error!("Touchscreen initialization failed: {}", err),
    //     };
    // } else {
    //     warn!("\"disable_touch\" was set to True. Touch features disabled")
    // }

    // if !args.settings.disable_jvs {
    //     match jvs::spawn_thread(&args, running.clone()) {
    //         Ok(jvs) => handles.push(jvs),
    //         Err(err) => error!("JVS initialization failed: {}", err),
    //     }
    // } else {
    //     warn!("\"disable_jvs\" was set to True. JVS features disabled")
    // }

    // if !args.settings.disable_reader {
    //     match card_reader::spawn_thread(&args, running.clone()) {
    //         Ok(reader) => handles.push(reader),
    //         Err(err) => error!("Card reader initialization failed: {}", err),
    //     }
    // } else {
    //     warn!("\"disable_reader\" was set to True. NFC reader proxy disabled")
    // }

    // ctrlc::set_handler(move || {
    //     info!("Exiting...");
    //     running.store(false, Ordering::Release);
    // })
    // .unwrap();

    // for _ in 0..handles.len() {
    //     if let Err(e) = handles.pop().unwrap().join() {
    //         error!("Thread panicked, {:?}", e);
    //     }
    // }
    if let Err(e) = setup() {
        error!("Error: {}", e);
    }

    use std::process::Command;
    let _ = Command::new("cmd.exe").arg("/c").arg("pause").status();
}

fn setup() -> Result<()> {
    // Set timer resolution to lower value possible. This is done for increasing reading speed of COM ports.
    unsafe {
        timeapi::timeBeginPeriod(1);
    }

    let logger = Logger::try_with_str("info")?
        .format(colored_opt_format)
        .start()?;

    let config = setup_config()?;

    if config.log_level != "info" {
        logger.parse_new_spec(&config.log_level)?;
    }

    let exit_sig = Arc::new(AtomicBool::new(false));

    let handles = setup_handles(&config.settings, &exit_sig)?;

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

fn setup_config() -> Result<Config> {
    let config = Config::parse();

    if config.create_config {
        let config_str = toml::to_string_pretty(&config).context("Failed to read config")?;
        File::create(&config.config_path).and_then(|mut f| f.write_all(config_str.as_bytes()))?;
        info!("Config successfully created in {}", config.config_path);
    };

    match File::open(&config.config_path) {
        Ok(mut f) => {
            let mut data = String::new();
            f.read_to_string(&mut data)?;
            match toml::from_str::<<Config as ClapSerde>::Opt>(data.as_str()) {
                Ok(config) => Ok(Config::from(config).merge_clap()),
                Err(err) => Err(anyhow::Error::from(err).into()),
            }
        }
        Err(_) => {
            info!("No configuration file found");
            Ok(config)
        }
    }
}

fn setup_handles(
    cfg: &Settings,
    exit_sig: &Arc<AtomicBool>,
) -> Result<Vec<JoinHandle<io::Result<()>>>> {
    let mut handles = Vec::with_capacity(4);

    if cfg.touch {
        touch::setup(cfg, &mut handles, exit_sig.clone())
            .map_err(log_error)
            .ok();
    };

    if cfg.jvs {
        jvs::setup(cfg, &mut handles, exit_sig.clone())
            .map_err(log_error)
            .ok();
    }

    if cfg.reader {
        card_reader::setup(cfg, &mut handles, exit_sig.clone())
            .map_err(log_error)
            .ok();
    }

    Ok(handles)
}
