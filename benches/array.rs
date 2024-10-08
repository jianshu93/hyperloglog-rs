//! Benchmark for the methods of the array data structure.
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::*;

const PRECISION: usize = 15;
const REGISTER_SIZE: usize = 6;
const NUMBER_OF_REGISTERS: usize = 1 << PRECISION;
const NUMBER_OF_REGISTERS_IN_U64: usize = 64 / REGISTER_SIZE;
const PADDED_SIZE: usize = ceil(NUMBER_OF_REGISTERS, NUMBER_OF_REGISTERS_IN_U64);
const PACKED_SIZE: usize = ceil(NUMBER_OF_REGISTERS * REGISTER_SIZE, 64);

fn bench_array(c: &mut Criterion) {
    let mut group = c.benchmark_group("array");

    group.bench_function("array_insert", |b| {
        b.iter(|| {
            let mut left = 0;
            let mut right = 0;
            let mut array: Array<PADDED_SIZE, false, Bits6> = Array::default();
            for i in 0..NUMBER_OF_REGISTERS {
                for value in 0..64 {
                    let (l, r) = array.set_apply(black_box(i), black_box(|x: u8| x.max(value)));
                    left ^= l;
                    right ^= r;
                }
            }
            (left, right)
        });
    });

    group.bench_function("packed_insert", |b| {
        b.iter(|| {
            let mut left = 0;
            let mut right = 0;
            let mut packed: Array<PACKED_SIZE, true, Bits6> = Array::default();
            for i in 0..NUMBER_OF_REGISTERS {
                for value in 0..64 {
                    let (l, r) = packed.set_apply(black_box(i), black_box(|x: u8| x.max(value)));
                    left ^= l;
                    right ^= r;
                }
            }
            (left, right)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_array);

criterion_main!(benches);