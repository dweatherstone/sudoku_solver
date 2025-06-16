use serde::{Deserialize, Serialize};

use crate::{SudokuVariant, file_parser::parse_positions, variant::Variant};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Thermometer {
    cells: Vec<(usize, usize)>,
}

impl Thermometer {
    pub fn new(cells: Vec<(usize, usize)>) -> Self {
        Thermometer { cells }
    }

    pub fn parse(data: &str) -> Option<SudokuVariant> {
        let positions = parse_positions(data).ok()?;
        Some(SudokuVariant::Thermometer(Thermometer::new(positions)))
    }
}

impl Variant for Thermometer {
    fn is_valid(&self, grid: &crate::SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        let idx = match self.cells.iter().position(|&(r, c)| r == row && c == col) {
            Some(i) => i,
            None => return true, // If (row, col) is not on the thermometer, just pass
        };

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

    fn name(&self) -> String {
        String::from("Thermometer")
    }
}
