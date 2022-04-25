use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use bindings::opinionated::Neovim;
use mlua::prelude::{Lua, LuaResult};
use tokio::sync::Mutex;

use crate::prelude::{Completions, Cursor, Result};

pub type Sources = Vec<Arc<Mutex<dyn CompletionSource>>>;

#[async_trait]
pub trait CompletionSource: Debug + Send + Sync {
    /// Called once when starting the plugin.
    fn setup(&mut self, _lua: &Lua) -> LuaResult<()> {
        Ok(())
    }

    /// Decides whether to attach the source to a buffer.
    async fn attach(&mut self, nvim: &Neovim, bufnr: u16) -> bool;

    /// Returns the completion results.
    async fn complete(
        &self,
        nvim: &Neovim,
        cursor: &Cursor,
    ) -> Result<Completions>;
}
