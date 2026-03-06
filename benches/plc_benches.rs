use criterion::{criterion_group, criterion_main, Criterion};
use omron_fins::{Client, ClientConfig, MemoryArea};
use std::net::Ipv4Addr;
use std::time::Duration;

fn create_client_with_node(node: u8) -> Option<Client> {
    let config = ClientConfig::new(Ipv4Addr::new(192, 168, 250, 5), node, 5)
        .with_timeout(Duration::from_secs(5));
    Client::new(config).ok()
}

fn create_client() -> Option<Client> {
    create_client_with_node(10)
}

fn bench_plc_reads(c: &mut Criterion) {
    let client = match create_client() {
        Some(client) => client,
        None => {
            // Cannot connect to PLC from benchmark runner; skipping actual measurements
            eprintln!("WARNING: Could not connect to PLC at 192.168.250.5 for reading benchmarks.");
            return;
        }
    };

    let mut group = c.benchmark_group("PLC Reads");
    // Using a sample size of 10 since network requests can be slow and varying
    group.sample_size(10);

    group.bench_function("read_single_word_DM", |b| {
        b.iter(|| {
            client.read(MemoryArea::DM, 0, 1).unwrap();
        })
    });

    group.bench_function("read_512_words_DM", |b| {
        b.iter(|| {
            client.read(MemoryArea::DM, 0, 512).unwrap();
        })
    });

    group.bench_function("read_4096_words_DM", |b| {
        b.iter(|| {
            client.read(MemoryArea::DM, 0, 4096).unwrap();
        })
    });

    group.bench_function("read_4096_words_CIO", |b| {
        b.iter(|| {
            client.read(MemoryArea::CIO, 0, 4096).unwrap();
        })
    });

    group.bench_function("read_512_words_WR", |b| {
        b.iter(|| {
            client.read(MemoryArea::WR, 0, 512).unwrap();
        })
    });

    group.bench_function("read_512_words_HR", |b| {
        b.iter(|| {
            client.read(MemoryArea::HR, 0, 512).unwrap();
        })
    });

    group.finish();
}

fn bench_plc_writes(c: &mut Criterion) {
    let client = match create_client() {
        Some(client) => client,
        None => {
            eprintln!("WARNING: Could not connect to PLC at 192.168.250.5 for writing benchmarks.");
            return;
        }
    };

    let mut group = c.benchmark_group("PLC Writes");
    group.sample_size(10);

    let words_512 = vec![0u16; 512];
    group.bench_function("write_512_words_DM", |b| {
        b.iter(|| {
            client.write(MemoryArea::DM, 1000, &words_512).unwrap();
        })
    });

    group.finish();
}

fn bench_concurrent_clients(c: &mut Criterion) {
    let edge_client = match create_client_with_node(10) {
        Some(client) => client,
        None => {
            eprintln!(
                "WARNING: Could not connect to PLC at 192.168.250.5 for concurrent benchmarks."
            );
            return;
        }
    };

    // Create multiple HMIs (they shouldn't fail if the Edge didn't)
    let hmi1_client = create_client_with_node(11).unwrap();
    let hmi2_client = create_client_with_node(12).unwrap();

    let mut group = c.benchmark_group("PLC Concurrent Reads");
    group.sample_size(10);

    group.bench_function("1_edge_1_hmi_reading_512_DM", |b| {
        b.iter(|| {
            std::thread::scope(|s| {
                s.spawn(|| {
                    let _ = edge_client.read(MemoryArea::DM, 0, 512);
                });
                s.spawn(|| {
                    let _ = hmi1_client.read(MemoryArea::DM, 1000, 512);
                });
            });
        })
    });

    group.bench_function("1_edge_2_hmis_reading_512_DM", |b| {
        b.iter(|| {
            std::thread::scope(|s| {
                s.spawn(|| {
                    let _ = edge_client.read(MemoryArea::DM, 0, 512);
                });
                s.spawn(|| {
                    let _ = hmi1_client.read(MemoryArea::DM, 1000, 512);
                });
                s.spawn(|| {
                    let _ = hmi2_client.read(MemoryArea::DM, 2000, 512);
                });
            });
        })
    });

    group.bench_function("1_edge_2_hmis_reading_4096_DM", |b| {
        b.iter(|| {
            std::thread::scope(|s| {
                s.spawn(|| {
                    let _ = edge_client.read(MemoryArea::DM, 0, 4096);
                });
                s.spawn(|| {
                    let _ = hmi1_client.read(MemoryArea::DM, 5000, 4096);
                });
                s.spawn(|| {
                    let _ = hmi2_client.read(MemoryArea::DM, 10000, 4096);
                });
            });
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_plc_reads,
    bench_plc_writes,
    bench_concurrent_clients
);
criterion_main!(benches);
