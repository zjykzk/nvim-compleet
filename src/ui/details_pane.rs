use mlua::{Lua, Result};
use neovim::Api;
use std::cmp;

use crate::ui::MenuPosition;

pub struct DetailsPane {
    /// The handle of the buffer used to show details for the currently
    /// selected completion items.
    bufnr: usize,

    /// The handle of the floating window used to show the detail infos, or
    /// `None` if the details pane is not currently visible.
    winid: Option<usize>,
}

impl DetailsPane {
    pub fn new(api: &Api) -> Result<Self> {
        Ok(DetailsPane {
            bufnr: api.create_buf(false, true)?,
            winid: None,
        })
    }
}

impl DetailsPane {
    /// TODO: docs
    fn create_floatwin(
        &self,
        lua: &Lua,
        api: &Api,
        width: usize,
        height: usize,
        menu_position: &MenuPosition,
    ) -> Result<usize> {
        let col = match menu_position {
            MenuPosition::Below(width) => *width,
        };

        let config = lua.create_table_with_capacity(0, 8)?;
        config.set("relative", "cursor")?;
        config.set("width", width)?;
        config.set("height", height)?;
        config.set("row", 1)?;
        config.set("col", col)?;
        config.set("focusable", false)?;
        config.set("style", "minimal")?;
        config.set("noautocmd", true)?;

        let winid = api.open_win(self.bufnr, false, config)?;
        api.win_set_option(winid, "winhl", "Normal:CompleetDetails")?;
        api.win_set_option(winid, "scrolloff", 0)?;
        Ok(winid)
    }

    /// TODO: docs
    pub fn hide(&mut self, api: &Api) -> Result<()> {
        if let Some(winid) = self.winid {
            api.win_hide(winid)?;
            self.winid = None;
        }
        Ok(())
    }

    /// TODO: docs
    pub fn is_visible(&self) -> bool {
        self.winid.is_some()
    }

    /// TODO: docs
    // fn set_lines<L: AsRef<str>>(
    fn set_lines(&self, api: &Api, lines: &[String]) -> Result<()> {
        api.buf_set_lines(self.bufnr, 0, -1, false, lines)
    }

    /// TODO: docs
    pub fn show(
        &mut self,
        lua: &Lua,
        api: &Api,
        lines: &[String],
        completion_menu_position: &MenuPosition,
    ) -> Result<()> {
        self.hide(api)?;

        let max_width = lines
            .iter()
            // TODO: Should use len of grapheme clusters, not bytes.
            .map(|line| line.len())
            .max()
            .unwrap_or(0);

        let width = cmp::min(max_width, 79);
        let height = lines.len();

        self.set_lines(api, lines)?;
        self.winid = Some(self.create_floatwin(
            lua,
            api,
            width,
            height,
            completion_menu_position,
        )?);

        // let print = lua.globals().get::<&str, mlua::Function>("print")?;
        // print.call::<_, ()>(format!("{:?}", self.winid))?;

        Ok(())
    }
}
