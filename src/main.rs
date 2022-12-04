mod config;
mod database;
mod lua_ctx;
mod r#override;
mod steam_apps;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use config::*;
use database::{DataBase, Mod};
use r#override::*;
use std::{collections::HashMap, path::PathBuf};
use steam_apps::*;

#[derive(Args)]
pub struct List {
    #[arg(long)]
    client: bool,
    #[arg(long)]
    all: bool,
    #[arg(long)]
    diff: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    List(List),
    Sync {
        ids: Vec<usize>,
    },
    Remove {
        ids: Vec<usize>,
        #[arg(long)]
        client: bool,
    },
    Option {
        id: usize,
        args: Option<String>,
        #[arg(long)]
        all: bool,
    },
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(long)]
    steamapps: Option<PathBuf>,
    #[clap(long)]
    save: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut config = restore().unwrap_or_else(|_| Default::default());
    if let Some(steam_apps) = cli.steamapps {
        config.steam_apps = steam_apps;
    }
    if let Some(save) = cli.save {
        config.save = save;
    }
    store(&config)?;

    let steam_apps = SteamApps::load(&config.steam_apps)?;

    match cli.command {
        Commands::List(args) => {
            steam_apps.list(&args);
        }
        Commands::Sync { ids } => {
            let mut modoverrides = Override::load(&config.save)?;
            for id in ids {
                steam_apps.sync(id)?;
                modoverrides.insert(id, Default::default());
            }
            modoverrides.sink()?;
        }
        Commands::Remove { ids, client } => {
            let mut modoverrides = Override::load(&config.save)?;
            for id in ids {
                steam_apps.remove(id, client);
                modoverrides.remove(id);
            }
            modoverrides.sink()?;
        }
        Commands::Option { id, args, all } => {
            if let Some(args) = args {
                let mut items = HashMap::new();
                let pairs = args.split(',');
                for pair in pairs {
                    let mut kv = pair.split('=');
                    let k = kv.next().expect("invalid option");
                    let v = kv.next().expect("invalid option");

                    items.insert(k.to_owned(), v.to_owned());
                }
                let mut modoverrides = Override::load(&config.save)?;
                modoverrides.insert(id, items);
                modoverrides.sink()?;
            }
            if all {
                steam_apps.read_options(id)?;
            } else {
                let modoverrides = Override::load(&config.save)?;
                modoverrides.list(id);
            }
        }
    }

    Ok(())
}
