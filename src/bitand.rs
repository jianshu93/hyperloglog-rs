use crate::primitive::Primitive;
use crate::{array_default::ArrayIter, prelude::*};
use core::ops::{BitAnd, BitAndAssign};

#[allow(clippy::suspicious_op_assign_impl)]
impl<PRECISION: Precision + WordType<BITS>, const BITS: usize, M: HasherMethod> BitAndAssign<Self>
    for HyperLogLog<PRECISION, BITS, M>
{
    #[inline(always)]
    /// Computes intersection between HLL counters.
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    /// # use core::ops::BitAndAssign;
    ///
    /// let mut hll = HyperLogLog::<Precision8, 6>::new();
    /// hll.insert(1u8);
    ///
    /// let mut hll2 = HyperLogLog::<Precision8, 6>::new();
    /// hll2.insert(2u8);
    ///
    /// hll.bitand_assign(hll2);
    ///
    /// assert!(hll.estimate_cardinality() < 0.1, "The cardinality is {}, we were expecting 0.", hll.estimate_cardinality());
    ///
    /// let mut hll = HyperLogLog::<Precision8, 6>::new();
    /// hll.insert(1u8);
    ///
    /// let mut hll2 = HyperLogLog::<Precision8, 6>::new();
    /// hll2.insert(1u8);
    ///
    /// hll.bitand_assign(hll2);
    ///
    /// assert!(hll.estimate_cardinality() > 1.0 - 0.1, "The cardinality is {}, we were expecting 1.", hll.estimate_cardinality());
    /// assert!(hll.estimate_cardinality() < 1.0 + 0.1, "The cardinality is {}, we were expecting 1.", hll.estimate_cardinality());
    ///
    /// let mut hll3 = HyperLogLog::<Precision16, 6>::new();
    /// hll3.insert(3u8);
    /// hll3.insert(5u8);
    ///
    /// let mut hll4 = HyperLogLog::<Precision16, 6>::new();
    /// hll4.insert(5u8);
    /// hll4.insert(6u8);
    ///
    /// hll3.bitand_assign(hll4);
    ///
    /// assert!(hll3.estimate_cardinality() > 1.0 - 0.1, "Expected a value equal to around 1, got {}", hll3.estimate_cardinality());
    /// assert!(hll3.estimate_cardinality() < 1.0 + 0.1, "Expected a value equal to around 1, got {}", hll3.estimate_cardinality());
    /// ```
    fn bitand_assign(&mut self, rhs: Self) {
        self.bitand_assign(&rhs)
    }
}

#[allow(clippy::suspicious_op_assign_impl)]
impl<PRECISION: Precision + WordType<BITS>, const BITS: usize, M: HasherMethod> BitAndAssign<&Self>
    for HyperLogLog<PRECISION, BITS, M>
{
    #[inline(always)]
    /// Computes intersection between HLL counters.
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    /// # use core::ops::BitAndAssign;
    ///
    /// let mut hll = HyperLogLog::<Precision8, 6>::new();
    /// hll.insert(1u8);
    ///
    /// let mut hll2 = HyperLogLog::<Precision8, 6>::new();
    /// hll2.insert(2u8);
    ///
    /// hll.bitand_assign(&hll2);
    ///
    /// assert!(hll.estimate_cardinality() < 0.1, "The cardinality is {}, we were expecting 0.", hll.estimate_cardinality());
    ///
    /// let mut hll = HyperLogLog::<Precision8, 6>::new();
    /// hll.insert(1u8);
    ///
    /// let mut hll2 = HyperLogLog::<Precision8, 6>::new();
    /// hll2.insert(1u8);
    ///
    /// hll.bitand_assign(&hll2);
    ///
    /// assert!(hll.estimate_cardinality() > 1.0 - 0.1, "The cardinality is {}, we were expecting 1.", hll.estimate_cardinality());
    /// assert!(hll.estimate_cardinality() < 1.0 + 0.1, "The cardinality is {}, we were expecting 1.", hll.estimate_cardinality());
    ///
    /// let mut hll3 = HyperLogLog::<Precision16, 6>::new();
    /// hll3.insert(3u8);
    /// hll3.insert(5u8);
    /// hll3.insert(6u8);
    ///
    /// let mut hll4 = HyperLogLog::<Precision16, 6>::new();
    /// hll4.insert(5u8);
    /// hll4.insert(6u8);
    ///
    /// hll3.bitand_assign(&hll4);
    ///
    /// assert!(hll3.estimate_cardinality() > 2.0 - 0.1, "Expected a value equal to around 2, got {}", hll3.estimate_cardinality());
    /// assert!(hll3.estimate_cardinality() < 2.0 + 0.1, "Expected a value equal to around 2, got {}", hll3.estimate_cardinality());
    /// ```
    fn bitand_assign(&mut self, rhs: &Self) {
        self.multeplicities.reset();
        for (left_word, mut right_word) in self
            .words
            .iter_elements_mut()
            .zip(rhs.words.into_iter_elements())
        {
            let mut left_word_copy = *left_word;

            for i in 0..Self::NUMBER_OF_REGISTERS_IN_WORD {
                let mut left_register = left_word_copy & Self::LOWER_REGISTER_MASK;
                let right_register = right_word & Self::LOWER_REGISTER_MASK;
                left_register = (left_register).min(right_register);
                *left_word &= !(Self::LOWER_REGISTER_MASK << (i * BITS));
                *left_word |= left_register << (i * BITS);
                self.multeplicities[left_register as usize] += PRECISION::NumberOfZeros::ONE;
                left_word_copy >>= BITS;
                right_word >>= BITS;
            }
        }

        self.multeplicities[0] -=
            PRECISION::NumberOfZeros::reverse(Self::get_number_of_padding_registers());
    }
}

impl<PRECISION: Precision + WordType<BITS>, const BITS: usize, M: HasherMethod> BitAnd<Self>
    for HyperLogLog<PRECISION, BITS, M>
{
    type Output = Self;

    #[inline(always)]
    /// Computes the intersection between two HyperLogLog counters of the same precision and number of bits per register.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    /// let mut hll1 = HyperLogLog::<Precision14, 5>::new();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    ///
    /// let mut hll2 = HyperLogLog::<Precision14, 5>::new();
    /// hll2.insert(&2);
    /// hll2.insert(&3);
    ///
    /// let hll_intersection = hll1 & hll2;
    ///
    /// assert!(hll_intersection.estimate_cardinality() >= 1.0_f32 * 0.9 &&
    ///         hll_intersection.estimate_cardinality() <= 1.0_f32 * 1.1);
    /// ```
    ///
    /// Executing the intersection between a set and an empty set
    /// should result in an empty set.
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    /// let mut hll1 = HyperLogLog::<Precision14, 5>::new();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    ///
    /// let hll_intersection = hll1.clone() & HyperLogLog::<Precision14, 5>::new();
    /// assert_eq!(
    ///     HyperLogLog::<Precision14, 5>::new(),
    ///     hll_intersection,
    ///     concat!(
    ///         "The cardinality of the intersection should ",
    ///         "be the same as the empty test."
    ///    )
    /// );
    /// ```
    ///
    /// We can create the HLL counters from array from registers,
    /// so to be able to check that everything works as expected.
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let first_registers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    /// let second_registers = [9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 19];
    /// let expected = [9, 9, 9, 9, 9, 9, 9, 9, 9, 10, 11, 12, 13, 14, 15, 19];
    ///
    /// let mut hll1 = HyperLogLog::<Precision4, 5>::from_registers(&first_registers);
    /// let mut hll2 = HyperLogLog::<Precision4, 5>::from_registers(&second_registers);
    /// let intersection = hll1 | hll2;
    ///
    /// assert_eq!(intersection.get_registers(), expected, "The registers are not the expected ones, got {:?} instead of {:?}.", intersection.get_registers(), expected);
    /// ```
    ///
    fn bitand(mut self, rhs: Self) -> Self {
        self.bitand_assign(rhs);
        self
    }
}

impl<PRECISION: Precision + WordType<BITS>, const BITS: usize, M: HasherMethod> BitAnd<&Self>
    for HyperLogLog<PRECISION, BITS, M>
{
    type Output = Self;

    #[inline(always)]
    /// Computes the intersection between two HyperLogLog counters of the same precision and number of bits per register.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    /// let mut hll1 = HyperLogLog::<Precision14, 5>::new();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    ///
    /// let mut hll2 = HyperLogLog::<Precision14, 5>::new();
    /// hll2.insert(&2);
    /// hll2.insert(&3);
    ///
    /// let hll_intersection = hll1 | hll2;
    ///
    /// assert!(hll_intersection.estimate_cardinality() >= 3.0_f32 * 0.9 &&
    ///         hll_intersection.estimate_cardinality() <= 3.0_f32 * 1.1);
    /// ```
    ///
    /// Merging a set with an empty set should not change the cardinality.
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    /// let mut hll1 = HyperLogLog::<Precision14, 5>::new();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    ///
    /// let hll_intersection = hll1.clone() | HyperLogLog::<Precision14, 5>::new();
    /// assert_eq!(
    ///     hll_intersection,
    ///     hll1,
    ///     concat!(
    ///         "The cardinality of the intersection should ",
    ///         "be the same as the cardinality of the first set."
    ///    )
    /// );
    /// ```
    ///
    /// We can create the HLL counters from array from registers,
    /// so to be able to check that everything works as expected.
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let first_registers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    /// let second_registers = [9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 19];
    /// let expected = [9, 9, 9, 9, 9, 9, 9, 9, 9, 10, 11, 12, 13, 14, 15, 19];
    ///
    /// let mut hll1 = HyperLogLog::<Precision4, 5>::from_registers(&first_registers);
    /// let mut hll2 = HyperLogLog::<Precision4, 5>::from_registers(&second_registers);
    /// let intersection = hll1 | &hll2;
    ///
    /// assert_eq!(intersection.get_registers(), expected, "The registers are not the expected ones, got {:?} instead of {:?}.", intersection.get_registers(), expected);
    /// ```
    ///
    fn bitand(mut self, rhs: &Self) -> Self {
        self.bitand_assign(rhs);
        self
    }
}
