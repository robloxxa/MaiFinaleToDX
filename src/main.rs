use crate::config::Config;
use anyhow::{Error, Result};
use clap::Parser;
use clap_serde_derive::ClapSerde;
use config::Settings;
use flexi_logger::{colored_opt_format, Logger};
use log::{error, info, warn};

use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use winapi::um::timeapi;

mod card_reader;
mod config;
mod helper_funcs;
mod jvs;
mod keyboard;
mod packets;
mod touch;

fn main() {
    let mut handles: Vec<JoinHandle<io::Result<()>>> = Vec::new();

    let running = Arc::new(AtomicBool::new(true));
    if !args.settings.disable_touch {
        match touch::spawn_thread(&args.settings, &running) {
            Ok((finale, deluxe)) => {
                handles.push(finale);
                handles.push(deluxe);
            }
            Err(err) => error!("Touchscreen initialization failed: {}", err),
        };
    } else {
        warn!("\"disable_touch\" was set to True. Touch features disabled")
    }

    if !args.settings.disable_jvs {
        match jvs::spawn_thread(&args, running.clone()) {
            Ok(jvs) => handles.push(jvs),
            Err(err) => error!("JVS initialization failed: {}", err),
        }
    } else {
        warn!("\"disable_jvs\" was set to True. JVS features disabled")
    }

    if !args.settings.disable_reader {
        match card_reader::spawn_thread(&args, running.clone()) {
            Ok(reader) => handles.push(reader),
            Err(err) => error!("Card reader initialization failed: {}", err),
        }
    } else {
        warn!("\"disable_reader\" was set to True. NFC reader proxy disabled")
    }

    ctrlc::set_handler(move || {
        info!("Exiting...");
        running.store(false, Ordering::Release);
    })
    .unwrap();

    for _ in 0..handles.len() {
        if let Err(e) = handles.pop().unwrap().join() {
            error!("Thread panicked, {:?}", e);
        }
    }
}

fn setup() -> Result<()> {
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

    Ok(())
}


fn setup_config() -> Result<Config> {
    let mut config = Config::parse();

    if config.create_config {
        let config_str = toml::to_string_pretty(&config)?;
        File::create(&config.config_path).and_then(|mut f| f.write_all(config_str.as_bytes()))?;
        info!("Config successfully created in {}", config.config_path);
    };

    match File::open(&config.config_path) {
        Ok(mut f) => {
            let mut data = String::new();
            f.read_to_string(&mut data)?;
            match toml::from_str::<<Config as ClapSerde>::Opt>(data.as_str()) {
                Ok(config) => Ok(Config::from(config).merge_clap()),
                Err(err) => Err(err.into()),
            }
        }
        Err(_) => {
            info!("No configuration file found");
            Ok(config)
        }
    }
}

fn setup_handlers(cfg: &Config) -> Result<()> {
    let (send, recv) = crossbeam_channel::bounded::<Error>(1);



    Ok(())
}
