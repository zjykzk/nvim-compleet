use std::rc::Rc;

use mlua::prelude::{Lua, LuaError, LuaResult, LuaValue};
use mlua::serde::Deserializer;
use parking_lot::Mutex;
use serde_path_to_error::deserialize;

use crate::bindings::r#fn;
use crate::channel::Channel;
use crate::settings::Settings;
use crate::state::State;
use crate::{autocmds, commands, hlgroups, mappings, utils};

/// Executed by the `require("compleet").setup` Lua function.
pub fn setup(
    lua: &Lua,
    state: &Rc<Mutex<State>>,
    preferences: LuaValue,
) -> LuaResult<()> {
    // Setup the highlight groups used in the error messages.
    hlgroups::setup_error_msg(lua)?;

    // If the Neovim version isn't 0.7+ we echo an error message and return
    // early.
    if !r#fn::has(lua, "nvim-0.7")? {
        utils::echoerr(lua, vec![("Neovim v0.7+ is required", None)])?;
        return Ok(());
    }

    // Try to merge the `preferences` table with the default settings, echoing
    // an error message and returning early if something is wront.
    let settings = match preferences {
        LuaValue::Nil => Settings::default(),

        LuaValue::Table(t) => match deserialize::<_, Settings>(
            Deserializer::new(LuaValue::Table(t)),
        ) {
            Ok(settings) => settings,

            Err(e) => match e.inner() {
                LuaError::DeserializeError(msg) => {
                    let path = e.path().to_string();

                    let chunks = [
                        ("Error for `", None),
                        (&path, Some("CompleetErrorMsgOptionPath")),
                        ("`: ", None),
                    ]
                    .into_iter()
                    .chain(to_chunks(msg).into_iter())
                    .collect::<Vec<(&str, Option<&str>)>>();

                    utils::echoerr(lua, chunks)?;
                    return Ok(());
                },

                _ => return Err(e.into_inner()),
            },
        },

        _ => {
            let preferences = format!("{:?}", preferences);
            let chunks = vec![
                ("Invalid value '", None),
                (&preferences, Some("Statement")),
                ("'. Please pass either a table or `", None),
                ("nil", Some("Statement")),
                ("` to the setup function", None),
            ];
            utils::echoerr(lua, chunks)?;
            return Ok(());
        },
    };

    // If there aren't any sources enabled we echo an error message and return
    // early.
    if settings.sources.is_empty() {
        let chunks = vec![(
            "No sources have been enabled. I'm feeling more useless than \
             nipples on a man :(",
            None,
        )];
        utils::echoerr(lua, chunks)?;
        return Ok(());
    }

    // Lock the Mutex and update the state.
    let locked = &mut state.lock();
    locked.settings = settings;

    if !locked.did_setup {
        let (id, registry_key) = autocmds::setup(lua, state)?;

        locked.augroup_id = Some(id);
        locked.try_buf_attach = Some(registry_key);

        commands::setup(lua, state)?;
        hlgroups::setup(lua)?;
        mappings::setup(lua, state)?;

        // Spawn the RPC channel to commicate with the compleet server.
        locked.channel = Some(Channel::new(lua)?);

        locked.did_setup = true;
    }

    Ok(())
}

fn to_chunks(msg: &str) -> Vec<(&'_ str, Option<&'static str>)> {
    msg.split('`')
        .enumerate()
        .map(|(i, str)| match i % 2 == 1 {
            true => vec![
                ("`", None),
                (str, Some("CompleetErrorMsgField")),
                ("`", None),
            ],
            false => vec![(str, None)],
        })
        .flatten()
        .collect()
}