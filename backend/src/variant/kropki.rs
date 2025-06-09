use serde::{Deserialize, Serialize};

use crate::{SudokuGrid, variant::Variant};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KropkiDot {
    cells: [(usize, usize); 2],
    colour: KropkiColour,
}

impl KropkiDot {
    pub fn new(cells: Vec<(usize, usize)>, colour: &str) -> KropkiDot {
        let kropki_colour = match colour.to_lowercase().as_str() {
            "black" => KropkiColour::Black,
            _ => KropkiColour::White,
        };
        KropkiDot {
            cells: [cells[0], cells[1]],
            colour: kropki_colour,
        }
    }
}

impl Variant for KropkiDot {
    fn is_valid(&self, grid: &SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        // If (row, col) is not on the dot, just pass
        if !self.cells.contains(&(row, col)) {
            return true;
        }

        let other_val = if (row, col) == self.cells[0] {
            grid.get_cell(self.cells[1].0, self.cells[1].1)
        } else {
            grid.get_cell(self.cells[0].0, self.cells[0].1)
        };

        if other_val == 0 {
            return true;
        }

        match self.colour {
            KropkiColour::Black => value * 2 == other_val || other_val * 2 == value,
            KropkiColour::White => value + 1 == other_val || value - 1 == other_val,
        }
    }

    fn affected_cells(&self) -> Vec<(usize, usize)> {
        Vec::from(self.cells)
    }

    fn name(&self) -> String {
        String::from("Kropki Dot")
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum KropkiColour {
    White,
    Black,
}
