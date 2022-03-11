use mlua::{Function, Result, Table};

use super::Api;

impl<'a> Api<'a> {
    /// Binding to `vim.api.nvim_create_buf`.
    ///
    /// Creates a new, empty, unnamed buffer. Returns the new buffer handle, or
    /// 0 on error.
    ///
    /// # Arguments
    ///
    /// * `listed`   Whether to set `buflisted`
    /// * `scratch`  Whether the new buffer is a "throwaway" (`:h scratch-buffer`) buffer used for temporary work.
    pub fn create_buf(&self, listed: bool, scratch: bool) -> Result<usize> {
        Ok(self
            .0
            .get::<&str, Function>("nvim_create_buf")?
            .call::<_, usize>((listed, scratch))?)
    }

    /// Binding to `vim.api.nvim_echo`.
    ///
    /// Echoes a message.
    ///
    /// # Arguments
    ///
    /// * `chunks`   A slice of `(text, hlgroup)` tuples, each representing a
    /// text chunk with specified highlight.
    /// * `history`  Whether to add the message to the message history
    pub fn echo(&self, chunks: &[(&str, &str)], history: bool) -> Result<()> {
        let chunks = chunks
            .iter()
            .map(|c| [c.0, c.1])
            .collect::<Vec<[&str; 2]>>();

        Ok(self.0.get::<&str, Function>("nvim_echo")?.call::<_, ()>((
            chunks,
            history,
            Vec::<u8>::new(),
        ))?)
    }

    /// Binding to `vim.api.nvim_get_current_line`
    ///
    /// Returns the current line.
    pub fn get_current_line(&self) -> Result<String> {
        Ok(self
            .0
            .get::<&str, Function>("nvim_get_current_line")?
            .call::<_, String>(())?)
    }

    /// Binding to `vim.api.nvim_set_hl`
    ///
    /// Sets a highlight group
    ///
    /// # Arguments
    ///
    /// * `ns_id`  Namespace to use, or 0 to set a highlight group in the global namespace
    /// * `name`   Highlight group name
    /// * `opts`   Optional parameters. See `:h nvim_set_hl` for  details
    pub fn set_hl(&self, ns_id: usize, name: &str, opts: Table) -> Result<()> {
        Ok(self
            .0
            .get::<&str, Function>("nvim_set_hl")?
            .call::<_, ()>((ns_id, name, opts))?)
    }
}
