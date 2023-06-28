/// Example file which writes a reference TSV with two random sets and their exact cardinality,
/// and the estimated cardinality using HyperLogLog. The file can be used to benchmark the
/// accuracy of the HyperLogLog algorithm against other implementations. Of course, we need to run this
/// for multiple precisions and number of bits, which we will log as different rows in the TSV.
///
/// The TSV will have the following columns:
///
/// - `precision`: The precision of the HyperLogLog algorithm.
/// - `bits`: The number of bits used by the HyperLogLog algorithm.
/// - `exact`: The exact cardinality between the two sets.
/// - `hll`: The estimated cardinality using HyperLogLog.
/// - `seed`: The seed used to generate the two sets.
/// - `set1`: The first set, with values separated by commas
/// - `set2`: The second set, with values separated by commas
///
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

use hyperloglog_rs::prelude::*;

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

fn xorshift(mut x: u64) -> u64 {
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

fn write_line<PRECISION: Precision + WordType<BITS>, const BITS: usize>(
    set: &HashSet<u64>,
    set_str: &str,
    exact_cardinality: usize,
    file: &mut File,
) -> std::io::Result<()> {
    let hll: HyperLogLog<PRECISION, BITS> = set.iter().collect();

    let line = format!(
        "{}\t{}\t{}\t{}\t{}\n",
        PRECISION::EXPONENT,
        BITS,
        exact_cardinality,
        hll.estimate_cardinality(),
        set_str,
    );

    file.write_all(line.as_bytes())
}

#[test]
fn test_cardinality_perfs() {
    let mut file = File::create("cardinality_benchmark.tsv").unwrap();
    file.write_all(b"precision\tbits\texact\thll\tset\n")
        .unwrap();

    // since both the precision and the number of bits are compile time constants, we can
    // not iterate over the precision and bits, but we need to manually change them, making
    // the code a bit verbose:

    // precision 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
    // bits 4, 5, 6

    // For each precision and number of bits, we generate 1000 random sets and write them to the file.
    // We also write the exact cardinality and the estimated cardinality using HyperLogLog.
    for i in 0..10_u64 {
        let seed = (i + 1).wrapping_mul(234567898765);
        let mut rng = splitmix64(seed);

        let mut set = HashSet::new();

        for _ in 0..10_000_000 {
            let value = xorshift(rng) % 10_000_000;
            set.insert(value);
            rng = splitmix64(rng);
        }

        let exact = set.len();

        let set_str = set
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(",");

        write_line::<Precision4, 1>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision4, 2>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision4, 3>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision4, 4>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision4, 5>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision4, 6>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision5, 1>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision5, 2>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision5, 3>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision5, 4>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision5, 5>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision5, 6>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision6, 1>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision6, 2>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision6, 3>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision6, 4>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision6, 5>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision6, 6>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision7, 1>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision7, 2>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision7, 3>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision7, 4>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision7, 5>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision7, 6>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision8, 1>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision8, 2>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision8, 3>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision8, 4>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision8, 5>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision8, 6>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision9, 1>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision9, 2>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision9, 3>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision9, 4>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision9, 5>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision9, 6>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision10, 1>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision10, 2>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision10, 3>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision10, 4>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision10, 5>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision10, 6>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision11, 1>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision11, 2>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision11, 3>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision11, 4>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision11, 5>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision11, 6>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision12, 1>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision12, 2>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision12, 3>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision12, 4>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision12, 5>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision12, 6>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision13, 1>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision13, 2>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision13, 3>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision13, 4>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision13, 5>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision13, 6>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision14, 1>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision14, 2>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision14, 3>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision14, 4>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision14, 5>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision14, 6>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision15, 1>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision15, 2>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision15, 3>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision15, 4>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision15, 5>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision15, 6>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision16, 1>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision16, 2>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision16, 3>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision16, 4>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision16, 5>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision16, 6>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision17, 1>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision17, 2>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision17, 3>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision17, 4>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision17, 5>(&set, &set_str, exact, &mut file).unwrap();
        write_line::<Precision17, 6>(&set, &set_str, exact, &mut file).unwrap();
        // write_line::<Precision18, 1>(&set, &set_str, exact, &mut file).unwrap();
        // write_line::<Precision18, 2>(&set, &set_str, exact, &mut file).unwrap();
        // write_line::<Precision18, 3>(&set, &set_str, exact, &mut file).unwrap();
        // write_line::<Precision18, 4>(&set, &set_str, exact, &mut file).unwrap();
        // write_line::<Precision18, 5>(&set, &set_str, exact, &mut file).unwrap();
        // write_line::<Precision18, 6>(&set, &set_str, exact, &mut file).unwrap();
    }
}
