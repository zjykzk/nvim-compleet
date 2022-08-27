use std::sync::Arc;

use tokio::sync::mpsc::UnboundedReceiver;

use crate::{Client, CompletionBundle};

/// Messages sent from the thread pool to the main thread.
#[derive(Debug)]
pub(crate) enum MainMessage {
    /// TODO: docs
    AttachBuf(Arc<crate::Buffer>),

    /// TODO: docs
    HandleCompletions(CompletionBundle),
}

pub(crate) fn main_cb(
    client: &Client,
    receiver: &mut UnboundedReceiver<MainMessage>,
) -> crate::Result<()> {
    let mut bundles = Vec::<CompletionBundle>::new();

    while let Ok(msg) = receiver.try_recv() {
        match msg {
            MainMessage::AttachBuf(buf) => client.attach_buffer(buf)?,
            MainMessage::HandleCompletions(bundle) => bundles.push(bundle),
        }
    }

    if !bundles.is_empty() {
        crate::on_completions_arrival(client, bundles)?;
    }

    Ok(())
}
