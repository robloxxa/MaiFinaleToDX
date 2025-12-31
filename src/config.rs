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

    #[arg(long, default_value = "true", action=ArgAction::SetTrue)]
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
        let mut doc = toml_edit::ser::to_document(self)?;

        let to_block_table = |item: &mut toml_edit::Item| {
            if let Some(inline) = item.as_inline_table_mut() {
                let table = inline.clone().into_table();
                *item = toml_edit::Item::Table(table);
            }
        };

        // Функция-помощник: делает таблицу ИНЛАЙНОВОЙ (в одну строку {..})
        let to_inline_table = |item: &mut toml_edit::Item| {
            if let Some(table) = item.as_table_mut() {
                let mut inline = table.clone().into_inline_table();
                inline.fmt(); // убираем лишние пробелы внутри
                *item = toml_edit::Item::Value(toml_edit::Value::InlineTable(inline));
            }
        };
        
        doc.entry_format(key)
        
        
        to_block_table(doc.get_mut("touch").unwrap_or(&mut toml_edit::Item::None));
        to_block_table(doc.get_mut("jvs").unwrap_or(&mut toml_edit::Item::None));
        to_block_table(doc.get_mut("reader").unwrap_or(&mut toml_edit::Item::None));

        if let Some(touch) = doc.get_mut("touch") {
            to_block_table(touch);
            // Поля e1..c2 для P1 (так как они flatten, они лежат прямо в touch)
            let keys = [
                "e1", "e2", "e3", "e4", "e5", "e6", "e7", "e8", "d1", "d2", "d3", "d4", "d5", "d6",
                "d7", "d8", "c1", "c2",
            ];

            if let Some(p1_map) = touch.get_mut("p1_dx_touch_mapping")
                .and_then(|t| {
                    to_block_table(t);
                    Some(t)
                })
                .and_then(|m| m.as_table_mut())
            {
                for key in keys {
                    if let Some(item) = p1_map.get_mut(key) {
                        to_inline_table(item);
                    }
                }
            }

            // Поля для P2 (они в подтаблице)
            if let Some(p2_map) = touch
                .get_mut("p2_dx_touch_mapping")
                .and_then(|m| m.as_table_mut())
            {
                for key in keys {
                    if let Some(item) = p2_map.get_mut(key) {
                        to_inline_table(item);
                    }
                }
            }

            // Сделаем пороги (threshold) тоже красивыми блоками, если они вдруг инлайновые
            to_block_table(
                touch
                    .get_mut("p1_threshold")
                    .unwrap_or(&mut toml_edit::Item::None),
            );
            to_block_table(
                touch
                    .get_mut("p2_threshold")
                    .unwrap_or(&mut toml_edit::Item::None),
            );
        }

        // --- ШАГ 3: Финальный штрих ---
        doc.fmt(); // Расставит отступы и переносы между секциями
        std::fs::write(path, doc.to_string())?;

        Ok(())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, error::Error> {
        let toml_str = std::fs::read_to_string(path)?;

        let config =
            toml::from_str::<Self>(&toml_str).context("failed to deserialize toml config")?;

        Ok(config)
    }
}
