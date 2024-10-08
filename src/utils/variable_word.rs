//! Submodule providing the variable word trait, which is used in combination
//! with a packed array. This allows to define 'virtual' words with sizes that
//! are not a power of two.
use super::PositiveInteger;
use core::fmt::Debug;
use hyperloglog_derive::VariableWord;

/// Trait marker for the variable word.
pub trait VariableWord: Send + Sync + Clone + Copy + Debug + Default {
    /// The number of bits in the word.
    const NUMBER_OF_BITS: u8;
    /// The number of bits in the word as a usize.
    const NUMBER_OF_BITS_USIZE: usize = Self::NUMBER_OF_BITS as usize;
    /// The number of bits in the word as a u64.
    const NUMBER_OF_BITS_U64: u64 = Self::NUMBER_OF_BITS as u64;
    /// The number of entries in a u64.
    const NUMBER_OF_ENTRIES: u8 = 64 / Self::NUMBER_OF_BITS;
    /// The number of entries in a u64 as a usize.
    const NUMBER_OF_ENTRIES_USIZE: usize = Self::NUMBER_OF_ENTRIES as usize;
    /// The number of entries in a u64.
    const NUMBER_OF_ENTRIES_U64: u64 = Self::NUMBER_OF_ENTRIES as u64;
    /// The mask for the word.
    const MASK: u64;
    /// The word type.
    type Word: PositiveInteger + TryInto<u8> + TryInto<u16> + TryInto<u64>;

    #[allow(unsafe_code)]
    /// Converts the word to a u64.
    ///
    /// # Safety
    /// This method is unsafe because it may return a value that may truncate the word.
    /// It needs to be used with caution and where appropriate.
    unsafe fn unchecked_from_u64(value: u64) -> Self::Word;
}

/// Virtual word with 24 bits.
#[allow(non_camel_case_types)]
#[derive(Clone, VariableWord)]
pub struct u24(u32);

/// Virtual word with 40 bits.
#[allow(non_camel_case_types)]
#[derive(Clone, VariableWord)]
pub struct u40(u64);

/// Virtual word with 48 bits.
#[allow(non_camel_case_types)]
#[derive(Clone, VariableWord)]
pub struct u48(u64);

/// Virtual word with 56 bits.
#[allow(non_camel_case_types)]
#[derive(Clone, VariableWord)]
pub struct u56(u64);

impl VariableWord for u8 {
    const NUMBER_OF_BITS: u8 = 8;
    type Word = u8;
    const MASK: u64 = 0xFF;

    #[inline]
    #[allow(unsafe_code)]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "The value is checked to be within the bounds and the method is marked as unsafe."
    )]
    unsafe fn unchecked_from_u64(value: u64) -> Self {
        debug_assert!(
            value <= <Self as crate::prelude::VariableWord>::MASK,
            "The value is too large for the number."
        );
        value as Self
    }
}

impl VariableWord for u16 {
    const NUMBER_OF_BITS: u8 = 16;
    type Word = u16;
    const MASK: u64 = 0xFFFF;

    #[inline]
    #[allow(unsafe_code)]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "The value is checked to be within the bounds and the method is marked as unsafe."
    )]
    unsafe fn unchecked_from_u64(value: u64) -> Self {
        debug_assert!(
            value <= <Self as crate::prelude::VariableWord>::MASK,
            "The value is too large for the number."
        );
        value as Self
    }
}

impl VariableWord for u32 {
    const NUMBER_OF_BITS: u8 = 32;
    type Word = u32;
    const MASK: u64 = 0xFFFF_FFFF;

    #[inline]
    #[allow(unsafe_code)]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "The value is checked to be within the bounds and the method is marked as unsafe."
    )]
    unsafe fn unchecked_from_u64(value: u64) -> Self {
        debug_assert!(
            value <= <Self as crate::prelude::VariableWord>::MASK,
            "The value is too large for the number."
        );
        value as Self
    }
}

impl VariableWord for u64 {
    const NUMBER_OF_BITS: u8 = 64;
    type Word = u64;
    const MASK: u64 = 0xFFFF_FFFF_FFFF_FFFF;

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn unchecked_from_u64(value: u64) -> Self {
        value
    }
}
