use std::collections::HashMap;
use std::fmt;

pub type PossibilityMap = HashMap<(usize, usize), Vec<u8>>;
pub type PossibilityResult = Result<PossibilityMap, VariantContradiction>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariantContradiction {
    // A specific cell ended up with no valid digits due to this variant
    NoPossibilities {
        cell: (usize, usize),
        variant: &'static str,
        reason: String,
    },
    // A broader failure (e.g., cross-cell rule collpase or internal invariant)
    Inconsistent {
        variant: &'static str,
        reason: String,
    },
}

impl std::error::Error for VariantContradiction {}

impl fmt::Display for VariantContradiction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VariantContradiction::NoPossibilities {
                cell,
                variant,
                reason,
            } => {
                write!(
                    f,
                    "{}: no possibilities at ({}, {}): {}",
                    variant, cell.0, cell.1, reason
                )
            }
            VariantContradiction::Inconsistent { variant, reason } => {
                write!(f, "{}: Inconsistent state: {}", variant, reason)
            }
        }
    }
}
