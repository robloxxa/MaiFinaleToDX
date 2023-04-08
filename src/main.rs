use crate::config::Config;
use ::clap;
use clap::Parser;
use clap_serde_derive::ClapSerde;
use flexi_logger::{colored_opt_format, opt_format, Logger};
use log::{debug, error, info, warn};
use std::fs;
use std::fs::File;
use std::io::Read;

mod card_reader;
mod config;
mod helper_funcs;
mod jvs;
mod keyboard;
mod touch;

fn main() {
    let args = Config::parse();
    let config = if let Ok(mut f) = File::open(&args.config_path) {
        let mut data = String::new();
        f.read_to_string(&mut data).expect("Unable to parse a file");
        match toml::from_str::<Config>(data.as_str()) {
            Ok(config) => Config::from(config).merge_clap(),
            Err(err) => panic!("Error in configuration file:\n{}", err),
        }
    } else {
        println!("No configuration file found");
        Config::from(args)
    };
    dbg!(config);
    // Logger::try_with_str(&config.log_level)
    //     .unwrap()
    //     .format(colored_opt_format)
    //     .start()
    //     .unwrap();
    // if !config.disable_touch {
    //     let (re2, alls) = touch::spawn_thread(&args);
    //     re2.join().unwrap();
    //     alls.join().unwrap();
    // } else {
    //     warn!("\"disable_touch\" was set to True. Touch features disabled")
    // }
    //
    // if !config.disable_jvs {
    // } else {
    //     warn!("\"disable_jvs\" was set to True. JVS features disabled")
    // }
}
