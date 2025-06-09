use serde::{Deserialize, Serialize};

use crate::variant::Variant;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QuadrupleCircle {
    cells: Vec<(usize, usize)>,
    required: Vec<u8>,
}

impl QuadrupleCircle {
    pub fn new(cells: Vec<(usize, usize)>, required: Vec<u8>) -> Self {
        QuadrupleCircle { cells, required }
    }
}

impl Variant for QuadrupleCircle {
    fn is_valid(&self, grid: &crate::SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        // If (row, col) is not in the quadruple circle, just pass
        if !self.cells.contains(&(row, col)) {
            return true;
        }
        // If there are 4 required numbers, and value is not one of them, then early return
        if self.required.len() == 4 && !self.required.contains(&value) {
            return false;
        }

        // Build the current set of values in the 4 cells, with the proposed value substituted in
        let current: Vec<u8> = self
            .cells
            .iter()
            .map(|&(r, c)| {
                if (r, c) == (row, col) {
                    value
                } else {
                    grid.get_cell(r, c)
                }
            })
            .collect();

        // Track which required digits are already present
        let mut missing_required = self.required.clone();
        missing_required.retain(|&d| !current.contains(&d));

        // Count how many cells are still unfilled
        let unfilled_count = current.iter().filter(|&&v| v == 0).count();

        // If the number of missing required digits is more than unfilled cells, fail early
        if missing_required.len() > unfilled_count {
            return false;
        }

        // If all filled, ensure all required digits are present
        if unfilled_count == 0 && !missing_required.is_empty() {
            return false;
        }
        true
    }

    fn affected_cells(&self) -> Vec<(usize, usize)> {
        self.cells.clone()
    }

    fn name(&self) -> String {
        String::from("Quadruple Circle")
    }
}
