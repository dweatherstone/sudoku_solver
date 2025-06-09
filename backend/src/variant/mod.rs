mod killer;
mod kropki;
mod quadruple_circles;

pub use killer::KillerCage;
pub use kropki::KropkiDot;
pub use quadruple_circles::QuadrupleCircle;

use crate::SudokuGrid;

pub trait Variant {
    fn is_valid(&self, grid: &SudokuGrid, row: usize, col: usize, value: u8) -> bool;
    fn affected_cells(&self) -> Vec<(usize, usize)>;
    fn name(&self) -> String;
}
