mod dot;
mod line;
mod misc;

use std::collections::HashMap;

pub use dot::KropkiDot;
pub use dot::XVDot;
pub use line::Arrow;
pub use line::Diagonal;
pub use line::Entropic;
pub use line::RegionSum;
pub use line::Renban;
pub use line::Thermometer;
pub use misc::KillerCage;
pub use misc::QuadrupleCircle;

use crate::SudokuGrid;

pub trait Variant {
    /// Determines if the variant is valid, given the current state of the `grid`, assuming a proposed `value` is placed in the cell in (`row`, `col`).
    fn is_valid(&self, grid: &SudokuGrid, row: usize, col: usize, value: u8) -> bool;
    /// Returns the list of cells affected by this variant.
    fn constrained_cells(&self) -> Vec<(usize, usize)>;
    /// Determines if the variant is valid for the proposed final grid.
    fn validate_solution(&self, grid: &SudokuGrid) -> bool;
    /// Once a proposed move is palced in (`row`, `col`), this will return the possible values remaining for any other cells affected by this variant.
    fn get_possibilities(
        &self,
        grid: &SudokuGrid,
        row: usize,
        col: usize,
    ) -> HashMap<(usize, usize), Vec<u8>>;
}
