pub mod any_cache;
pub mod file_store;
pub mod thread_local_cell;
pub mod webfs;

use futures::{
    Future,
    FutureExt,
};
use leptos::spawn_local;

pub fn spawn_local_and_handle_error<
    F: Future<Output = Result<(), E>> + 'static,
    E: std::error::Error,
>(
    fut: F,
) {
    spawn_local(fut.map(|result| {
        if let Err(error) = result {
            let mut error: &dyn std::error::Error = &error;

            tracing::error!(%error);

            while let Some(source) = error.source() {
                tracing::error!(%source);
                error = source;
            }
        }
    }));
}
