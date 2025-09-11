use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    SudokuGrid, SudokuVariant,
    file_parser::parse_positions,
    variant::{
        ALL_POSSIBILITIES, Variant,
        error::{PossibilityResult, VariantContradiction},
    },
};

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

    fn validate_solution(&self, grid: &SudokuGrid) -> bool {
        let val1 = grid.get_cell(self.cells[0].0, self.cells[0].1);
        let val2 = grid.get_cell(self.cells[1].0, self.cells[1].1);

        // Check both cells are filled
        if val1 == 0 || val2 == 0 {
            return false;
        }

        // Check the relationship is satisfied
        match self.colour {
            KropkiColour::Black => val1 * 2 == val2 || val2 * 2 == val1,
            KropkiColour::White => val1 + 1 == val2 || val1 - 1 == val2,
        }
    }

    fn constrained_cells(&self) -> Vec<(usize, usize)> {
        vec![self.cells[0], self.cells[1]]
    }

    fn get_possibilities(&self, grid: &SudokuGrid) -> PossibilityResult {
        let [(r1, c1), (r2, c2)] = self.cells;
        let val1 = grid.get_cell(r1, c1);
        let val2 = grid.get_cell(r2, c2);
        let mut possibilities = HashMap::new();
        // Neither value is known, so just return all possibilities for both
        if val1 == 0 && val2 == 0 {
            possibilities.insert(self.cells[0], ALL_POSSIBILITIES.to_vec());
            possibilities.insert(self.cells[1], ALL_POSSIBILITIES.to_vec());
        }
        // If both are already known, then just return the known value vector
        else if val1 != 0 && val2 != 0 {
            possibilities.insert(self.cells[0], vec![val1]);
            possibilities.insert(self.cells[1], vec![val2]);
        }
        // One value is known, the other is not
        else {
            let known_value = if val1 == 0 { val2 } else { val1 };
            let known_index: usize = if val1 == 0 { 1 } else { 0 };
            possibilities.insert(self.cells[known_index], vec![known_value]);
            let mut poss = vec![];
            match self.colour {
                KropkiColour::White => {
                    if known_value > 1 {
                        poss.push(known_value - 1);
                    }
                    if known_value < 9 {
                        poss.push(known_value + 1);
                    }
                }
                KropkiColour::Black => {
                    if known_value % 2 == 0 {
                        poss.push(known_value / 2);
                    }
                    if known_value * 2 < 9 {
                        poss.push(known_value * 2);
                    }
                }
            }
            if poss.is_empty() {
                let reason =
                    format!("No possible values based on other cell value of {known_value}");
                return Err(VariantContradiction::NoPossibilities {
                    cell: self.cells[(known_index + 1) % 2],
                    variant: "KropkiDot",
                    reason,
                });
            }
            possibilities.insert(self.cells[(known_index + 1) % 2], poss);
        }
        Ok(possibilities)
    }
}

impl std::fmt::Display for KropkiDot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::from("Kropki dot [");
        output.push_str(
            self.cells
                .iter()
                .map(|&(r, c)| format!("({r}, {c})"))
                .collect::<Vec<_>>()
                .join(", ")
                .as_str(),
        );
        output.push_str(&format!("] {}", self.colour));
        write!(f, "{output}")
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
enum KropkiColour {
    White,
    Black,
}

impl std::fmt::Display for KropkiColour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KropkiColour::Black => write!(f, "black"),
            KropkiColour::White => write!(f, "white"),
        }
    }
}
