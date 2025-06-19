use serde::{Deserialize, Serialize};

use crate::{SudokuVariant, file_parser::parse_positions, variant::Variant};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct QuadrupleCircle {
    cells: Vec<(usize, usize)>,
    required: Vec<u8>,
}

impl QuadrupleCircle {
    pub fn new(cells: Vec<(usize, usize)>, required: Vec<u8>) -> Self {
        QuadrupleCircle { cells, required }
    }

    pub fn parse(data: &str) -> Option<SudokuVariant> {
        let parts: Vec<&str> = data.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        let cells = parse_positions(parts[0].trim()).ok()?;
        if cells.len() != 4 {
            return None;
        }
        let required_str: Vec<&str> = parts[1].split(',').collect();
        let required: Option<Vec<u8>> = required_str
            .iter()
            .map(|&r| r.trim().parse::<u8>().ok())
            .collect();
        let required = required?;
        if required.is_empty() || required.len() > 4 {
            return None;
        }
        Some(SudokuVariant::QuadrupleCircles(QuadrupleCircle::new(
            cells, required,
        )))
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

        // Check all required digits are present
        self.required.iter().all(|&d| values.contains(&d))
    }

    fn constrained_cells(&self) -> Vec<(usize, usize)> {
        self.cells.clone()
    }
}

impl std::fmt::Display for QuadrupleCircle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::from("Quadruple Circle [");
        output.push_str(
            self.cells
                .iter()
                .map(|&(r, c)| format!("({}, {})", r, c))
                .collect::<Vec<_>>()
                .join(", ")
                .as_str(),
        );
        output.push_str("] required values: [");
        let required_str = self
            .required
            .iter()
            .map(|req| req.to_string() + ", ")
            .collect::<String>();
        let required_str = required_str.trim_end_matches(", ");
        output.push_str(&required_str);
        output.push(']');
        write!(f, "{}", output)
    }
}
