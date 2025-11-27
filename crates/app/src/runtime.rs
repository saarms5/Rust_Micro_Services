#[cfg(feature = "tokio_runtime")]
use std::future::Future;

/// Small runtime helper that starts a Tokio runtime and blocks on the
/// provided future. This keeps `main.rs` free of the `#[tokio::main]`
/// attribute so the runtime can be selected via Cargo features.
#[cfg(feature = "tokio_runtime")]
pub fn run<F>(fut: F)
where
    F: Future<Output = ()>,
{
    let rt = tokio::runtime::Runtime::new().expect("failed to create Tokio runtime");
    rt.block_on(fut);
}
