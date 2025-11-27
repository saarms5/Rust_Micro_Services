use std::time::Duration;

use telemetry::TelemetryPacket;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Demo simulator: sending telemetry to http://127.0.0.1:3030/telemetry");

    for i in 0..10u32 {
        let mut pkt = TelemetryPacket::new(i as u64);
        // attach a simple diagnostic entry to carry demo info
        let mut diag = pkt.diagnostics.clone();
        diag.add_entry(telemetry::DiagnosticEntry::new(
            telemetry::DiagnosticLevel::Info,
            format!("sim-{}", i),
            format!("demo counter={}", i),
        ));
        pkt.diagnostics = diag;

        let client = reqwest::Client::new();
        let res = client
            .post("http://127.0.0.1:3030/telemetry")
            .json(&pkt)
            .send()
            .await;

        match res {
            Ok(r) => println!("Sent telemetry {}: status {}", i, r.status()),
            Err(e) => eprintln!("Failed to send telemetry {}: {}", i, e),
        }

        sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}
