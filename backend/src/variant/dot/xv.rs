use std::{collections::HashMap, fmt};

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
pub struct XVDot {
    cells: [(usize, usize); 2],
    flavour: XVFlavour,
}

impl XVDot {
    pub fn new(cells: Vec<(usize, usize)>, xv_flavour: &str) -> XVDot {
        let flavour = match xv_flavour.to_lowercase().as_str() {
            "x" => XVFlavour::X,
            _ => XVFlavour::V,
        };
        XVDot {
            cells: [cells[0], cells[1]],
            flavour,
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
        let flavour = match parts[1].trim().to_lowercase().as_str() {
            "x" => "x",
            "v" => "v",
            _ => return None,
        };
        Some(SudokuVariant::XVDot(XVDot::new(cells, flavour)))
    }
}

impl Variant for XVDot {
    fn is_valid(&self, grid: &crate::SudokuGrid, row: usize, col: usize, value: u8) -> bool {
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

        match self.flavour {
            XVFlavour::X => value + other_val == 10,
            XVFlavour::V => value + other_val == 5,
        }
    }

    fn validate_solution(&self, grid: &crate::SudokuGrid) -> bool {
        let val1 = grid.get_cell(self.cells[0].0, self.cells[0].1);
        let val2 = grid.get_cell(self.cells[1].0, self.cells[1].1);

        // Check both cells are filled
        if val1 == 0 || val2 == 0 {
            return false;
        }

        // Check the relationship is satisfied
        match self.flavour {
            XVFlavour::X => val1 + val2 == 10,
            XVFlavour::V => val1 + val2 == 5,
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
            match self.flavour {
                XVFlavour::X => {
                    possibilities.insert(self.cells[0], ALL_POSSIBILITIES.to_vec());
                    possibilities.insert(self.cells[1], ALL_POSSIBILITIES.to_vec());
                }
                XVFlavour::V => {
                    possibilities.insert(self.cells[0], vec![1, 2, 3, 4]);
                    possibilities.insert(self.cells[1], vec![1, 2, 3, 4]);
                }
            }
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
            match self.flavour {
                XVFlavour::V => {
                    if 5 - (known_value as i8) < 1 {
                        let reason = format!(
                            "No possible values for V dot based on other cell value of {known_value}"
                        );
                        return Err(VariantContradiction::NoPossibilities {
                            cell: self.cells[(known_index + 1) % 2],
                            variant: "XVDot:V",
                            reason,
                        });
                    }
                    possibilities.insert(self.cells[(known_index + 1) % 2], vec![5 - known_value]);
                }
                XVFlavour::X => {
                    if known_value == 5 {
                        return Err(VariantContradiction::NoPossibilities {
                            cell: self.cells[(known_index + 1) % 2],
                            variant: "XVDot:X",
                            reason: String::from(
                                "No possible values for X dot based on other cell value of 5",
                            ),
                        });
                    }
                    possibilities.insert(self.cells[(known_index + 1) % 2], vec![10 - known_value]);
                }
            }
        }

        Ok(possibilities)
    }
}

impl fmt::Display for XVDot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cell_str = self
            .cells
            .iter()
            .map(|&(r, c)| format!("({r}, {c})"))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "XVDot: [{}], {}", cell_str, self.flavour)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
enum XVFlavour {
    X,
    V,
}

impl fmt::Display for XVFlavour {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XVFlavour::X => write!(f, "X"),
            XVFlavour::V => write!(f, "V"),
        }
    }
}
