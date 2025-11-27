use criterion::{criterion_group, criterion_main, Criterion};
use telemetry::TelemetryPacket;

fn serialize_bench(c: &mut Criterion) {
    let mut packet = TelemetryPacket::new(42);
    // Add some sensor readings to increase serialization cost
    for i in 0..50 {
        packet.sensor_readings.push(telemetry::SensorReading::new(
            format!("sensor-{}", i),
            format!("Sensor {}", i),
            telemetry::SensorData::Temperature { value: 20.0 + i as f32 * 0.1, unit: "C".to_string() },
            i,
        ));
    }

    c.bench_function("telemetry_serialize_json", |b| {
        b.iter(|| {
            let _ = packet.to_json_bytes().unwrap();
        })
    });
}

criterion_group!(benches, serialize_bench);
criterion_main!(benches);
