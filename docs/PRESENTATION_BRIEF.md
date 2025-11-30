Rust Micro Services — Project Presentation Brief

Executive Summary

- Purpose: Rust Micro Services is a small modular Rust workspace that demonstrates embedded-friendly telemetry collection, streaming, and remote delivery. It includes a simulated runtime, file-backed transports, a streaming pipeline with optional compression, and CI/CD with notification hooks.
- Scope: Prototype-level system for demonstrating telemetry collection, processing, and remote forwarding in both embedded and simulation environments. Not intended as a production-ready cloud service.

Key Capabilities

- Telemetry Schema and Collector:
  - Structured telemetry packet types (`TelemetryPacket`, `SystemHealth`, `SensorReading`, `DiagnosticsReport`).
  - Serialization with Serde to JSON and support for compact binary-friendly representation.

- Transport Abstractions:
  - `Transport` trait with adapters.
  - File-backed `MqttTransport` and `SerialTransport` used for local testing.
  - Easily replaceable with real MQTT or serial implementations behind feature flags.

- Streaming Pipeline:
  - `StreamingPipeline` batches telemetry, compresses payloads when suitable, and sends them via a chosen transport.
  - Non-blocking design using `tokio` to ensure main control loops remain responsive.
  - `PipelineTransport` enum to avoid async-trait object issues; supports multiple transport backends.

- Demo & Simulation:
  - `crates/demo_simulator`: a small client that periodically generates telemetry and posts to a receiver.
  - `crates/demo_receiver`: a simple HTTP server accepting POSTed telemetry for demonstration.
  - `crates/app`: main application integrating the telemetry pipeline for local runs.

- Testing, Fuzzing, and Benchmarks:
  - Unit tests and integration tests (including an E2E integration under `crates/telemetry/tests`).
  - Fuzz harness under `fuzz/` to validate parsing robustness.
  - Criterion benches (or `benches/`) in crates for performance evaluation.

- CI and Notifications:
  - GitHub Actions matrix CI (`.github/workflows/ci.yml`) for Ubuntu, Windows, macOS.
  - Notification workflows (`notify_develop.yml`, `notify_test.yml`) that can send SMTP email via `dawidd6/action-send-mail` when configured.
  - Workflows include an opt-in secret `SEND_ON_SUCCESS` to control success emails and a fallback to open a GitHub Issue when SMTP is not configured.

Architecture Overview

- Workspace structure:
  - Top-level crates: `app`, `core` (package name `rms_core`), `telemetry`, `hal`, `demo_receiver`, `demo_simulator`.
  - `telemetry` owns the types, collector, transports, and streaming pipeline.
  - `core` contains shared models, control loops, and scheduler logic.
  - `hal` contains hardware abstraction and device mocks used by both `app` and tests.

- Data Flow:
  - Sensors and system components create `TelemetryPacket` instances.
  - A `TelemetryCollector` centralizes and forwards packets to the `StreamingPipeline`.
  - The pipeline batches packets, optionally compresses them (gzip), and uses a `PipelineTransport` to send compressed payloads to the remote target.

Run & Development Guide

Prerequisites:
- Rust toolchain (stable) and `cargo` installed.
- Optional: `gh` CLI for workflow dispatching and GitHub operations.

Clone and build:

```powershell
git clone https://github.com/saarms5/Rust_Micro_Services.git
cd Rust_Micro_Services
cargo build --workspace
```

Run the demo receiver (accepts HTTP POST telemetry):

```powershell
cd crates/demo_receiver
cargo run
```

Run the demo simulator (sends telemetry to receiver):

```powershell
cd crates/demo_simulator
cargo run -- --url http://localhost:8080/telemetry
```

Run the app (local simulation):

```powershell
cargo run -p app
```

CI and Notifications

- CI: Pushes to `develop` trigger the matrix CI. The `notify_develop.yml` workflow performs build and tests; if SMTP secrets are configured and `SEND_ON_SUCCESS` is set to `1`, an email will be sent on success/failure.

- Secrets required for SMTP email (set these in GitHub repository Secrets):
  - `SMTP_SERVER`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `EMAIL_FROM`, `NOTIFY_TO`, `SEND_ON_SUCCESS`

- Manual test: use the `Notify Test` workflow (Actions → Notify Test → Run workflow) to validate SMTP credentials.

Troubleshooting

- No email received:
  - Confirm all SMTP secrets are present and correct.
  - Confirm `SEND_ON_SUCCESS` is set to `1` to enable success emails.
  - Check workflow logs in Actions for `dawidd6/action-send-mail` output and errors.
  - Some providers require app-specific passwords or TLS; confirm `SMTP_PORT` and provider policy.

- Workflow failed creating issues (403):
  - Workflow-level `permissions` were added to allow `issues: write` and `contents: read` so the fallback can create GitHub Issues using the `GITHUB_TOKEN`.

Extensibility & Next Steps

- Implement production-grade transports (MQTT via `rumqttc`, serial via `serialport-rs`) behind Cargo features.
- Add Prometheus or OpenTelemetry exporters to stream metrics in addition to telemetry packets.
- Harden pipeline with retries, backoff, and persistent storage for offline buffering.
- Add more realistic simulator scenarios and dataset-driven fuzz targets.

Appendix: Important Files

- `crates/telemetry/src/streaming.rs` — streaming pipeline implementation
- `crates/telemetry/src/transports.rs` — file-backed transport adaptors
- `crates/demo_receiver/src/main.rs` — demo HTTP receiver
- `crates/demo_simulator/src/main.rs` — demo simulator client
- `.github/workflows/ci.yml` — CI matrix
- `.github/workflows/notify_develop.yml` — develop notification workflow
- `.github/workflows/notify_test.yml` — manual notification test

Contact / Maintainer

- Repository owner: `saarms5` (GitHub)
- For email notifications, configure `NOTIFY_TO` to the desired recipient and ensure SMTP credentials are valid.

End of brief
