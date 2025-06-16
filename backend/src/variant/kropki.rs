use serde::{Deserialize, Serialize};

use crate::{SudokuGrid, SudokuVariant, file_parser::parse_positions, variant::Variant};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

    pub fn parse(data: &str) -> Option<SudokuVariant> {
        let parts: Vec<&str> = data.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        let cells = parse_positions(parts[0].trim()).ok()?;
        if cells.len() != 2 {
            return None;
        }
        let colour = match parts[1].trim().to_lowercase().as_str() {
            "white" => "white",
            "black" => "black",
            _ => return None,
        };
        Some(SudokuVariant::Kropki(KropkiDot::new(cells, colour)))
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

    fn name(&self) -> String {
        String::from("Kropki Dot")
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
enum KropkiColour {
    White,
    Black,
}
