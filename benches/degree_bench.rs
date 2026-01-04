use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use degree_rs::DegreePlugin;
use std::hint::black_box;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_test_matrix(size: usize) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();

    // Write header
    write!(file, "\"\"").unwrap();
    for i in 0..size {
        write!(file, ",\"Node{}\"", i).unwrap();
    }
    writeln!(file).unwrap();

    // Write data rows
    for i in 0..size {
        write!(file, "\"Node{}\"", i).unwrap();
        for j in 0..size {
            let weight = if i == j { 1.0 } else { 0.5 };
            write!(file, ",{}", weight).unwrap();
        }
        writeln!(file).unwrap();
    }

    file
}

fn bench_run(c: &mut Criterion) {
    let mut group = c.benchmark_group("degree_run");

    for size in [10, 50, 100, 200].iter() {
        let file = create_test_matrix(*size);
        let mut plugin = DegreePlugin::new();
        plugin.input(file.path()).unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                plugin.run();
                let result = plugin.get_centrality().len();
                black_box(result)
            })
        });
    }

    group.finish();
}

fn bench_input(c: &mut Criterion) {
    let mut group = c.benchmark_group("degree_input");

    for size in [10, 50, 100, 200].iter() {
        let file = create_test_matrix(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut plugin = DegreePlugin::new();
                plugin.input(black_box(file.path())).unwrap();
                black_box(plugin.size())
            })
        });
    }

    group.finish();
}

fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("degree_full");

    for size in [10, 50, 100, 200].iter() {
        let file = create_test_matrix(*size);
        let output = NamedTempFile::new().unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut plugin = DegreePlugin::new();
                plugin.input(black_box(file.path())).unwrap();
                plugin.run();
                plugin.output(black_box(output.path())).unwrap();
                black_box(plugin.size())
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_run, bench_input, bench_full_pipeline);
criterion_main!(benches);
