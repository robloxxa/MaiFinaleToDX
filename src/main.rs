use crate::config::Config;
use ::clap;
use clap::{Parser};
use clap_serde_derive::ClapSerde;
use flexi_logger::{colored_opt_format, Logger};
use log::{warn};

use std::fs::File;
use std::io::{Read, Write};
use std::thread::JoinHandle;

mod card_reader;
mod config;
mod helper_funcs;
mod jvs;
mod keyboard;
mod touch;

fn main() {
    let args = Config::parse();

    if args.create_config {
        let mut file = File::create(args.config_path.clone())
            .expect("Couldn't create a config file");
        let config_str = toml::to_string_pretty(&args)
            .expect("Couldn't serialize struct");
        file.write(config_str.as_bytes())
            .expect("Failed to write a config file");
        println!("Config successfully created in {}", args.config_path);
        return
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
        Config::from(args)
    };
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    Logger::try_with_str(&config.log_level)
        .unwrap()
        .format(colored_opt_format)
        .start()
        .unwrap();
    if !config.settings.disable_touch {
        let (re2, alls) = touch::spawn_thread(&config.settings);
        handles.push(re2);
        handles.push(alls);
    } else {
        warn!("\"disable_touch\" was set to True. Touch features disabled")
    }

    if !config.settings.disable_jvs {
        handles.push(jvs::spawn_thread(&config))
    } else {
        warn!("\"disable_jvs\" was set to True. JVS features disabled")
    }

    if !config.settings.disable_reader {
        handles.push(card_reader::spawn_thread(&config));
    } else {
        warn!("\"disable_reader\" was set to True. NFC reader proxy disabled")
    }

    loop{}
}
