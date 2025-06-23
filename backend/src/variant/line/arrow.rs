use serde::{Deserialize, Serialize};

use crate::{SudokuVariant, file_parser::parse_positions, variant::Variant};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Arrow {
    cells: Vec<(usize, usize)>,
}

impl Arrow {
    pub fn new(cells: Vec<(usize, usize)>) -> Self {
        Arrow { cells }
    }

    pub fn parse(data: &str) -> Option<SudokuVariant> {
        let cells = parse_positions(data).ok()?;
        Some(SudokuVariant::Arrow(Arrow::new(cells)))
    }
}

impl Variant for Arrow {
    fn is_valid(&self, grid: &crate::SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        if !self.cells.contains(&(row, col)) {
            return true;
        }
        // Clone current values, and insert proposed value
        let mut values = self
            .cells
            .iter()
            .map(|&(r, c)| grid.get_cell(r, c))
            .collect::<Vec<u8>>();

        // Find the index of (row, col) in the arrow
        if let Some(pos) = self.cells.iter().position(|&(r, c)| r == row && c == col) {
            values[pos] = value;
        }

        let head_value = values[0];
        let body_values = &values[1..];

        let known_sum: u8 = body_values.iter().sum();
        let unknown_count = body_values.iter().filter(|&&v| v == 0).count();

        // If the head cell is 0 (unknown), we can only check whether the body can *possibly* sum to a valid head (<=9)
        if head_value == 0 {
            // If body is fully filled but head is unknown, we can't validate yet
            if unknown_count == 0 {
                // Head must be equal to the known body sum and nonzero
                return known_sum <= 9;
            }
            // Otherwise, just check that the body sum is still in the realm of possibility
            // (realistically not needed unless you want to prune impossible sums)
            return true;
        }

        // Head is known, apply tighter constraint
        if known_sum > head_value {
            return false; // overshoot
        }

        // Miniumum possible sum for body must be <= head
        let minimum_possible_sum = known_sum + unknown_count as u8;
        if minimum_possible_sum > head_value {
            return false;
        }

        // Final check: if all body digits are known, they must sum to head
        if unknown_count == 0 {
            return known_sum == head_value;
        }

        // Stil consistent
        true
    }

    fn validate_solution(&self, grid: &crate::SudokuGrid) -> bool {
        if self.cells.len() < 2 {
            return false;
        }
        let head = self.cells[0];
        let head_value = grid.get_cell(head.0, head.1);
        if head_value == 0 {
            return false;
        }
        let body_values = self
            .cells
            .iter()
            .skip(1)
            .map(|&(r, c)| grid.get_cell(r, c))
            .collect::<Vec<_>>();
        if body_values.contains(&0) {
            return false;
        }

        body_values.iter().sum::<u8>() == head_value
    }

    fn constrained_cells(&self) -> Vec<(usize, usize)> {
        self.cells.clone()
    }
}

impl std::fmt::Display for Arrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::from("Arrow: head = ");
        output.push_str(&format!("({}, {})", self.cells[0].0, self.cells[0].1));
        output.push_str(", arrow: [");
        output.push_str(
            self.cells
                .iter()
                .skip(1)
                .map(|&(r, c)| format!("({}, {})", r, c))
                .collect::<Vec<_>>()
                .join(", ")
                .as_str(),
        );
        output.push(']');
        write!(f, "{}", output)
    }
}
