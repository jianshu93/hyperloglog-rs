//! Submodule providing the trait marker Bits.

/// Trait marker for the number of bits.
pub trait Bits: Default + Copy + PartialEq + Eq + Send + Sync + core::fmt::Debug{
    /// The number of bits.
    const NUMBER_OF_BITS: u8;
}

/// Implementation
macro_rules! impl_bits {
    ($($n: expr),*) => {
        $(
            paste::paste! {
                #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
                #[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                /// A struct representing the number of bits.
                pub struct [<Bits $n>];

                #[cfg(feature = "std")]
                impl crate::prelude::Named for [<Bits $n>] {
                    fn name(&self) -> String {
                        format!("B{}", $n)
                    }
                }

                impl Bits for [<Bits $n>] {
                    const NUMBER_OF_BITS: u8 = $n;
                }
            }
        )*
    };
}

impl_bits!(1, 2, 3, 4, 5, 6, 7, 8);
