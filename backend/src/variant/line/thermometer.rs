use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{SudokuVariant, file_parser::parse_positions, variant::Variant};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Thermometer {
    cells: Vec<(usize, usize)>,
    length: usize,
}

impl Thermometer {
    pub fn new(cells: Vec<(usize, usize)>) -> Self {
        let length = cells.len();
        Thermometer { cells, length }
    }

    pub fn parse(data: &str) -> Option<SudokuVariant> {
        let positions = parse_positions(data).ok()?;
        Some(SudokuVariant::Thermometer(Thermometer::new(positions)))
    }
}

impl Variant for Thermometer {
    fn is_valid(&self, grid: &crate::SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        if !self.cells.contains(&(row, col)) {
            return true;
        }
        let idx = match self.cells.iter().position(|&(r, c)| r == row && c == col) {
            Some(i) => i,
            None => return true, // If (row, col) is not on the thermometer, just pass
        };
        let min_val = (idx + 1) as u8;
        let max_val = (9 - (self.length - 1 - idx)) as u8;

        if value < min_val || value > max_val {
            return false;
        }

        for (i, &(r, c)) in self.cells.iter().enumerate() {
            if r == row && c == col {
                continue;
            }

            let cell_value = grid.get_cell(r, c);
            if cell_value == 0 {
                continue; // Skip unknown cells
            }

            if (i < idx && cell_value >= value) || (i > idx && cell_value <= value) {
                return false;
            }
        }
        true
    }

    fn validate_solution(&self, grid: &crate::SudokuGrid) -> bool {
        let values: Vec<u8> = self
            .cells
            .iter()
            .map(|&(r, c)| grid.get_cell(r, c))
            .collect();

        // Check all cells are filled
        if values.contains(&0) {
            return false;
        }

        // Check values are in ascending order
        values.windows(2).all(|w| w[0] < w[1])
    }

    fn constrained_cells(&self) -> Vec<(usize, usize)> {
        self.cells.clone()
    }

    fn get_possibilities(
        &self,
        grid: &crate::SudokuGrid,
        row: usize,
        col: usize,
    ) -> HashMap<(usize, usize), Vec<u8>> {
        unimplemented!()
    }
}

impl std::fmt::Display for Thermometer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let final_cell = self.cells.last().unwrap_or(&(0, 0));
        write!(
            f,
            "Thermometer starting at ({}, {}), ending at ({}, {})",
            self.cells[0].0, self.cells[0].1, final_cell.0, final_cell.1
        )
    }
}
