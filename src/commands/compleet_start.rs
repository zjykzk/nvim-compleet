use mlua::prelude::{Lua, LuaResult};
use neovim::{api::LogLevel, Neovim};

use crate::State;

/// Executed by the `CompleetStart` user command.
pub fn compleet_start(
    lua: &Lua,
    state: &mut State,
    bang: bool,
) -> LuaResult<()> {
    // The `CompleetStart!` command attaches to all the buffers, while
    // `CompleetStart` only attaches to the current buffer.
    match bang {
        true => attach_all_buffers(lua, state),
        false => attach_current_buffer(lua, state),
    }
}

/// TODO: docs
fn attach_all_buffers(lua: &Lua, state: &mut State) -> LuaResult<()> {
    let nvim = Neovim::new(lua)?;
    let api = &nvim.api;

    if state.augroup_id.is_some() {
        api.notify(
            "[nvim-compleet] Completion is already on",
            LogLevel::Error,
        )?;
        return Ok(());
    }

    state.buffers_to_be_detached.clear();

    // TODO: this leaves `state.try_buf_attach` as `None`, meaning the next
    // time `CompleetStart!` is called this will panic! We can't clone it bc
    // `state.try_buf_attach.unwrap()` is a `Box<dyn ..>`, which doesn't
    // implement `Clone` since it isn't sized.
    let try_buf_attach = lua.create_function(
        state
            .try_buf_attach
            .take()
            .expect("`try_buf_attach` has already been created"),
    )?;

    // Recreate the `Compleet` augroup.
    let opts = lua.create_table_from([("clear", true)])?;
    let augroup_id = api.create_augroup("Compleet", opts)?;

    // Add the `BufEnter` autocmd.
    let opts = lua.create_table_with_capacity(0, 2)?;
    opts.set("group", augroup_id)?;
    opts.set("callback", try_buf_attach.clone())?;
    api.create_autocmd(&["BufEnter"], opts)?;

    // We can't call `autocmds::try_buf_attach` here or the state's mutex would
    // deadlock. Instead we schedule it for a later time in neovim's event loop
    // via `vim.schedule`.
    nvim.schedule(try_buf_attach)?;

    state.augroup_id = Some(augroup_id);

    api.notify(
        "[nvim-compleet] Started completion in all buffers",
        LogLevel::Info,
    )?;

    Ok(())
}

fn attach_current_buffer(lua: &Lua, state: &mut State) -> LuaResult<()> {
    let nvim = Neovim::new(lua)?;
    let api = &nvim.api;

    let bufnr = api.get_current_buf()?;

    if state.attached_buffers.contains(&bufnr) {
        api.notify(
            "[nvim-compleet] Completion is already on in this buffer",
            LogLevel::Error,
        )?;
        return Ok(());
    }

    // If this buffer was queued to be detached from buffer update events (the
    // ones setup by `nvim_buf_attach`, not autocmds) now it no longer needs
    // to.
    if state.buffers_to_be_detached.contains(&bufnr) {
        state.buffers_to_be_detached.retain(|&b| b != bufnr);
    }

    // If there's currently no `Compleet` augroup we need to recreate it.
    if state.augroup_id.is_none() {
        let opts = lua.create_table_from([("clear", true)])?;
        state.augroup_id = Some(api.create_augroup("Compleet", opts)?);
    }

    // TODO: this leaves `state.try_buf_attach` as `None`, meaning the next
    // call to `CompleetStart` will panic!
    let try_buf_attach = lua.create_function(
        state
            .try_buf_attach
            .take()
            .expect("`try_buf_attach` has already been created"),
    )?;

    nvim.schedule(try_buf_attach)?;

    // TODO: only display this once we've successfully attached to the
    // buffer.
    api.notify(
        &format!("[nvim-compleet] Started completion in buffer {bufnr}"),
        LogLevel::Info,
    )?;

    Ok(())
}
