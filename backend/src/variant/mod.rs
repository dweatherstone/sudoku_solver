mod diagonal;
mod killer;
mod kropki;
mod quadruple_circles;
mod renban;
mod thermometer;

pub use diagonal::Diagonal;
pub use killer::KillerCage;
pub use kropki::KropkiDot;
pub use quadruple_circles::QuadrupleCircle;
pub use renban::Renban;
pub use thermometer::Thermometer;

use crate::SudokuGrid;

pub trait Variant {
    fn is_valid(&self, grid: &SudokuGrid, row: usize, col: usize, value: u8) -> bool;
    fn constrained_cells(&self) -> Vec<(usize, usize)>;
    fn validate_solution(&self, grid: &SudokuGrid) -> bool;
}
