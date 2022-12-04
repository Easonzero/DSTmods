use crate::{DataBase, List, Mod};
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct SteamApps {
    client: [PathBuf; 2],
    server: PathBuf,
    client_list: HashMap<usize, Mod>,
    server_list: HashMap<usize, Mod>,
}

impl SteamApps {
    pub fn load(path: &Path) -> Result<Self> {
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

    pub fn sync(&self, id: usize) -> Result<()> {
        if let Some(x) = self.client_list.get(&id) {
            self.server.insert(x)?;
        } else {
            eprintln!("WARN! {} not found", id);
        }
        Ok(())
    }

    pub fn remove(&self, id: usize, client: bool) {
        if client {
            for database in &self.client {
                database.remove(id);
            }
        } else {
            self.server.remove(id);
        }
    }

    pub fn list(&self, args: &List) {
        let list = if args.client {
            &self.client_list
        } else {
            &self.server_list
        };

        println!(
            "{} {} mods:",
            if args.diff { "diff" } else { "list" },
            if args.client { "client" } else { "server" }
        );
        if args.diff {
            let diff_list = if args.client {
                &self.server_list
            } else {
                &self.client_list
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

    pub fn read_options(&self, id: usize) -> Result<()> {
        println!("list {} all options:", id);
        let elem = if let Some(elem) = self.client_list.get(&id) {
            elem
        } else if let Some(elem) = self.server_list.get(&id) {
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

        Ok(())
    }
}
