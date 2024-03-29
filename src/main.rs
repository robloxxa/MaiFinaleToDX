use crate::config::Config;
use clap::Parser;
use clap_serde_derive::ClapSerde;
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
    unsafe {
        timeapi::timeBeginPeriod(1);
    }

    let args = Config::parse();
    if args.create_config {
        let mut file =
            File::create(args.config_path.clone()).expect("Couldn't create a config file");
        let config_str = toml::to_string_pretty(&args).expect("Couldn't serialize struct");
        file.write_all(config_str.as_bytes())
            .expect("Failed to write a config file");
        println!("Config successfully created in {}", args.config_path);
        return;
    }
    let config = if let Ok(mut f) = File::open(&args.config_path) {
        let mut data = String::new();
        f.read_to_string(&mut data).expect("Unable to parse a file");
        match toml::from_str::<<Config as ClapSerde>::Opt>(data.as_str()) {
            Ok(config) => Config::from(config).merge_clap(),
            Err(err) => panic!("Error in configuration file:\n{}", err),
        }
    } else {
        println!("No configuration file found");
        args
    };
    let mut handles: Vec<JoinHandle<io::Result<()>>> = Vec::new();
    Logger::try_with_str(&config.log_level)
        .unwrap()
        .format(colored_opt_format)
        .start()
        .unwrap();

    let running = Arc::new(AtomicBool::new(true));
    if !config.settings.disable_touch {
        match touch::spawn_thread(&config.settings, &running) {
            Ok((finale, deluxe)) => {
                handles.push(finale);
                handles.push(deluxe);
            }
            Err(err) => error!("Touchscreen initialization failed: {}", err),
        };
    } else {
        warn!("\"disable_touch\" was set to True. Touch features disabled")
    }

    if !config.settings.disable_jvs {
        match jvs::spawn_thread(&config, running.clone()) {
            Ok(jvs) => handles.push(jvs),
            Err(err) => error!("JVS initialization failed: {}", err),
        }
    } else {
        warn!("\"disable_jvs\" was set to True. JVS features disabled")
    }

    if !config.settings.disable_reader {
        match card_reader::spawn_thread(&config, running.clone()) {
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
