use compleet::api::incoming::Request;
use mlua::prelude::{Lua, LuaFunction, LuaResult};

use crate::state::State;
use crate::ui::Buffer;
use crate::utils;

/// Called on every `BufEnter` event and by the `:CompleetStart{!}` user
/// commands.
pub fn on_buf_enter(
    lua: &Lua,
    state: &mut State,
    on_insert_leave: LuaFunction,
    on_cursor_moved_i: LuaFunction,
    on_bytes: LuaFunction,
) -> LuaResult<()> {
    let buffer = Buffer::get_current(lua)?;

    // Don't attach if:
    //
    // 1. the buffer is already attached;
    //
    // 2. the `modifiable` option is turned off. This should catch a large
    //    number of buffers we'd like to ignore like netwr, startify, terminal,
    //    help, etc;
    //
    // 3. the server doesn't have any source for this buffer.
    if state.attached_buffers.contains(&buffer)
        || !buffer.get_option::<bool>(lua, "modifiable")?
        || !state
            .channel
            .request::<bool>(lua, Request::ShouldAttach(buffer.number))?
    {
        return Ok(());
    }

    if !buffer.attach(lua, on_bytes)? {
        // Echo an error if for some reason we couldn't attach to the buffer.
        utils::echoerr(lua, "Couldn't attach to buffer")?;
    } else {
        // Add two buffer-local autocmds on this buffer.
        state.augroup.add_autocmds(
            lua,
            Some(&buffer),
            vec![
                ("CursorMovedI", on_cursor_moved_i),
                ("InsertLeave", on_insert_leave),
            ],
        )?;

        state.attached_buffers.push(buffer);
    };

    Ok(())
}
