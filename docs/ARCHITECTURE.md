# Architecture Overview

This document describes the high-level architecture of the Rust Microservices workspace and the main integration points.

Overview
- `crates/core`: core domain models and component runtime abstractions.
- `crates/hal`: hardware abstraction layer (traits + implementations).
- `crates/telemetry`: telemetry schema, collector, transports, and streaming pipeline.
- `crates/app`: main application that composes components and runs on a host or embedded runtime.
- `crates/demo_receiver`: small HTTP server that accepts telemetry POSTs (for demo purposes).
- `crates/demo_simulator`: host-side simulator that posts telemetry periodically to the receiver.

Integration Points
- Telemetry types are defined in `crates/telemetry::types` and shared across crates.
- `StreamingPipeline` (in `crates/telemetry::streaming`) accepts `TelemetryPacket` instances and forwards them to configured `PipelineTransport` adapters.
- Transports in `crates/telemetry::transports` include file-backed adapters and can be extended with real MQTT or serial implementations behind features.

Running the demo locally
1. Start the demo receiver:
   ```bash
   cargo run -p demo_receiver
   ```
2. In another shell, run the simulator:
   ```bash
   cargo run -p demo_simulator
   ```

Embedded support
- The `crates/app` crate contains two build modes: `tokio_runtime` (host simulation) and `rtic_firmware` (MCU firmware scaffold). To build for real MCUs, create an RTIC-based firmware crate that depends on `crates/core` and `crates/hal` with target triples configured.

Documentation and next steps
- Add more detailed component-level sequence diagrams and platform-specific build instructions in `docs/`. Consider adding an end-to-end walkthrough with screenshots and a small recorded demo.
