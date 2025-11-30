use std::path::PathBuf;
use std::time::{Duration, Instant};
use telemetry::{PipelineConfig, StreamingPipeline, SystemHealth, TelemetryPacket};

#[tokio::test]
async fn e2e_pipeline_runs_and_sends() {
    // Shorter duration for CI; use 60s when running locally if desired.
    let run_duration = Duration::from_secs(5);

    let config = PipelineConfig {
        batch_size: 5,
        batch_timeout_secs: 2,
        enable_compression: false,
        enable_resilience: true,
        channel_capacity: 1024,
    };

    // Use a temporary file under target/test_output
    let out_path = PathBuf::from("target/test_output/e2e_mqtt.log");

    // Create a real MqttTransport wrapped in PipelineTransport
    let mqtt = telemetry::MqttTransport::new(Some(out_path.clone()))
        .await
        .expect("mqtt transport");
    let transports = vec![telemetry::streaming::PipelineTransport::Mqtt(mqtt)];

    let pipeline = StreamingPipeline::new(config, transports)
        .await
        .expect("pipeline");
    let sender = pipeline.get_sender();

    // Spawn a producer that generates telemetry for `run_duration`.
    let producer = tokio::spawn(async move {
        let start = Instant::now();
        let mut seq: u64 = 0;
        while Instant::now() - start < run_duration {
            let packet = TelemetryPacket {
                sequence: seq,
                timestamp: chrono::Utc::now(),
                health: SystemHealth::new(),
                sensor_readings: vec![],
                diagnostics: Default::default(),
            };
            if let Err(_) = sender.send(packet).await {
                break;
            }
            seq += 1;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    // Wait for producer to finish
    producer.await.expect("producer panicked");

    // Allow pipeline to flush
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Check output file exists and has content
    let content = tokio::fs::read_to_string(out_path)
        .await
        .expect("read out file");
    assert!(
        !content.trim().is_empty(),
        "Expected telemetry output in file"
    );
}
