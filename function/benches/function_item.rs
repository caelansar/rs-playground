use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[inline(never)]
pub fn calculate<F>(f: F) -> u128
where
    F: FnMut(u128) -> u128,
    F: Copy,
{
    (0..1000000).map(f).map(f).map(f).map(f).map(f).sum()
}

#[inline(never)]
pub fn calculate1<F>(f: F) -> u128
where
    F: FnMut(u128) -> u128,
    F: Copy,
{
    (0..100000000000).map(f).map(f).map(f).map(f).map(f).sum()
}

#[inline(never)]
pub fn calculate_pointer(f: fn(u128) -> u128) -> u128 {
    (0..1000).map(f).map(f).map(f).map(f).map(f).sum()
}

pub fn f(x: u128) -> u128 {
    x + x
}

pub fn f1(x: u128) -> u128 {
    x * 2
}

pub fn call() -> u128 {
    calculate(f);

    calculate(f1)
}

pub fn call1() -> u128 {
    calculate1(f);

    calculate1(f1)
}

pub fn call_pointer() -> u128 {
    let mut fp: fn(u128) -> u128 = f;
    calculate_pointer(fp);

    fp = f1;
    calculate_pointer(fp)
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("func item 1000000", |b| {
        b.iter(|| {
            black_box(for _ in 1..=100 {
                black_box(call());
            })
        })
    });

    c.bench_function("func item 100000000000", |b| {
        b.iter(|| {
            black_box(for _ in 1..=100 {
                black_box(call1());
            })
        })
    });

    c.bench_function("func pointer 1000", |b| {
        b.iter(|| {
            black_box(for _ in 1..=100 {
                black_box(call_pointer());
            })
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
