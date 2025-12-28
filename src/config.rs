use std::path::Path;

use crate::error;
use anyhow::Context;
use clap::{ArgAction, Parser};
use serde::{Deserialize, Serialize};

#[cfg(feature = "jvs")]
pub use jvs::*;
#[cfg(feature = "reader")]
pub use reader::*;
#[cfg(feature = "touch")]
pub use touch::*;

#[cfg(feature = "jvs")]
pub mod jvs;
#[cfg(feature = "reader")]
pub mod reader;
#[cfg(feature = "touch")]
pub mod touch;

#[derive(Parser, Deserialize, Serialize, Debug)]
#[clap(author = "robloxxa", version, about, long_about = None)]
/// Tool that allow playing Maimai DX on original Maimai Finale Cabinet
pub struct CLI {
    #[arg(long, short = 'l')]
    pub log_level: Option<String>,
    
    #[arg(long, default_value = "false", action=ArgAction::SetTrue)]
    pub log_to_file: bool,

    #[arg(long, short = 'p', default_value = "./config.toml")]
    #[serde(skip)]
    pub config_path: String,

    #[arg(long, short = 'c', default_value = "false", action=ArgAction::SetTrue)]
    #[serde(skip)]
    pub create_config: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    #[cfg(feature = "touch")]
    #[serde(default)]
    pub touch: Touch,

    #[cfg(feature = "jvs")]
    #[serde(default)]
    pub jvs: JVS,

    #[cfg(feature = "reader")]
    #[serde(default)]
    pub reader: Reader,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            #[cfg(feature = "touch")]
            touch: touch::Touch::default(),

            #[cfg(feature = "jvs")]
            jvs: jvs::JVS::default(),

            #[cfg(feature = "reader")]
            reader: reader::Reader::default(),
        }
    }
}

impl Config {
    pub fn init(cli: &CLI) -> Result<Self, error::Error> {
        if cli.create_config {
            let config = Self::default();
            config.save(&cli.config_path)?;

            Err(anyhow::anyhow!("first time creating config, exiting").into())
        } else {
            Self::load(&cli.config_path)
        }
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), error::Error> {
        let toml_str =
            toml::to_string_pretty(&self).context("failed to deserialize toml config")?;
        std::fs::write(path, toml_str)?;

        Ok(())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, error::Error> {
        let toml_str = std::fs::read_to_string(path)?;

        let config =
            toml::from_str::<Self>(&toml_str).context("failed to deserialize toml config")?;

        Ok(config)
    }
}
