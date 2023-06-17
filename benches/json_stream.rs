#![allow(dead_code)]

use std::iter::repeat_with;
use criterion::{black_box, Criterion, criterion_group, criterion_main};
use futures::StreamExt;
use serde::Deserialize;
use json_stream::JsonStream;
use futures::stream::{iter, once};

#[derive(Deserialize)]
struct Entry {
    bar: String
}

pub fn json_ingest(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // measures ingesting 1 small json object
    c.bench_function("ingest single entry", |b| {
        b.to_async(&rt).iter(|| async {
            let json_stream = JsonStream::<Entry, _>::new(once(Box::pin(async {
                Ok::<_, std::io::Error>("{\"bar\": \"foo\"}\n".as_bytes())
            })));

            let _ = json_stream.collect::<Vec<_>>().await;
        });
    });

    // measures ingesting 1 million small json objects, at 15 bytes each
    c.bench_function("ingest 1 million entries", |b| {
        let raw_json_sequence = "{\"bar\": \"foo\"}\n"
            .as_bytes()
            .to_vec();

        let large_data = repeat_with(|| raw_json_sequence.clone())
            .take(1_000_000)
            .flatten()
            .collect::<Vec<u8>>();

        let large_chunks = large_data
            // default chunk size of BodyStream from axum - seems like a reasonable default
            .chunks(400 * 1024)
            .map(|chunk| Ok::<_, std::io::Error>(chunk));

        let large_stream = iter(large_chunks);

        b.to_async(&rt).iter(|| async {
            JsonStream::<Entry, _>::new(large_stream.clone())
                .for_each(|entry| async {
                    let _ = black_box(entry);
                })
                .await;
        });
    });
}

criterion_group!(benches, json_ingest);
criterion_main!(benches);