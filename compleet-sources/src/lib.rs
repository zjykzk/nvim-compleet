pub mod lipsum;
pub mod lsp;

mod completion_item;
mod completion_source;
mod cursor;
mod result;
mod valid_source;

use result::Result;

pub mod prelude {
    pub use crate::completion_item::{CompletionItem, Completions};
    pub use crate::completion_source::{CompletionSource, Sources};
    pub use crate::cursor::Cursor;
    pub use crate::result::Result;
    pub use crate::valid_source::ValidSource;
}
