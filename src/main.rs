mod config;
mod database;
mod lua_ctx;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use config::*;
use database::{DataBase, Mod};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

struct SteamApps {
    client: [PathBuf; 2],
    server: PathBuf,
    client_list: HashMap<usize, Mod>,
    server_list: HashMap<usize, Mod>,
}

impl SteamApps {
    fn load(path: &Path) -> Result<Self> {
        let server = path.join("common/Don't Starve Together Dedicated Server/mods");
        let client = [
            path.join("common/Don't Starve Together/mods"),
            path.join("workshop/content/322330"),
        ];

        let server_list = server.list()?.map(|x| (x.id, x)).collect();
        let client_list = client[0]
            .list()?
            .chain(client[1].list()?)
            .map(|x| (x.id, x))
            .collect();

        Ok(Self {
            client,
            server,
            server_list,
            client_list,
        })
    }
}

#[derive(Args)]
struct List {
    #[arg(long)]
    client: bool,
    #[arg(long)]
    all: bool,
    #[arg(long)]
    diff: bool,
}

#[derive(Subcommand)]
enum Commands {
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
    },
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(long)]
    steamapps: Option<PathBuf>,
    #[clap(long)]
    save_path: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut config = restore().unwrap_or_else(|_| Default::default());
    if let Some(steam_apps) = cli.steamapps {
        config.steam_apps = steam_apps;
    }
    store(&config)?;

    let steam_apps = SteamApps::load(&config.steam_apps)?;

    match cli.command {
        Commands::List(args) => {
            let list = if args.client {
                &steam_apps.client_list
            } else {
                &steam_apps.server_list
            };

            println!(
                "{} {} mods:",
                if args.diff { "diff" } else { "list" },
                if args.client { "client" } else { "server" }
            );
            if args.diff {
                let diff_list = if args.client {
                    &steam_apps.server_list
                } else {
                    &steam_apps.client_list
                };

                for elem in list.values().filter(|elem| args.all || !elem.client_only) {
                    if !diff_list.contains_key(&elem.id) {
                        println!("[+] {:<10} => {}", elem.id, elem.name.trim());
                        println!("\t\t{:?}", elem.path);
                    }
                }
                for elem in diff_list
                    .values()
                    .filter(|elem| args.all || !elem.client_only)
                {
                    if !list.contains_key(&elem.id) {
                        println!("[-] {:<10} => {}", elem.id, elem.name.trim());
                        println!("\t\t{:?}", elem.path);
                    }
                }
            } else {
                for elem in list.values().filter(|elem| args.all || !elem.client_only) {
                    println!("{:<10} => {}", elem.id, elem.name.trim());
                    println!("\t\t{:?}", elem.path);
                }
            }
        }
        Commands::Sync { ids } => {
            for id in ids {
                if let Some(x) = steam_apps.client_list.get(&id) {
                    steam_apps.server.insert(x)?;
                } else {
                    eprintln!("WARN! {} not found", id);
                }
            }
        }
        Commands::Remove { ids, client } => {
            for id in ids {
                if client {
                    for database in &steam_apps.client {
                        database.remove(id);
                    }
                } else {
                    steam_apps.server.remove(id);
                }
            }
        }
        Commands::Option { id } => {
            let elem = if let Some(elem) = steam_apps.client_list.get(&id) {
                elem
            } else if let Some(elem) = steam_apps.server_list.get(&id) {
                elem
            } else {
                eprintln!("WARN! {} not found", id);
                return Ok(());
            };
            println!("{:<10} => {}", id, elem.name);
            println!("\t\t{:?}", elem.path);
            let options = elem.read_options()?;

            for option in options {
                println!(
                    "\t{}({}): --{}",
                    option.name,
                    option.default,
                    option.description.as_deref().unwrap_or("")
                );

                for item in option.options {
                    println!(
                        "\t\t{}: --{}",
                        item.data,
                        item.description.as_deref().unwrap_or("")
                    );
                }
            }
        }
    }

    Ok(())
}
