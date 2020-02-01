use criterion::{black_box, criterion_group, criterion_main, Criterion};

use z85::{rfc::Z85, padded::Z85p};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("rfc", |b| {
        let z = Z85::encode(&[0;32]).unwrap();
        b.iter(|| format!("{}", black_box(z.clone())))
    });

    c.bench_function("padded", |b| {
        let z = Z85p::encode(&[0;32]);
        b.iter(|| format!("{}", black_box(z.clone())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
