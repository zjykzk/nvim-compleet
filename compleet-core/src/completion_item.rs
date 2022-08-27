/// TODO: docs
#[derive(Debug)]
pub struct CompletionItem {
    pub(crate) text: String,
}

impl CompletionItem {
    #[inline]
    pub(crate) fn new<T: Into<String>>(text: T) -> Self {
        Self { text: text.into() }
    }
}

/// TODO: docs
#[derive(Debug, Clone)]
pub struct CompletionItemBuilder {
    text: Option<String>,
}

impl CompletionItemBuilder {
    /// TODO: docs
    #[inline(always)]
    pub fn new<T: Into<String>>(text: T) -> Self {
        Self { text: Some(text.into()) }
    }

    /// TODO: docs
    #[inline(always)]
    pub fn build(mut self) -> CompletionItem {
        CompletionItem { text: self.text.take().unwrap() }
    }
}
