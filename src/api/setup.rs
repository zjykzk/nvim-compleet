use mlua::{Function, Lua, Result, Table};
use std::sync::{Arc, Mutex};

use crate::settings::{Error, Settings};
use crate::{Nvim, State};

const COMPLEET_AUGROUP_NAME: &'static str = "Compleet";

/// Executed on every call to `require("compleet").setup({..})`.
pub fn setup(
    lua: &Lua,
    state: &Arc<Mutex<State>>,
    preferences: Option<Table>,
) -> Result<()> {
    let nvim = Nvim::new(lua)?;

    let _state = state.clone();
    let _state = &mut _state.lock().unwrap();

    _state.settings = match Settings::try_from(preferences) {
        Ok(settings) => settings,
        Err(e) => match e {
            Error::OptionDoesntExist { option } => {
                nvim.echo(
                    &[
                        &["[nvim-compleet]: ", "ErrorMsg"],
                        &["Config option '"],
                        &[&option, "Statement"],
                        &["' doesn't exist!"],
                    ],
                    true,
                    &[],
                )?;

                return Ok(());
            },

            Error::FailedConversion { option, expected } => {
                nvim.echo(
                    &[
                        &["[nvim-compleet]: ", "ErrorMsg"],
                        &["Error parsing config option '"],
                        &[option, "Statement"],
                        &[&format!("': expected a {expected}.")],
                    ],
                    true,
                    &[],
                )?;

                return Ok(());
            },

            Error::InvalidValue { option, reason } => {
                nvim.echo(
                    &[
                        &["[nvim-compleet]: ", "ErrorMsg"],
                        &["Invalid value for config option '"],
                        &[&option, "Statement"],
                        &[&format!("': {reason}.")],
                    ],
                    true,
                    &[],
                )?;

                return Ok(());
            },

            Error::Lua(e) => return Err(e),
        },
    };

    // let print = lua.globals().get::<&str, Function>("print")?;
    // print.call::<_, ()>(format!("{:?}", &config))?;

    _state.ui.completion_menu.max_height = _state.settings.max_menu_height;

    setup_augroups(lua, &nvim, state)?;
    setup_hlgroups(lua, &nvim)?;
    setup_mappings(lua, state)?;

    if _state.settings.enable_default_mappings {
        enable_default_mappings(lua, state)?;
    }

    Ok(())
}

fn setup_augroups(
    lua: &Lua,
    nvim: &Nvim,
    state: &Arc<Mutex<State>>,
) -> Result<()> {
    let _state = state.clone();
    let cleanup = lua.create_function(move |lua, ()| {
        super::cleanup_ui(lua, &mut _state.lock().unwrap().ui)
    })?;

    let _state = state.clone();
    let maybe_show_hint = lua.create_function(move |lua, ()| {
        super::maybe_show_hint(lua, &mut _state.lock().unwrap())
    })?;

    let _state = state.clone();
    let text_changed = lua.create_function(move |lua, ()| {
        super::text_changed(lua, &mut _state.lock().unwrap())
    })?;

    let opts = lua.create_table_with_capacity(0, 1)?;
    opts.set("clear", true)?;
    let _group_id = nvim.create_augroup(COMPLEET_AUGROUP_NAME, opts)?;

    let opts_w_callback = |callback: Function| -> Result<Table> {
        let opts = lua.create_table_with_capacity(0, 2)?;
        // TODO: why doesn't it work if I use the group id returned by
        // `create_augroup` here instead of the name?
        // opts.set("group", group_id)?;
        opts.set("group", COMPLEET_AUGROUP_NAME)?;
        opts.set("callback", callback)?;
        Ok(opts)
    };

    nvim.create_autocmd(
        &["CursorMovedI", "InsertLeave"],
        opts_w_callback(cleanup)?,
    )?;
    nvim.create_autocmd(
        &["CursorMovedI", "InsertEnter"],
        opts_w_callback(maybe_show_hint)?,
    )?;
    nvim.create_autocmd(&["TextChangedI"], opts_w_callback(text_changed)?)?;

    Ok(())
}

fn setup_hlgroups(lua: &Lua, nvim: &Nvim) -> Result<()> {
    // TODO: make something like this work
    // nvim.set_hl(0, "CompleetMenu", lua.t! { link = "NormalFloat" })?;

    // `CompleetMenu`
    // Used to highlight the completion menu.
    let opts = lua.create_table_from([("link", "NormalFloat")])?;
    nvim.set_hl(0, "CompleetMenu", opts)?;

    // `CompleetHint`
    // Used to highlight the completion hint.
    let opts = lua.create_table_from([("link", "Comment")])?;
    nvim.set_hl(0, "CompleetHint", opts)?;

    // `CompleetDetails`
    // Used to highlight the details window.
    let opts = lua.create_table_from([("link", "NormalFloat")])?;
    nvim.set_hl(0, "CompleetDetails", opts)?;

    // `CompleetMenuSelected`
    // Used to highlight the currently selected completion item.
    let opts = lua.create_table_from([("link", "PmenuSel")])?;
    nvim.set_hl(0, "CompleetMenuSelected", opts)?;

    // `CompleetMenuMatchingChars`
    // Used to highlight the characters where a completion item matches the
    // current prefix.
    let opts = lua.create_table_from([("link", "Statement")])?;
    nvim.set_hl(0, "CompleetMenuMatchingChars", opts)?;

    Ok(())
}

fn setup_mappings(lua: &Lua, state: &Arc<Mutex<State>>) -> Result<()> {
    // Insert the currently hinted completion
    let _state = state.clone();
    let insert_hinted_completion = lua.create_function(move |lua, ()| {
        let _state = &mut _state.lock().unwrap();
        if let Some(index) = _state.ui.completion_hint.hinted_index {
            super::insert_completion(lua, &mut _state.completion, index)?;
        }
        Ok(())
    })?;

    // Insert the currently selected completion
    let _state = state.clone();
    let insert_selected_completion = lua.create_function(move |lua, ()| {
        let _state = &mut _state.lock().unwrap();
        if let Some(index) = _state.ui.completion_menu.selected_completion {
            super::insert_completion(lua, &mut _state.completion, index)?;
        }
        Ok(())
    })?;

    // Select either the previous or next completion in the completion menu
    // based on the value of `step`.
    let _state = state.clone();
    let select_completion = lua.create_function(move |lua, step| {
        super::select_completion(lua, &mut _state.lock().unwrap(), step)
    })?;

    // Show the completion menu with all the currently available completion
    // candidates.
    let _state = state.clone();
    let show_completions = lua.create_function(move |lua, ()| {
        super::show_completions(lua, &mut _state.lock().unwrap())
    })?;

    let set_keymap = lua
        .globals()
        .get::<&str, Table>("vim")?
        .get::<&str, Table>("keymap")?
        .get::<&str, Function>("set")?;

    let opts = lua.create_table_with_capacity(0, 1)?;
    opts.set("silent", true)?;

    set_keymap.call::<_, ()>((
        "i",
        "<Plug>(compleet-insert-hinted-completion)",
        insert_hinted_completion,
        opts.clone(),
    ))?;

    set_keymap.call::<_, ()>((
        "i",
        "<Plug>(compleet-insert-selected-completion)",
        insert_selected_completion,
        opts.clone(),
    ))?;

    set_keymap.call::<_, ()>((
        "i",
        "<Plug>(compleet-next-completion)",
        select_completion.bind(1)?,
        opts.clone(),
    ))?;

    set_keymap.call::<_, ()>((
        "i",
        "<Plug>(compleet-prev-completion)",
        select_completion.bind(-1)?,
        opts.clone(),
    ))?;

    set_keymap.call::<_, ()>((
        "i",
        "<Plug>(compleet-show-completions)",
        show_completions,
        opts.clone(),
    ))?;

    Ok(())
}

fn enable_default_mappings(
    lua: &Lua,
    state: &Arc<Mutex<State>>,
) -> Result<()> {
    // Insert mode mapping for `<Tab>`. If the completion menu is visible
    // select the next completion, if it isn't but there are completions to be
    // displayed show the completion menu, else just return `<Tab>`.
    let _state = state.clone();
    let tab = lua.create_function(move |lua, ()| -> Result<&'static str> {
        let _state = &mut _state.lock().unwrap();
        if _state.ui.completion_menu.is_visible() {
            Ok("<Plug>(compleet-next-completion)")
        } else if super::has_completions(lua, &mut _state.completion)? {
            Ok("<Plug>(compleet-show-completions)")
        } else {
            Ok("<Tab>")
        }
    })?;

    // Insert mode mapping for `<Tab>`. If the completion menu is visible
    // select the previous completion, else just return `<S-Tab>`.
    let _state = state.clone();
    let s_tab = lua.create_function(move |_, ()| -> Result<&'static str> {
        if _state.lock().unwrap().ui.completion_menu.is_visible() {
            Ok("<Plug>(compleet-prev-completion)")
        } else {
            Ok("<S-Tab>")
        }
    })?;

    // Insert mode mapping for `<Tab>`. If a completion item in the completion
    // menu is currently selected insert it, else just return `<CR>`.
    let _state = state.clone();
    let cr = lua.create_function(move |_, ()| -> Result<&'static str> {
        if _state.lock().unwrap().ui.completion_menu.is_item_selected() {
            Ok("<Plug>(compleet-insert-selected-completion)")
        } else {
            Ok("<CR>")
        }
    })?;

    let set_keymap = lua
        .globals()
        .get::<&str, Table>("vim")?
        .get::<&str, Table>("keymap")?
        .get::<&str, Function>("set")?;

    let opts = lua.create_table_with_capacity(0, 2)?;
    opts.set("expr", true)?;
    opts.set("remap", true)?;

    set_keymap.call::<_, ()>(("i", "<Tab>", tab, opts.clone()))?;
    set_keymap.call::<_, ()>(("i", "<S-Tab>", s_tab, opts.clone()))?;
    set_keymap.call::<_, ()>(("i", "<CR>", cr, opts.clone()))?;

    Ok(())
}