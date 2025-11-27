# Rust Microservices

A modular Rust microservices framework with clear separation of concerns.

## Workspace Structure

```
├── crates/
│   ├── core/          # Core business logic and domain models
│   ├── hal/           # Hardware Abstraction Layer
│   ├── telemetry/     # Logging and metrics collection
│   └── app/           # Main application entry point
├── Cargo.toml         # Workspace configuration
└── README.md
```

## Crates

### core
Contains the core business logic and domain models that are independent of implementation details.

### hal
Provides hardware abstraction layer with trait definitions and device implementations for system-level interactions.

### telemetry
Handles logging, metrics collection, and observability features.

### app
The main application that orchestrates the other crates and contains the entry point.

## Building

```bash
cargo build
```

## Running

```bash
cargo run -p app
```

## Testing

```bash
cargo test
```

## Development

Each crate is a self-contained module that can be developed and tested independently. Cross-crate dependencies are defined in each crate's `Cargo.toml`.