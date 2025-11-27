Fuzzing telemetry parser with libFuzzer

Steps to run locally:

1. Install `cargo-fuzz` and required toolchain components:

```powershell
rustup component add llvm-tools-preview
cargo install cargo-fuzz
```

2. Build and run the fuzz target (example, short run):

```powershell
cd fuzz
cargo run --bin telemetry_parser
# For real fuzzing with libFuzzer you would use cargo-fuzz tooling; see cargo-fuzz docs.
```

Notes:
- The fuzz target calls `TelemetryPacket::from_json` and `from_json_bytes` with arbitrary data to ensure no panics.
