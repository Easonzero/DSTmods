use crate::lua_ctx::*;
use anyhow::Result;
use std::fs::ReadDir;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct OptionItem {
    pub data: String,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct ModOption {
    pub name: String,
    pub options: Vec<OptionItem>,
    pub description: Option<String>,
    pub default: String,
}

#[derive(Debug)]
pub struct Mod {
    pub name: String,
    pub id: usize,
    pub path: PathBuf,
    pub client_only: bool,
}

impl Mod {
    pub fn read_options(&self) -> Result<Vec<ModOption>> {
        let file_name = self.path.file_name().unwrap();
        let file_name = file_name.to_str().unwrap();
        let info_path = self.path.join("modinfo.lua");
        let content = std::fs::read_to_string(info_path)?;
        lua().context(|ctx| -> Result<_> {
            let globals = ctx.globals();
            globals.set("folder_name", file_name)?;
            ctx.load(&content).exec()?;
            let mut options = vec![];
            if let Ok(roptions) = globals.get::<_, rlua::Table>("configuration_options") {
                for roption in roptions.sequence_values::<rlua::Table>() {
                    let roption = roption?;
                    let mut items = vec![];
                    let client: bool = roption.get("client").unwrap_or(false);
                    if client {
                        continue;
                    }
                    let name: String = roption.get("name")?;
                    let description: Option<String> = roption.get("label").ok();
                    let default: rlua::Value = roption.get("default")?;
                    let default = value2str(&default)?;

                    let roption_items: rlua::Table = roption.get("options")?;
                    for item in roption_items.sequence_values::<rlua::Table>() {
                        let item = item?;
                        let data: rlua::Value = item.get("data")?;
                        let data = value2str(&data)?;
                        let description: Option<String> = item.get("description").ok();

                        items.push(OptionItem { data, description });
                    }
                    options.push(ModOption {
                        name,
                        options: items,
                        description,
                        default,
                    })
                }
            }
            Ok(options)
        })
    }
}

pub struct ModIter(ReadDir);

impl Iterator for ModIter {
    type Item = Mod;

    fn next(&mut self) -> Option<Self::Item> {
        let Ok(entry) = self.0.next()? else { return self.next(); };
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_str().unwrap();
        let Some(Ok(id)) = file_name
            .split('-')
            .rev()
            .next()
            .map(|x| x.parse::<usize>()) else { return self.next(); };

        let info_path = path.join("modinfo.lua");
        let Ok(content) = std::fs::read_to_string(info_path) else { return self.next(); };

        let x = lua()
            .context(|ctx| -> rlua::Result<_> {
                let globals = ctx.globals();
                globals.set("folder_name", file_name)?;
                ctx.load(&content).exec()?;
                let name: String = globals.get("name")?;
                let client_only: bool = globals.get("client_only_mod")?;

                Ok(Mod {
                    name,
                    path,
                    id,
                    client_only,
                })
            })
            .unwrap_or_else(|_| panic!("cannot parse mod {}", id));
        Some(x)
    }
}

pub trait DataBase: AsRef<Path> {
    fn get_name(&self, id: usize) -> String {
        if self.as_ref().file_name().map(|x| x == "322330").unwrap() {
            id.to_string()
        } else {
            format!("workshop-{}", id)
        }
    }

    fn list(&self) -> Result<ModIter> {
        Ok(ModIter(std::fs::read_dir(self.as_ref())?))
    }

    fn remove(&self, id: usize) {
        let name = self.get_name(id);
        std::fs::remove_dir_all(self.as_ref().join(&name)).ok();
    }

    fn insert(&self, elem: &Mod) -> Result<()> {
        let name = self.get_name(elem.id);
        let options = fs_extra::dir::CopyOptions::new();
        fs_extra::dir::copy(elem.path.as_path(), self.as_ref().join(&name), &options)?;
        Ok(())
    }
}

impl<T> DataBase for T where T: AsRef<Path> {}
