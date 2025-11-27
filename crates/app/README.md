# app crate

This crate is the application entry point for the Rust Micro Services example.

Runtime selection

- The crate defaults to using the `tokio_runtime` feature. That runtime is suitable for Embedded Linux or general OS targets.
- An RTIC firmware variant is planned as a separate crate/scaffold and is selected with the `rtic_firmware` feature (not included in this workspace by default).

Build & run (default - Tokio):

```powershell
# build and run using the default tokio runtime feature
cargo run -p app
```

Build with explicit feature selection:

```powershell
# enable the tokio runtime feature explicitly
cargo run -p app --features tokio_runtime
```

Shutdown behavior

- Press Ctrl-C while the app is running to trigger a graceful shutdown.
- The runtime uses a `CancellationToken` that is passed to components' `run()` method; components should observe cancellation and exit promptly.

