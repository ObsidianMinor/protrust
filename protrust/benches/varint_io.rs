use criterion::{Criterion, black_box, criterion_group, criterion_main};

use protrust::io::{CodedWriter, CodedReader};

macro_rules! add_write_group {
    ($g:ident, $n:expr, $f:ident, $v:expr) => {
        $g.bench_function($n, |b| {
            let mut output = [0u8; 10];
            b.iter(|| {
                let mut writer = CodedWriter::with_slice(&mut output);
                writer.$f(black_box($v)).unwrap();
            });
        });
    };
}

macro_rules! add_read_group {
    ($g:ident, $n:expr, $f:ident, $v:expr) => {
        $g.bench_function($n, |b| {
            b.iter(|| {
                let mut reader = CodedReader::with_slice(black_box($v));
                reader.$f().unwrap();
            });
        });
    };
}

fn write_varint32(c: &mut Criterion) {
    let mut group = c.benchmark_group("write-varint32");
    group.bench_function("0-byte", |b| {
        let mut output = [0u8; 10];
        b.iter(|| {
            let mut writer = CodedWriter::with_slice(&mut output);
            black_box(&mut writer);
            // do nothing as a baseline
        })
    });
    add_write_group!(group, "1-byte", write_varint32, 127);
    add_write_group!(group, "2-byte", write_varint32, 16_383);
    add_write_group!(group, "3-byte", write_varint32, 2_097_151);
    add_write_group!(group, "4-byte", write_varint32, 268_435_455);
    add_write_group!(group, "5-byte", write_varint32, u32::max_value());
    group.finish();
}

fn write_varint64(c: &mut Criterion) {
    let mut group = c.benchmark_group("write-varint64");
    add_write_group!(group, "1-byte", write_varint64, 127);
    add_write_group!(group, "10-byte", write_varint64, u64::max_value());
    group.finish();
}

fn read_varint32(c: &mut Criterion) {
    let mut group = c.benchmark_group("read-varint32");
    add_read_group!(group, "1-byte", read_varint32, &[0xFF]);
    add_read_group!(group, "2-byte", read_varint32, &[0x7F, 0xFF]);
    add_read_group!(group, "3-byte", read_varint32, &[0x7F, 0x7F, 0xFF]);
    add_read_group!(group, "4-byte", read_varint32, &[0x7F, 0x7F, 0x7F, 0xFF]);
    add_read_group!(group, "5-byte", read_varint32, &[0x7F, 0x7F, 0x7F, 0x7F, 0x8F]);
    group.finish();
}

fn read_varint64(c: &mut Criterion) {
    let mut group = c.benchmark_group("read-varint64");
    add_read_group!(group, "1-byte", read_varint64, &[0xFF]);
    add_read_group!(group, "10-byte", read_varint64, &[0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x81]);
    group.finish();
}

criterion_group!(benches, write_varint32, write_varint64, read_varint32, read_varint64);
criterion_main!(benches);