#![allow(unused_variables)]
#![feature(box_syntax)]

use sledge::channels::parser::Channel;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    const MAYBE_CHANNEL: Option<Channel> = None;

    c.bench_function("process_kvs_with_ch", |b|
        b.iter(||{}
            
        ),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
