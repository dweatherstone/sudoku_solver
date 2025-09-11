mod chess;
mod dot;
mod error;
mod line;
mod misc;

pub use chess::King;
pub use chess::Knight;
pub use dot::KropkiDot;
pub use dot::XVDot;
pub use error::{PossibilityResult, VariantContradiction};
pub use line::Arrow;
pub use line::Diagonal;
pub use line::Entropic;
pub use line::GermanWhisper;
pub use line::Nabner;
pub use line::RegionSum;
pub use line::Renban;
pub use line::Thermometer;
pub use misc::KillerCage;
pub use misc::QuadrupleCircle;
pub use misc::Shaded;

use crate::SudokuGrid;

pub trait Variant {
    /// Determines if the variant is valid, given the current state of the `grid`, assuming a proposed `value` is placed in the cell in (`row`, `col`).
    fn is_valid(&self, grid: &SudokuGrid, row: usize, col: usize, value: u8) -> bool;
    /// Returns the list of cells affected by this variant.
    fn constrained_cells(&self) -> Vec<(usize, usize)>;
    /// Determines if the variant is valid for the proposed final grid.
    fn validate_solution(&self, grid: &SudokuGrid) -> bool;
    /// Return all possible values (according to the variant's constraint rules) for all cells affected by the variant.
    fn get_possibilities(&self, grid: &SudokuGrid) -> PossibilityResult;
}

pub const ALL_POSSIBILITIES: [u8; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];
