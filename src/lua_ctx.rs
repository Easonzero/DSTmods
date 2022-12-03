use rlua::prelude::*;

static mut LUA: Option<Lua> = None;

pub fn lua() -> &'static mut Lua {
    unsafe {
        if let Some(lua_mut) = LUA.as_mut() {
            lua_mut
        } else {
            LUA = Some(Lua::new());
            let lua_mut = LUA.as_mut().unwrap();

            lua_mut.context(|ctx| {
                let globals = ctx.globals();
                globals.set("locale", "zh").unwrap();
            });

            lua_mut
        }
    }
}

pub fn value2str(value: &rlua::Value) -> rlua::Result<String> {
    Ok(match value {
        rlua::Value::Boolean(value) => value.to_string(),
        rlua::Value::String(value) => format!("\"{}\"", value.to_str()?),
        rlua::Value::Integer(value) => value.to_string(),
        rlua::Value::Number(value) => value.to_string(),
        _ => return Err(rlua::Error::UserDataTypeMismatch),
    })
}
