use crate::lua_ctx::value2str;
use anyhow::Result;
use rlua::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Override {
    paths: Vec<PathBuf>,
    options: HashMap<usize, HashMap<String, String>>,
}

impl Override {
    pub fn load(path: &Path) -> Result<Self> {
        let paths = vec![
            path.join("DoNotStarveTogether/BearDediServer/world0/modoverrides.lua"),
            path.join("DoNotStarveTogether/BearDediServer/world1/modoverrides.lua"),
        ];

        let content = fs::read_to_string(&paths[0])?;

        let mut options = HashMap::new();
        Lua::new().context(|ctx| -> Result<_> {
            let table: rlua::Table = ctx.load(&content).eval()?;
            for pair in table.pairs::<String, rlua::Table>() {
                let (k, v) = pair?;
                let id: usize = k.split('-').rev().next().unwrap().parse()?;
                let mut items = HashMap::new();
                let ritems: rlua::Table = v.get("configuration_options")?;
                for pair in ritems.pairs::<String, rlua::Value>() {
                    let (k, v) = pair?;
                    let v = value2str(&v)?;

                    items.insert(k, v);
                }
                options.insert(id, items);
            }
            Ok(())
        })?;

        Ok(Self { paths, options })
    }

    pub fn list(&self, id: usize) {
        if let Some(items) = self.options.get(&id) {
            println!("list {} options:", id);
            for (k, v) in items {
                println!("{:<10} => {}", k, v);
            }
        }
    }

    pub fn insert(&mut self, id: usize, opt: HashMap<String, String>) {
        self.options.entry(id).or_default().extend(opt);
    }

    pub fn remove(&mut self, id: usize) {
        self.options.remove(&id);
    }

    pub fn sink(&self) -> Result<()> {
        let base = &self.paths[0];
        let file = fs::File::create(base)?;
        let mut file = BufWriter::new(file);

        self.dump(&mut file)?;
        file.flush()?;

        for other in &self.paths[1..] {
            fs::copy(base, other)?;
        }

        Ok(())
    }

    fn dump(&self, writer: &mut impl Write) -> Result<()> {
        writeln!(writer, "return {{")?;

        for (id, items) in &self.options {
            write!(
                writer,
                "\t[\"workshop-{}\"] = {{ configuration_options = {{",
                id
            )?;

            if !items.is_empty() {
                write!(writer, "\n\t")?;
            }

            for (k, v) in items {
                write!(writer, "\t{}={},\n\t", k, v)?;
            }

            writeln!(writer, "}}, enabled = true }},")?;
        }

        writeln!(writer, "}}")?;
        Ok(())
    }
}
