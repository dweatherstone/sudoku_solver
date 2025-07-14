use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

use crate::{SudokuVariant, file_parser::parse_positions, variant::Variant};

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

    fn get_possibilities(
        &self,
        grid: &crate::SudokuGrid,
        row: usize,
        col: usize,
    ) -> std::collections::HashMap<(usize, usize), Vec<u8>> {
        // If (row, col) is not on the dot, just pass
        if !self.cells.contains(&(row, col)) {
            return HashMap::new();
        }
        let value = grid.get_cell(row, col);
        if value == 0 {
            return HashMap::new();
        }

        let [(r1, c1), (r2, c2)] = self.cells;
        let (other_row, other_col) = if (row, col) == (r1, c1) {
            (r2, c2)
        } else {
            (r1, c1)
        };

        if grid.get_cell(other_row, other_col) != 0 {
            return HashMap::new();
        }

        let other_value = match self.flavour {
            XVFlavour::V => 5 - value,
            XVFlavour::X => 10 - value,
        };
        let mut result = HashMap::new();
        result.insert((other_row, other_col), vec![other_value]);
        result
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
