use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

const STORAGE_NAME: &str = ".dst_mods.toml";

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    pub steam_apps: PathBuf,
    pub save: PathBuf,
}

pub fn store(config: &Config) -> Result<()> {
    let file = std::fs::File::create(home::home_dir().unwrap().join(STORAGE_NAME))?;
    let mut file = BufWriter::new(file);

    let config_str = toml::to_string_pretty(config)?;

    writeln!(file, "{}", config_str)?;

    Ok(())
}

pub fn restore() -> Result<Config> {
    let config_str = std::fs::read_to_string(home::home_dir().unwrap().join(STORAGE_NAME))?;
    Ok(toml::from_str(&config_str)?)
}
