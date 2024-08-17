//! Marker struct for the hybrid approach, that keeps the hash explicit up until they fit into the registers.

use crate::prelude::*;
use core::cmp::Ordering;
use core::hash::Hash;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A struct representing the hybrid for approximate set cardinality estimation,
/// where the hash values are kept explicit up until they fit into the registers.
pub struct Hybrid<H> {
    /// The inner counter.
    inner: H,
}

impl<H: Hybridazable> Default for Hybrid<H>
where
    H: Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            inner: H::new_hybrid(),
        }
    }
}

impl<H: Hybridazable> Hybridazable for Hybrid<H>
where
    H: Hybridazable,
{
    type IterSortedHashes<'words> = H::IterSortedHashes<'words> where Self: 'words;

    #[inline]
    fn dehybridize(&mut self) {
        self.inner.dehybridize();
    }

    #[inline]
    fn new_hybrid() -> Self {
        Self::default()
    }

    #[inline]
    fn is_hybrid(&self) -> bool {
        self.inner.is_hybrid()
    }

    #[inline]
    fn number_of_hashes(&self) -> usize {
        self.inner.number_of_hashes()
    }

    #[inline]
    fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    #[inline]
    fn clear_words(&mut self) {
        self.inner.clear_words();
    }

    #[inline]
    fn iter_sorted_hashes(&self) -> Self::IterSortedHashes<'_> {
        self.inner.iter_sorted_hashes()
    }

    #[inline]
    fn contains<T: Hash>(&self, element: &T) -> bool {
        self.inner.contains(element)
    }

    #[inline]
    fn hybrid_insert<T: Hash>(&mut self, value: &T) -> bool {
        self.inner.hybrid_insert(value)
    }
}

impl<H: PartialEq> PartialEq<Self> for Hybrid<H> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<H: PartialEq<H>> PartialEq<H> for Hybrid<H> {
    #[inline]
    fn eq(&self, other: &H) -> bool {
        &self.inner == other
    }
}

impl<H: Eq> Eq for Hybrid<H> {}

impl<H: SetProperties + Hybridazable> SetProperties for Hybrid<H> {
    #[inline]
    fn is_empty(&self) -> bool {
        if self.is_hybrid() {
            self.inner.number_of_hashes() == 0
        } else {
            self.inner.is_empty()
        }
    }

    #[inline]
    fn is_full(&self) -> bool {
        if self.is_hybrid() {
            self.inner.number_of_hashes() == self.inner.capacity()
        } else {
            self.inner.is_full()
        }
    }
}

impl<T: Hash, H: ApproximatedSet<T> + Hybridazable> ApproximatedSet<T> for Hybrid<H> {
    #[inline]
    fn may_contain(&self, element: &T) -> bool {
        if self.is_hybrid() {
            Hybridazable::contains(&self.inner, element)
        } else {
            self.inner.may_contain(element)
        }
    }
}

impl<H: MutableSet + Hybridazable> MutableSet for Hybrid<H> {
    #[inline]
    fn clear(&mut self) {
        self.inner.clear_words();
    }
}

impl<T: Hash, H: ExtendableApproximatedSet<T> + Hybridazable> ExtendableApproximatedSet<T>
    for Hybrid<H>
{
    #[inline]
    fn insert(&mut self, element: &T) -> bool {
        if self.is_hybrid() {
            Hybridazable::hybrid_insert(&mut self.inner, element)
        } else {
            self.inner.insert(element)
        }
    }
}

#[inline]
/// Returns the number of unique values from two sorted iterators.
///
/// # Implementative details
/// The sets we are considering are the union of the two sorted iterators
/// of Hybrid counters' hashes. The largest possible number of unique values
/// in each iterator is the number of words in a 2**18 counter, with the bit
/// size set to 8 (used primarely to benefit from the SIMD instructions).
/// As such 8 * 2**18 = 2**21, divided by the number of bits in a u64, we get
/// 2**21 / 64 = 2**15 unique values. The number of unique values in the union
/// of the two sets is at most the sum of the number of unique values in each set,
/// so at most 2**16 unique values. We can thus use a u32 to represent the number
/// of unique values.
fn unique_values_from_sorted_iterators<T: Ord, I: Iterator<Item = T>, J: Iterator<Item = T>>(
    mut left: I,
    mut right: J,
) -> u32 {
    let mut count = u32::ZERO;
    let mut maybe_left_value = left.next();
    let mut maybe_right_value = right.next();
    while let Some(ord) = maybe_left_value.as_ref().and_then(|left_value| {
        maybe_right_value
            .as_ref()
            .map(|right_value| left_value.cmp(right_value))
    }) {
        count += u32::ONE;
        match ord {
            Ordering::Less => {
                maybe_left_value = left.next();
            }
            Ordering::Greater => {
                maybe_right_value = right.next();
            }
            Ordering::Equal => {
                maybe_left_value = left.next();
                maybe_right_value = right.next();
            }
        }
    }

    if maybe_left_value.is_some() {
        count += u32::ONE;
    }

    if maybe_right_value.is_some() {
        count += u32::ONE;
    }

    count + u32::try_from(left.count()).unwrap() + u32::try_from(right.count()).unwrap()
}

/// Trait for a struct that can be used in the hybrid approach.
pub trait Hybridazable: Default {
    /// The type of the iterator over the sorted hashes.
    type IterSortedHashes<'words>: Iterator<Item = u64>
    where
        Self: 'words;

    /// De-hybridize the struct, i.e., convert it to a register-based counter.
    fn dehybridize(&mut self);

    /// Returns a new hybrid instance.
    fn new_hybrid() -> Self;

    /// Returns whether the struct is currently behaving as
    /// a hybrid counter.
    fn is_hybrid(&self) -> bool;

    /// Returns the number of hashes currently inserted.
    fn number_of_hashes(&self) -> usize;

    /// Returns the capacity of the counter.
    fn capacity(&self) -> usize;

    /// Clears the counter.
    fn clear_words(&mut self);

    /// Returns an iterator over the sorted hashes.
    fn iter_sorted_hashes(&self) -> Self::IterSortedHashes<'_>;

    /// Returns whether the counter contains the element.
    fn contains<T: Hash>(&self, element: &T) -> bool;

    /// Inserts a value into the counter.
    fn hybrid_insert<T: Hash>(&mut self, value: &T) -> bool;
}

#[cfg(feature = "std")]
impl<H: Named> Named for Hybrid<H> {
    #[inline]
    fn name(&self) -> String {
        format!("Hybrid{}", self.inner.name())
    }
}

impl<H: Clone + Estimator<f64> + Hybridazable + Default> Estimator<f64> for Hybrid<H>
where
    Hybrid<H>: Default,
{
    #[inline]
    fn estimate_cardinality(&self) -> f64 {
        if self.inner.is_hybrid() {
            // We can safely convert this usize to an u32 because the maximal value that
            // can be stored in an Hybrid counter with the largest possible number of words
            // using the largest possible bit size (8) is 2**21 / 64 = 2**15, which fits
            // cosily in an u16.
            f64::from(u16::try_from(self.inner.number_of_hashes()).unwrap())
        } else {
            self.inner.estimate_cardinality()
        }
    }

    #[inline]
    fn is_union_estimate_non_deterministic(&self, other: &Self) -> bool {
        !(self.is_hybrid() && other.is_hybrid())
            && self.inner.is_union_estimate_non_deterministic(&other.inner)
    }

    #[inline]
    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        match (self.is_hybrid(), other.is_hybrid()) {
            (true, true) => {
                // In the case where both counters are in hybrid mode, we can
                // simply iterate on the two sorted hash arrays and determine the number
                // of unique hashes.
                f64::from(unique_values_from_sorted_iterators(
                    self.iter_sorted_hashes(),
                    other.iter_sorted_hashes(),
                ))
            }
            (true, false) => {
                let mut copy = self.clone();
                copy.dehybridize();
                copy.estimate_union_cardinality(other)
            }
            (false, true) => other.estimate_union_cardinality(self),
            (false, false) => self.inner.estimate_union_cardinality(&other.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "std")]
    fn test_unique_values_from_sorted_iterators() {
        let number_of_iterations = 10;
        let mut random_state = splitmix64(3456789456776543);

        for _ in 0..number_of_iterations {
            random_state = splitmix64(random_state);
            let mut left = iter_random_values(1000, None, random_state).collect::<Vec<_>>();
            left.sort();
            random_state = splitmix64(random_state);
            let mut right = iter_random_values(1000, None, random_state).collect::<Vec<_>>();
            right.sort();

            let unique_values =
                unique_values_from_sorted_iterators(left.iter().cloned(), right.iter().cloned());
            let unique_values_set = u32::try_from(
                left.iter()
                    .chain(right.iter())
                    .collect::<std::collections::HashSet<_>>()
                    .len(),
            )
            .unwrap();
            assert_eq!(unique_values, unique_values_set);
        }
    }

    #[test]
    #[cfg(feature = "precision_10")]
    fn test_hybrid_plusplus() {
        let number_of_iterations = 10;
        let mut hybrid: Hybrid<
            PlusPlus<
                Precision10,
                Bits6,
                <Precision10 as ArrayRegister<Bits6>>::ArrayRegister,
                twox_hash::XxHash64,
            >,
        > = Default::default();

        // The estimations up until the number of words is reached should be exact.
        for _ in 0..number_of_iterations {
            hybrid.clear();
            assert!(hybrid.is_empty());
            let cardinality: f64 = hybrid.estimate_cardinality();
            assert_eq!(cardinality, 0.0_f64);

            assert!(hybrid.is_hybrid());
            let mut exact_set = std::collections::HashSet::new();
            let mut random_state = splitmix64(3456789456776543);

            for element in iter_random_values(1000, None, random_state) {
                random_state = splitmix64(random_state);
                hybrid.insert(&element);
                exact_set.insert(element);
                assert!(hybrid.may_contain(&element));
                if !hybrid.is_hybrid() {
                    break;
                }
                let estimated_cardinality: f64 = hybrid.estimate_cardinality();
                assert_eq!(estimated_cardinality, exact_set.len() as f64);
            }
        }
    }

    #[cfg(feature = "std")]
    /// This test populates two hybrid counters, of which one is populated up until
    /// it saturates and is no longer in hybrid mode. The union of the two counters
    /// is then estimated, and is expected to be within the error rate defined by the
    /// provided precision.
    fn test_mixed_hybrid_union<
        P: Precision + ArrayRegister<Bits6>,
        H: Hybridazable
            + HyperLogLog<P, Bits6, twox_hash::XxHash64>
            + Estimator<f64>
            + Default
            + ExtendableApproximatedSet<u64>
            + core::fmt::Debug,
    >()
    where
        Hybrid<H>: Default + Estimator<f64> + ApproximatedSet<u64>,
    {
        use std::collections::HashSet;
        let mut hybrid_to_saturate = Hybrid::<H>::default();
        let mut hybrid = Hybrid::<H>::default();
        let mut left_normal_counter: H = Default::default();
        let mut right_normal_counter: H = Default::default();

        let mut left_set = HashSet::new();
        let mut right_set = HashSet::new();
        let number_of_iterations = 10;
        let mut number_of_elements_needed_for_saturation;
        let mut random_state = splitmix64(3456789456776543);
        let mut union_cardinality_errors_total = 0.0;
        let mut normal_union_cardinality_errors_total = 0.0;
        let mut dehybridized_union_cardinality_errors_total = 0.0;

        for _ in 0..number_of_iterations {
            random_state = splitmix64(random_state);
            hybrid_to_saturate.clear();
            hybrid.clear();
            left_set.clear();
            right_set.clear();
            left_normal_counter.clear();
            right_normal_counter.clear();
            assert!(hybrid.is_hybrid());
            assert!(hybrid_to_saturate.is_hybrid());
            assert!(!hybrid.is_union_estimate_non_deterministic(&hybrid_to_saturate));
            number_of_elements_needed_for_saturation = 0;

            // First, we make sure that the one we intend to saturate
            // is populated up until it saturates.
            while hybrid_to_saturate.is_hybrid() {
                random_state = splitmix64(random_state);
                for element in iter_random_values(10_000, None, random_state) {
                    hybrid_to_saturate.insert(&element);
                    left_set.insert(element);
                    left_normal_counter.insert(&element);
                    assert!(hybrid_to_saturate.may_contain(&element));
                    if hybrid_to_saturate.is_hybrid() {
                        number_of_elements_needed_for_saturation += 1;
                    }
                }
            }

            // Next, we populate the other counter, filling it up to
            // at most the number of elements needed for saturation.
            // This guarantees that the union of the two counters
            // will be between a hybrid counter and a register-based counter.
            for element in
                iter_random_values(number_of_elements_needed_for_saturation, None, random_state)
            {
                hybrid.insert(&element);
                right_set.insert(element);
                right_normal_counter.insert(&element);
                assert!(hybrid.may_contain(&element));
            }

            // We check that the two counters are in the expected state.
            assert!(hybrid.is_hybrid());
            assert!(!hybrid_to_saturate.is_hybrid());
            assert!(!hybrid.is_union_estimate_non_deterministic(&hybrid_to_saturate));

            // We estimate the union of the two counters.
            let union_estimate: f64 = hybrid.estimate_union_cardinality(&hybrid_to_saturate);
            // The value we obtain must be symmetric.
            assert_eq!(
                union_estimate,
                hybrid_to_saturate.estimate_union_cardinality(&hybrid)
            );

            // We calculate the exact union of the two sets.
            let exact_union = left_set.union(&right_set).count() as f64;

            union_cardinality_errors_total += (union_estimate - exact_union).abs() / exact_union;

            // We estimate the union of the two normal counters.
            let normal_union_estimate: f64 =
                left_normal_counter.estimate_union_cardinality(&right_normal_counter);

            normal_union_cardinality_errors_total +=
                (normal_union_estimate - exact_union).abs() / exact_union;

            // We check that if we de-hybridize the counter, the resulting counter is
            // identical to the normal counter.
            hybrid.dehybridize();
            assert!(!hybrid.is_hybrid());
            assert_eq!(right_normal_counter, hybrid.inner);
            assert_eq!(left_normal_counter, hybrid_to_saturate.inner);
            assert!(!hybrid.is_union_estimate_non_deterministic(&hybrid_to_saturate));

            // We estimate the union of the two de-hybridized counters.
            let dehybridized_union_estimate: f64 =
                hybrid.estimate_union_cardinality(&hybrid_to_saturate);

            dehybridized_union_cardinality_errors_total +=
                (dehybridized_union_estimate - exact_union).abs() / exact_union;
        }

        let average_union_cardinality_error =
            union_cardinality_errors_total / number_of_iterations as f64;
        let average_normal_union_cardinality_error =
            normal_union_cardinality_errors_total / number_of_iterations as f64;
        let average_dehybridized_union_cardinality_error =
            dehybridized_union_cardinality_errors_total / number_of_iterations as f64;

        assert!(
            average_union_cardinality_error < P::error_rate(),
            "Expected: <{}, got: {} with hybrid mix, {} dehybridized, and {} with normal counters",
            P::error_rate(),
            average_union_cardinality_error,
            average_dehybridized_union_cardinality_error,
            average_normal_union_cardinality_error
        );
    }

    /// Macro to generate tests for the test_mixed_hybrid_union.
    macro_rules! test_mixed_hybrid_union_from_exponents {
        ($($exponent:expr),*) => {
            $(
                paste::item! {
                    #[test]
                    #[cfg(all(feature = "precision_" $exponent, feature = "std"))]
                    fn [<test_mixed_plusplus_hybrid_union_ $exponent>]() {
                        test_mixed_hybrid_union::<[<Precision $exponent>], PlusPlus<[<Precision $exponent>], Bits6, <[<Precision $exponent>] as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>>();
                    }

                    #[test]
                    #[cfg(all(feature = "precision_" $exponent, feature = "std"))]
                    fn [<test_mixed_beta_hybrid_union_ $exponent>]() {
                        test_mixed_hybrid_union::<[<Precision $exponent>], LogLogBeta<[<Precision $exponent>], Bits6, <[<Precision $exponent>] as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>>();
                    }
                }
            )*
        };
    }

    test_mixed_hybrid_union_from_exponents!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);

    #[cfg(feature = "std")]
    /// This test populates two hybrid counters, of which one is populated up until
    /// it saturates and is no longer in hybrid mode. The union of the two counters
    /// is then estimated, and is expected to be within the error rate defined by the
    /// provided precision.
    fn test_randomized_hybrid_union<
        P: Precision + ArrayRegister<Bits6>,
        H: Hybridazable
            + HyperLogLog<P, Bits6, twox_hash::XxHash64>
            + Estimator<f64>
            + Default
            + ExtendableApproximatedSet<u64>
            + core::fmt::Debug,
    >()
    where
        Hybrid<H>: Default + Estimator<f64>,
    {
        use std::collections::HashSet;
        let mut hybrid_left = Hybrid::<H>::default();
        let mut hybrid_right = Hybrid::<H>::default();
        let mut left_normal_counter: H = Default::default();
        let mut right_normal_counter: H = Default::default();

        let mut left_set = HashSet::new();
        let mut right_set = HashSet::new();
        let number_of_iterations = 10;
        let mut random_state = splitmix64(3456789456776543);
        let mut union_cardinality_errors_total = 0.0;
        let mut normal_union_cardinality_errors_total = 0.0;
        let mut number_of_samples = 0;
        let starting_union_sampling_rate = 10;
        let maximal_union_sampling_rate = 10_000;

        for _ in 0..number_of_iterations {
            random_state = splitmix64(random_state);
            hybrid_left.clear();
            hybrid_right.clear();
            left_set.clear();
            right_set.clear();
            left_normal_counter.clear();
            right_normal_counter.clear();
            let mut sampling_rate = starting_union_sampling_rate;

            // Next, we populate the other counter, filling it up to
            // at most the number of elements needed for saturation.
            // This guarantees that the union of the two counters
            // will be between a hybrid counter and a register-based counter.
            for (i, element) in iter_random_values(200_000, None, random_state).enumerate() {
                if i % 2 == 0 {
                    hybrid_left.insert(&element);
                    left_set.insert(element);
                    left_normal_counter.insert(&element);
                } else {
                    hybrid_right.insert(&element);
                    right_set.insert(element);
                    right_normal_counter.insert(&element);
                }

                if i % sampling_rate == 0 {
                    if sampling_rate < maximal_union_sampling_rate {
                        sampling_rate *= 2;
                    }

                    // We calculate the exact union of the two sets.
                    let exact_union = left_set.union(&right_set).count() as f64;

                    let union_estimate: f64 = hybrid_left.estimate_union_cardinality(&hybrid_right);

                    union_cardinality_errors_total +=
                        (union_estimate - exact_union).abs() / exact_union;

                    // We estimate the union of the two normal counters.
                    let normal_union_estimate: f64 =
                        left_normal_counter.estimate_union_cardinality(&right_normal_counter);

                    normal_union_cardinality_errors_total +=
                        (normal_union_estimate - exact_union).abs() / exact_union;

                    number_of_samples += 1;
                }
            }
        }

        let average_union_cardinality_error =
            union_cardinality_errors_total / number_of_samples as f64;

        let average_normal_union_cardinality_error =
            normal_union_cardinality_errors_total / number_of_samples as f64;

        assert!(
            average_union_cardinality_error < P::error_rate(),
            "Expected union cardinality error: <{}, got: {} with hybrid mix, and {} with normal counters",
            P::error_rate(),
            average_union_cardinality_error,
            average_normal_union_cardinality_error
        );
    }

    /// Macro to generate tests for the test_randomized_hybrid_union.
    macro_rules! test_randomized_hybrid_union_from_exponents {
        ($($exponent:expr),*) => {
            $(
                paste::item! {
                    #[test]
                    #[cfg(all(feature = "precision_" $exponent, feature = "std"))]
                    fn [<test_randomized_mixed_plusplus_hybrid_union_ $exponent>]() {
                        test_randomized_hybrid_union::<[<Precision $exponent>], PlusPlus<[<Precision $exponent>], Bits6, <[<Precision $exponent>] as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>>();
                    }

                    #[test]
                    #[cfg(all(feature = "precision_" $exponent, feature = "std"))]
                    fn [<test_randomized_mixed_beta_hybrid_union_ $exponent>]() {
                        test_randomized_hybrid_union::<[<Precision $exponent>], LogLogBeta<[<Precision $exponent>], Bits6, <[<Precision $exponent>] as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>>();
                    }
                }
            )*
        };
    }

    test_randomized_hybrid_union_from_exponents!(
        4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18
    );

    #[cfg(feature = "std")]
    fn test_only_hybrid_union<
        P: Precision + ArrayRegister<Bits6>,
        H: Hybridazable
            + HyperLogLog<P, Bits6, twox_hash::XxHash64>
            + Estimator<f64>
            + Default
            + ExtendableApproximatedSet<u64>
            + core::fmt::Debug,
    >()
    where
        Hybrid<H>: Default + Estimator<f64>,
    {
        use std::collections::HashSet;
        let mut hybrid_left = Hybrid::<H>::default();
        let mut hybrid_right = Hybrid::<H>::default();
        let mut left_normal_counter: H = Default::default();
        let mut right_normal_counter: H = Default::default();

        let mut left_set = HashSet::new();
        let mut right_set = HashSet::new();
        let number_of_iterations = 10;
        let mut random_state = splitmix64(3456789456776543);
        let mut union_cardinality_errors_total = 0.0;
        let mut normal_union_cardinality_errors_total = 0.0;
        let mut number_of_samples = 0;
        let starting_union_sampling_rate = 1;
        let maximal_union_sampling_rate = 10_000;

        for _ in 0..number_of_iterations {
            random_state = splitmix64(random_state);
            hybrid_left.clear();
            hybrid_right.clear();
            left_set.clear();
            right_set.clear();
            left_normal_counter.clear();
            right_normal_counter.clear();
            let mut sampling_rate = starting_union_sampling_rate;

            // Next, we populate the other counter, filling it up to
            // at most the number of elements needed for saturation.
            // This guarantees that the union of the two counters
            // will be between a hybrid counter and a register-based counter.
            for (i, element) in iter_random_values(200_000, None, random_state).enumerate() {
                if i % 2 == 0 {
                    if hybrid_left.is_full() {
                        continue;
                    }
                    assert!(hybrid_left.is_hybrid());
                    hybrid_left.insert(&element);
                    left_set.insert(element);
                    left_normal_counter.insert(&element);
                    assert!(hybrid_left.is_hybrid());
                } else {
                    if hybrid_right.is_full() {
                        continue;
                    }
                    assert!(hybrid_right.is_hybrid());
                    hybrid_right.insert(&element);
                    right_set.insert(element);
                    right_normal_counter.insert(&element);
                    assert!(hybrid_right.is_hybrid());
                }

                if i % sampling_rate == 0 {
                    if sampling_rate < maximal_union_sampling_rate {
                        sampling_rate *= 2;
                    }

                    // We calculate the exact union of the two sets.
                    let exact_union = left_set.union(&right_set).count() as f64;

                    let union_estimate: f64 = hybrid_left.estimate_union_cardinality(&hybrid_right);

                    union_cardinality_errors_total +=
                        (union_estimate - exact_union).abs() / exact_union;

                    // We estimate the union of the two normal counters.
                    let normal_union_estimate: f64 =
                        left_normal_counter.estimate_union_cardinality(&right_normal_counter);

                    normal_union_cardinality_errors_total +=
                        (normal_union_estimate - exact_union).abs() / exact_union;

                    number_of_samples += 1;
                }
            }
        }

        let average_union_cardinality_error =
            union_cardinality_errors_total / number_of_samples as f64;

        let average_normal_union_cardinality_error =
            normal_union_cardinality_errors_total / number_of_samples as f64;

        assert!(
            average_union_cardinality_error < P::error_rate(),
            "Expected: <{}, got: {} with hybrid mix, and {} with normal counters",
            P::error_rate(),
            average_union_cardinality_error,
            average_normal_union_cardinality_error
        );
    }

    /// Macro to generate tests for the test_only_hybrid_union.
    macro_rules! test_only_hybrid_union_from_exponents {
        ($($exponent:expr),*) => {
            $(
                paste::item! {
                    #[test]
                    #[cfg(all(feature = "precision_" $exponent, feature = "std"))]
                    fn [<test_only_hybrid_plusplus_union_ $exponent>]() {
                        test_only_hybrid_union::<[<Precision $exponent>], PlusPlus<[<Precision $exponent>], Bits6, <[<Precision $exponent>] as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>>();
                    }

                    #[test]
                    #[cfg(all(feature = "precision_" $exponent, feature = "std"))]
                    fn [<test_only_hybrid_beta_union_ $exponent>]() {
                        test_only_hybrid_union::<[<Precision $exponent>], LogLogBeta<[<Precision $exponent>], Bits6, <[<Precision $exponent>] as ArrayRegister<Bits6>>::ArrayRegister, twox_hash::XxHash64>>();
                    }
                }
            )*
        };
    }

    test_only_hybrid_union_from_exponents!(4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18);
}
