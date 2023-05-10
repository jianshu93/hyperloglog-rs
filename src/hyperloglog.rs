use crate::prelude::*;
use core::hash::{Hash, Hasher};
use core::ops::{BitOr, BitOrAssign};
use std::collections::hash_map::DefaultHasher;

#[derive(Clone, Debug, Eq, PartialEq)]
/// HyperLogLog is a probabilistic algorithm for estimating the number of distinct elements in a set.
/// It uses a small amount of memory to produce an approximate count with a guaranteed error rate.
pub struct HyperLogLog<const PRECISION: usize, const BITS: usize>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    words: [u32; ceil(1 << PRECISION, 32 / BITS)],
    number_of_zero_register: u16,
}

impl<const PRECISION: usize, const BITS: usize, T: Hash> From<T> for HyperLogLog<PRECISION, BITS>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
    [(); 1 << PRECISION]:,
{
    fn from(value: T) -> Self {
        let mut hll = Self::new();
        hll.insert(value);
        hll
    }
}

impl<const PRECISION: usize, const BITS: usize> HyperLogLog<PRECISION, BITS>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
    [(); 1 << PRECISION]:,
{
    pub const NUMBER_OF_REGISTERS: usize = 1 << PRECISION;
    pub const NUMBER_OF_REGISTERS_SQUARED: f32 =
        (Self::NUMBER_OF_REGISTERS * Self::NUMBER_OF_REGISTERS) as f32;
    pub const SMALL_RANGE_CORRECTION_THRESHOLD: f32 = 2.5_f32 * (Self::NUMBER_OF_REGISTERS as f32);
    pub const TWO_32: f32 = (1u64 << 32) as f32;
    pub const INTERMEDIATE_RANGE_CORRECTION_THRESHOLD: f32 = Self::TWO_32 / 30.0_f32;
    pub const ALPHA: f32 = get_alpha(1 << PRECISION);
    pub const LOWER_REGISTER_MASK: u32 = (1 << BITS) - 1;
    pub const NUMBER_OF_REGISTERS_IN_WORD: usize = 32 / BITS;

    /// Create a new HyperLogLog counter.
    pub fn new() -> Self {
        assert!(PRECISION >= 4);
        assert!(PRECISION <= 16);
        Self {
            words: [0; ceil(1 << PRECISION, 32 / BITS)],
            number_of_zero_register: 1 << PRECISION,
        }
    }

    /// Create a new HyperLogLog counter.
    pub fn from_registers(registers: [u32; 1 << PRECISION]) -> Self {
        let mut words = [0; ceil(1 << PRECISION, 32 / BITS)];
        let number_of_zero_register = words
            .iter_mut()
            .zip(registers.chunks(Self::NUMBER_OF_REGISTERS_IN_WORD))
            .fold(0, |mut number_of_zero_register, (word, word_registers)| {
                number_of_zero_register += word_registers
                    .iter()
                    .filter(|&&register| register == 0)
                    .count() as u16;
                *word = to_word::<BITS>(&word_registers);
                number_of_zero_register
            });
        Self {
            words,
            number_of_zero_register,
        }
    }

    pub fn estimate_cardinality(&self) -> f32 {
        let mut raw_estimate: f32 = dispatch_specialized_count::<
            { ceil(1 << PRECISION, 32 / BITS) },
            PRECISION,
            { 32 / BITS },
        >(&self.words);

        // Apply the final scaling factor to obtain the estimate of the cardinality
        raw_estimate = Self::ALPHA * Self::NUMBER_OF_REGISTERS_SQUARED / raw_estimate;

        if raw_estimate <= Self::SMALL_RANGE_CORRECTION_THRESHOLD
            && self.number_of_zero_register > 0
        {
            get_small_correction_lookup_table::<{ 1 << PRECISION }>(
                self.number_of_zero_register as usize,
            )
        } else if raw_estimate >= Self::INTERMEDIATE_RANGE_CORRECTION_THRESHOLD {
            -Self::TWO_32 * (-raw_estimate / Self::TWO_32).ln_1p()
        } else {
            raw_estimate
        }
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        debug_assert_eq!(
            self.words.len(),
            ceil(1 << PRECISION, Self::NUMBER_OF_REGISTERS_IN_WORD)
        );

        self.words
            .iter()
            .copied()
            .flat_map(|six_registers| {
                (0..Self::NUMBER_OF_REGISTERS_IN_WORD)
                    .map(move |i| six_registers >> i * BITS & Self::LOWER_REGISTER_MASK)
            })
            .take(Self::NUMBER_OF_REGISTERS)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        debug_assert_eq!(Self::NUMBER_OF_REGISTERS, self.iter().count());
        Self::NUMBER_OF_REGISTERS
    }

    #[inline(always)]
    pub fn get_number_of_bits(&self) -> usize {
        PRECISION
    }

    #[inline(always)]
    pub fn get_number_of_zero_registers(&self) -> usize {
        self.iter()
            .filter(|&register_value| register_value == 0)
            .count()
    }

    #[inline(always)]
    pub fn get_number_of_non_zero_registers(&self) -> usize {
        self.iter()
            .filter(|&register_value| register_value > 0)
            .count()
    }

    #[inline(always)]
    pub fn get_registers(&self) -> [u32; 1 << PRECISION] {
        let mut array = [0; (1 << PRECISION)];
        self.iter()
            .zip(array.iter_mut())
            .for_each(|(value, target)| {
                *target = value;
            });
        array
    }

    #[inline(always)]
    /// Adds an element to the HyperLogLog counter.
    ///
    /// # Arguments
    /// * `rhs` - The element to add.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// const PRECISION: usize = 10;
    ///
    /// let mut hll = HyperLogLog::<PRECISION, 6>::new();
    ///
    /// hll.insert("Hello");
    /// hll.insert("World");
    ///
    /// assert!(hll.estimate_cardinality() >= 2.0);
    /// ```
    ///
    /// # Performance
    ///
    /// The performance of this function depends on the size of the HyperLogLog counter (`N`), the number
    /// of distinct elements in the input, and the hash function used to hash elements. For a given value of `N`,
    /// the function has an average time complexity of O(1) and a worst-case time complexity of O(log N).
    /// However, the actual time complexity may vary depending on the distribution of the hashed elements.
    ///
    /// # Errors
    ///
    /// This function does not return any errors.
    pub fn insert<T: Hash>(&mut self, rhs: T) {
        // Create a new hasher.
        let mut hasher = DefaultHasher::new();
        // Calculate the hash.
        rhs.hash(&mut hasher);
        // Drops the higher 32 bits.
        let mut hash: u32 = hasher.finish() as u32;

        // Calculate the register's index.
        let index: usize = (hash >> (32 - PRECISION)) as usize;
        debug_assert!(
            index < Self::NUMBER_OF_REGISTERS,
            "The index {} must be less than the number of registers {}.",
            index,
            Self::NUMBER_OF_REGISTERS
        );

        // Shift left the bits of the index.
        hash = (hash << PRECISION) | (1 << (PRECISION - 1));

        // Count leading zeros.
        let number_of_zeros: u32 = 1 + hash.leading_zeros();

        // Calculate the position of the register in the internal buffer array.
        let register_position_in_array = index / Self::NUMBER_OF_REGISTERS_IN_WORD;

        debug_assert!(
            register_position_in_array < self.words.len(),
            concat!(
                "The register_position_in_array {} must be less than the number of words {}. ",
                "You have obtained this values starting from the index {} and the word size {}."
            ),
            register_position_in_array,
            self.words.len(),
            index,
            Self::NUMBER_OF_REGISTERS_IN_WORD
        );

        // Calculate the position of the register within the 32-bit word containing it.
        let register_position_in_u32 = index % Self::NUMBER_OF_REGISTERS_IN_WORD;

        // Extract the current value of the register at `index`.
        let register_value: u32 = (self.words[register_position_in_array]
            >> (register_position_in_u32 * BITS))
            & Self::LOWER_REGISTER_MASK;

        // If `number_of_zeros` is greater than the current number_of_zeros, update the register.
        if number_of_zeros > register_value {
            let shifted_zeros = number_of_zeros << (register_position_in_u32 * BITS);
            if register_value == 0 {
                self.number_of_zero_register -= 1;
                // If the current number_of_zeros is zero, decrement `zeros` and set the register to `number_of_zeros`.
                self.words[register_position_in_array] |= shifted_zeros;
            } else {
                // Otherwise, update the register using a bit mask.
                let mask = Self::LOWER_REGISTER_MASK << (register_position_in_u32 * BITS);
                self.words[register_position_in_array] =
                    (self.words[register_position_in_array] & !mask) | shifted_zeros;
            }
        }
    }
}

impl<const PRECISION: usize, const BITS: usize> BitOrAssign for HyperLogLog<PRECISION, BITS>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    #[inline(always)]
    /// Computes union between HLL counters.
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    /// # use core::ops::BitOrAssign;
    ///
    /// let mut hll = HyperLogLog::<8, 6>::new();
    /// hll.insert(1u8);
    ///
    /// let mut hll2 = HyperLogLog::<8, 6>::new();
    /// hll2.insert(2u8);
    ///
    /// hll.bitor_assign(hll2);
    ///
    /// assert!(hll.estimate_cardinality() > 2.0 - 0.1);
    /// assert!(hll.estimate_cardinality() < 2.0 + 0.1);
    ///
    /// let mut hll = HyperLogLog::<8, 6>::new();
    /// hll.insert(1u8);
    ///
    /// let mut hll2 = HyperLogLog::<8, 6>::new();
    /// hll2.insert(1u8);
    ///
    /// hll.bitor_assign(hll2);
    ///
    /// assert!(hll.estimate_cardinality() > 1.0 - 0.1);
    /// assert!(hll.estimate_cardinality() < 1.0 + 0.1);
    ///
    /// let mut hll3 = HyperLogLog::<16, 6>::new();
    /// hll3.insert(3u8);
    /// hll3.insert(4u8);
    ///
    /// let mut hll4 = HyperLogLog::<16, 6>::new();
    /// hll4.insert(5u8);
    /// hll4.insert(6u8);
    ///
    /// hll3.bitor_assign(hll4);
    ///
    /// assert!(hll3.estimate_cardinality() > 4.0 - 0.1, "Expected a value equal to around 4, got {}", hll3.estimate_cardinality());
    /// assert!(hll3.estimate_cardinality() < 4.0 + 0.1, "Expected a value equal to around 4, got {}", hll3.estimate_cardinality());
    /// ```
    fn bitor_assign(&mut self, rhs: Self) {
        for (left_word, right_word) in self.words.iter_mut().zip(rhs.words.iter().copied()) {
            let mut left_registers = split_registers::<{ 32 / BITS }>(*left_word);
            let right_registers = split_registers::<{ 32 / BITS }>(right_word);

            left_registers
                .iter_mut()
                .zip(right_registers.into_iter())
                .for_each(|(left, right)| {
                    *left = (*left).max(right);
                });

            *left_word = to_word::<BITS>(&left_registers)
        }
    }
}

impl<const PRECISION: usize, const BITS: usize> BitOr for HyperLogLog<PRECISION, BITS>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    type Output = Self;

    #[inline(always)]
    /// Computes union between HLL counters.
    fn bitor(mut self, rhs: Self) -> Self {
        self.bitor_assign(rhs);
        self
    }
}
