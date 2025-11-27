use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;

// Simulate component context switching by spawning lightweight async tasks.
fn context_switch_bench(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    c.bench_function("context_switch_spawn_1000", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = Vec::with_capacity(1000);
                for _ in 0..1000 {
                    handles.push(tokio::spawn(async { 1u64 + 1 }));
                }
                for h in handles {
                    let _ = h.await;
                }
            })
        })
    });
}

criterion_group!(benches, context_switch_bench);
criterion_main!(benches);
