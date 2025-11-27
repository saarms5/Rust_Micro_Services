#![no_main]
use libfuzzer_sys::fuzz_target;

use telemetry::TelemetryPacket;

fuzz_target!(|data: &[u8]| {
    // Try to parse as utf8 string first
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = TelemetryPacket::from_json(s);
    }
    // Also attempt to parse from bytes (serde_json accepts &[u8])
    let _ = TelemetryPacket::from_json_bytes(data);
});
