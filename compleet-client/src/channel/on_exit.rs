use mlua::prelude::{Lua, LuaResult};

use crate::bindings::api;
use crate::state::State;
use crate::ui;
use crate::utils;

/// Called when the RPC channel gets closed.
pub fn on_exit(lua: &Lua, state: &mut State, exit_code: u32) -> LuaResult<()> {
    match exit_code {
        // Exit code 143 means the server received a SIGTERM. That happens when
        // the user quits Neovim, which is not an error.
        143 => {},

        // All others exit codes should be considered errors.
        num => {
            // Cleanup the UI.
            ui::cleanup(lua, &mut state.ui.as_mut().unwrap())?;

            // Remove all the autocmds.
            if let Some(id) = state.augroup_id {
                api::del_augroup_by_id(lua, id)?;
            }

            // Echo an error message to the user.
            utils::echoerr(
                lua,
                vec![
                    ("The server just quit with exit code ", None),
                    (&num.to_string(), Some("Visual")),
                ],
            )?;
        },
    };

    Ok(())
}
