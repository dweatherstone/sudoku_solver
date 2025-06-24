mod dot;
mod line;
mod misc;

pub use dot::KropkiDot;
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
}
