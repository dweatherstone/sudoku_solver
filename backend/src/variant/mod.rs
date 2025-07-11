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
    fn is_valid(&self, grid: &SudokuGrid, row: usize, col: usize, value: u8) -> bool;
    fn constrained_cells(&self) -> Vec<(usize, usize)>;
    fn validate_solution(&self, grid: &SudokuGrid) -> bool;
    fn get_possibilities(
        &self,
        grid: &SudokuGrid,
        row: usize,
        col: usize,
    ) -> HashMap<(usize, usize), Vec<u8>>;
}
