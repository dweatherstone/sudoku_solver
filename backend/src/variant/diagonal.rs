use serde::{Deserialize, Serialize};

use crate::{SudokuVariant, variant::Variant};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Diagonal {
    cells: Vec<(usize, usize)>,
}

impl Diagonal {
    pub fn new(is_positive_diagonal: bool) -> Self {
        let cells = if is_positive_diagonal {
            (0..9).map(|i| (8 - i, i)).collect()
        } else {
            (0..9).map(|i| (i, i)).collect()
        };
        Diagonal { cells }
    }

    pub fn parse(data: &str) -> Option<SudokuVariant> {
        match data.trim().to_lowercase().as_str() {
            "positive" => Some(SudokuVariant::Diagonal(Diagonal::new(true))),
            "negative" => Some(SudokuVariant::Diagonal(Diagonal::new(false))),
            _ => None,
        }
    }
}

impl Variant for Diagonal {
    fn is_valid(&self, grid: &crate::SudokuGrid, _row: usize, _col: usize, value: u8) -> bool {
        for &(r, c) in &self.cells {
            if grid.get_cell(r, c) == value {
                return false;
            }
        }
        true
    }

    fn name(&self) -> String {
        String::from("Diagonal")
    }
}
