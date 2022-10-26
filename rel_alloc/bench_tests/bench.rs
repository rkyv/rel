pub mod from_data;
pub mod gen;
mod log;
mod mc_savedata;
mod mesh;

use ::criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let mut bench = mesh::make_bench(&mut gen::default_rng(), 125_000);
    println!("populate_mesh           size:   {} bytes", bench());
    c.bench_function("populate_mesh", |b| {
        b.iter(&mut bench);
    });

    let mut bench = log::make_bench(&mut gen::default_rng(), 10_000);
    println!("populate_log            size:   {} bytes", bench());
    c.bench_function("populate_log", |b| {
        b.iter(&mut bench);
    });

    let mut bench = mc_savedata::make_bench(&mut gen::default_rng(), 500);
    println!("populate_mc_savedata    size:   {} bytes", bench());
    c.bench_function("populate_mc_savedata", |b| {
        b.iter(&mut bench);
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
