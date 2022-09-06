use std::cmp;
use std::ops::RangeBounds;

use nvim::api::{self, Buffer, Window};
use nvim::types::{WindowConfig, WindowRelativeTo};
use nvim_oxi as nvim;
use unicode_segmentation::UnicodeSegmentation;

use super::ui_config::MenuConfig;
use crate::CompletionItem;

const MENU_NAMESPACE: &str = "completion_menu";

#[derive(Debug)]
pub(crate) struct CompletionMenu {
    config: MenuConfig,

    /// TODO: docs
    buf: Buffer,

    /// TODO: docs
    win: Option<Window>,

    /// TODO: docs
    win_config: WindowConfig,

    /// TODO: docs
    height: u16,

    /// TODO: docs
    width: u16,

    /// TODO: docs
    namespace_id: u32,
}

impl Default for CompletionMenu {
    #[inline]
    fn default() -> Self {
        let win_config = WindowConfig::builder()
            .relative(WindowRelativeTo::Cursor)
            .noautocmd(true)
            .zindex(200)
            .build();

        Self {
            config: MenuConfig::default(),
            buf: 0.into(),
            win: None,
            win_config,
            height: 0,
            width: 0,
            namespace_id: api::create_namespace(MENU_NAMESPACE),
        }
    }
}

impl CompletionMenu {
    #[inline]
    pub(super) fn init(&mut self, config: MenuConfig) -> nvim::Result<()> {
        self.config = config;
        self.buf = api::create_buf(false, true)?;

        Ok(())
    }

    #[inline]
    pub(crate) fn is_open(&self) -> bool {
        self.win.is_some()
    }

    /// TODO: docs
    pub(crate) fn open(
        &mut self,
        completions: &[CompletionItem],
        start: &std::time::Instant, /* this will be removed, it's here just for
                                     * perf testing */
    ) -> nvim::Result<()> {
        debug_assert!(!self.is_open());

        // Populate the buffer.

        let lines = completions.iter().map(|cmp| cmp.single_line_display());
        self.buf.set_lines(0, 10, false, lines.clone())?;

        nvim::print!(
            "populating the buffer at {}ms",
            start.elapsed().as_millis()
        );

        // Open the window.

        let height = cmp::min(self.config.max_height, completions.len() as _);
        nvim::print!("computed height at {}ms", start.elapsed().as_millis());

        // Computing the "correct" width in terms of grapheme clusters is
        // around 8 times slower than using the number of code points.
        // Tested with 30k completions, using graphemes takes 57ms vs 7ms w/
        // code points.
        //
        // Code points is already a big improvement vs using raw bytes, so the
        // marginal increase in correctness is probably not worth 8x the
        // performance cost.
        let width = lines.map(|l| l.chars().count()).max().unwrap();
        // let width = lines.map(|l| l.graphemes(true).count()).max().unwrap();

        nvim::print!("computed width at {}ms", start.elapsed().as_millis());

        let config = WindowConfig::builder()
            .relative(WindowRelativeTo::Cursor)
            .row(1)
            .col(0)
            .height(height)
            .width(width as u32)
            .noautocmd(true)
            .zindex(200)
            .build();

        self.win = Some(api::open_win(&self.buf, false, &config)?);

        nvim::print!("opened window at {}ms", start.elapsed().as_millis());

        Ok(())
    }

    /// TODO: docs
    pub(crate) fn insert(
        &mut self,
        completions: &[(&CompletionItem, usize)],
    ) -> nvim::Result<()> {
        debug_assert!(self.is_open());

        Ok(())
    }

    /// Removes a set of lines from the completion menu.
    ///
    /// # Arguments
    ///
    /// `ranges`: a slice of line ranges where each item in a range represents
    /// the 0-based index of a line of the menu to be removed.
    ///
    /// **NOTE**: it assumes the ranges are disjoint and monotonically
    /// increasing. The buffer may be modified in unexpected ways if those
    /// conditions are not met.
    pub(super) fn remove<Range>(
        &mut self,
        ranges: &[Range],
    ) -> nvim::Result<()>
    where
        Range: RangeBounds<usize>,
    {
        debug_assert!(self.is_open());

        todo!()
    }

    /// Scrolls the completion menu.
    ///
    /// # Arguments
    ///
    /// `line`: the 0-based index of the line to scroll to
    pub(super) fn scroll(&mut self, line: usize) -> nvim::Result<()> {
        debug_assert!(self.is_open());

        todo!()
    }

    /// Closes the completion menu if it's currently open, does nothing if it's
    /// closed.
    pub(super) fn close(&mut self) -> nvim::Result<()> {
        if let Some(win) = self.win.take() {
            win.hide()?;
        }

        Ok(())
    }
}