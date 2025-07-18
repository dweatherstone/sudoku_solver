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
        // 1. If the cell isn't on the thermometer, return empty
        if !self.cells.contains(&(row, col)) {
            return HashMap::new();
        }

        // 2. Gather known values with their positions (index on the thermometer)
        let known_cells: HashMap<usize, u8> = self
            .cells
            .iter()
            .enumerate()
            .filter_map(|(i, &(r, c))| {
                let val = grid.get_cell(r, c);
                if val == 0 { None } else { Some((i, val)) }
            })
            .collect();

        let mut possibilities = HashMap::new();

        for (i, &(r, c)) in self.cells.iter().enumerate() {
            if known_cells.contains_key(&i) {
                continue; // already known, skip
            }

            // Compute min and max values allowed at this position
            // Based on known values before and after

            // Min = 1 + max known value before
            let min_val = known_cells
                .iter()
                .filter(|&(&idx, _)| idx < i)
                .map(|(&idx, &val)| val + (i - idx) as u8)
                .max()
                .unwrap_or(1 + i as u8);

            // Find tightest max based on any known value after
            let max_val = known_cells
                .iter()
                .filter(|&(&idx, _)| idx > i)
                .map(|(&idx, &val)| val - (idx - i) as u8)
                .min()
                .unwrap_or(9 - (self.length - i - 1) as u8);

            if min_val <= max_val {
                possibilities.insert((r, c), (min_val..=max_val).collect());
            } else {
                possibilities.insert((r, c), vec![]);
            }
        }
        possibilities
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

#[cfg(test)]
mod tests {
    use crate::SudokuGrid;

    use super::*;

    #[test]
    fn test_basic_get_possibilities() {
        let mut grid = SudokuGrid::empty();
        let thermometer = create_thermometer();
        grid.set_cell(0, 2, 5);
        let result = thermometer.get_possibilities(&grid, 0, 2);
        assert_eq!(result.len(), 3);
        assert_eq!(result.get(&(0, 1)).unwrap(), &vec![1, 2, 3, 4]);
        assert_eq!(result.get(&(0, 3)).unwrap(), &vec![6, 7, 8]);
        assert_eq!(result.get(&(0, 4)).unwrap(), &vec![7, 8, 9]);
    }

    #[test]
    fn test_get_possiblilities_value_at_start() {
        let mut grid = SudokuGrid::empty();
        let thermometer = create_thermometer();
        grid.set_cell(0, 1, 3); // Set first cell
        let result = thermometer.get_possibilities(&grid, 0, 1);
        assert_eq!(result.len(), 3);
        assert_eq!(result.get(&(0, 2)).unwrap(), &vec![4, 5, 6, 7]);
        assert_eq!(result.get(&(0, 3)).unwrap(), &vec![5, 6, 7, 8]);
        assert_eq!(result.get(&(0, 4)).unwrap(), &vec![6, 7, 8, 9]);
    }

    #[test]
    fn test_get_possiblilities_value_at_end() {
        let mut grid = SudokuGrid::empty();
        let thermometer = create_thermometer();
        grid.set_cell(0, 4, 6); // Set first cell
        let result = thermometer.get_possibilities(&grid, 0, 1);
        assert_eq!(result.len(), 3);
        assert_eq!(result.get(&(0, 1)).unwrap(), &vec![1, 2, 3]);
        assert_eq!(result.get(&(0, 2)).unwrap(), &vec![2, 3, 4]);
        assert_eq!(result.get(&(0, 3)).unwrap(), &vec![3, 4, 5]);
    }

    #[test]
    fn test_get_possibilities_two_known() {
        let mut grid = SudokuGrid::empty();
        let thermometer = create_thermometer();
        grid.set_cell(0, 1, 3);
        grid.set_cell(0, 4, 7);
        let result = thermometer.get_possibilities(&grid, 0, 1);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(&(0, 2)).unwrap(), &vec![4, 5]);
        assert_eq!(result.get(&(0, 3)).unwrap(), &vec![5, 6]);
    }

    #[test]
    fn test_get_possibilities_not_present() {
        let mut grid = SudokuGrid::empty();
        let thermometer = create_thermometer();
        grid.set_cell(1, 1, 1);
        let result = thermometer.get_possibilities(&grid, 1, 1);
        assert!(result.is_empty());
    }

    fn create_thermometer() -> Thermometer {
        Thermometer::new(vec![(0, 1), (0, 2), (0, 3), (0, 4)])
    }
}
