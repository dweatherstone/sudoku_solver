use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{SudokuVariant, file_parser::parse_positions, variant::Variant};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GermanWhisper {
    cells: Vec<(usize, usize)>,
    is_circular: bool,
}

impl GermanWhisper {
    pub fn new(cells: Vec<(usize, usize)>, is_circular: bool) -> Self {
        GermanWhisper { cells, is_circular }
    }

    pub fn parse(data: &str) -> Option<SudokuVariant> {
        let splits = data.split(":").collect::<Vec<_>>();
        if splits.len() == 1 {
            let positions = parse_positions(data).ok()?;
            return Some(SudokuVariant::GermanWhisper(GermanWhisper::new(
                positions, false,
            )));
        } else if splits.len() == 2 && splits[1].to_lowercase().trim() == "circular" {
            let positions = parse_positions(splits[0]).ok()?;
            return Some(SudokuVariant::GermanWhisper(GermanWhisper::new(
                positions, true,
            )));
        } else {
            return None;
        }
    }
}

impl Variant for GermanWhisper {
    fn is_valid(&self, grid: &crate::SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        let cell_idx = match self.cells.iter().position(|&(r, c)| r == row && c == col) {
            Some(idx) => idx,
            None => return true, // Cell is not on the line, so return early
        };
        let max_idx = self.cells.len() - 1;
        // Check following cell
        if cell_idx < max_idx {
            let next_val = grid.get_cell(self.cells[cell_idx + 1].0, self.cells[cell_idx + 1].1);
            if next_val != 0 && value.abs_diff(next_val) < 5 {
                return false;
            }
        }
        // Check previous cell
        if cell_idx > 0 {
            let prev_val = grid.get_cell(self.cells[cell_idx - 1].0, self.cells[cell_idx - 1].1);
            if prev_val != 0 && value.abs_diff(prev_val) < 5 {
                return false;
            }
        }
        // If circular, and at the end of the line, check the other end
        if self.is_circular && (cell_idx == 0 || cell_idx == max_idx) {
            let other_val = if cell_idx == 0 {
                grid.get_cell(self.cells[max_idx].0, self.cells[max_idx].1)
            } else {
                grid.get_cell(self.cells[0].0, self.cells[0].1)
            };
            if other_val != 0 && value.abs_diff(other_val) < 5 {
                return false;
            }
        }
        true
    }

    fn constrained_cells(&self) -> Vec<(usize, usize)> {
        self.cells.clone()
    }

    fn validate_solution(&self, grid: &crate::SudokuGrid) -> bool {
        for window in self.cells.windows(2) {
            let val0 = grid.get_cell(window[0].0, window[0].1) as i8;
            let val1 = grid.get_cell(window[1].0, window[1].1) as i8;
            if val0 == 0 || val1 == 0 {
                return false;
            }
            if (val0 - val1).abs() < 5 {
                return false;
            }
        }
        if self.is_circular && self.cells.len() > 1 {
            let first = self.cells.first().unwrap();
            let last = self.cells.last().unwrap();
            let first_val = grid.get_cell(first.0, first.1) as i8;
            let last_val = grid.get_cell(last.0, last.1) as i8;
            if (first_val - last_val).abs() < 5 {
                return false;
            }
        }
        true
    }

    fn get_possibilities(
        &self,
        grid: &crate::SudokuGrid,
        row: usize,
        col: usize,
    ) -> HashMap<(usize, usize), Vec<u8>> {
        let Some(start_idx) = self.cells.iter().position(|&(r, c)| (r, c) == (row, col)) else {
            return HashMap::new(); // Not on the line
        };
        let mut possibilities = HashMap::new();
        let len = self.cells.len();

        // Cache current values from the grid
        let cell_values: Vec<u8> = self
            .cells
            .iter()
            .map(|&(r, c)| grid.get_cell(r, c))
            .collect();

        // Alternating group propogation (low/high groups)
        const LOW_VALS: [u8; 4] = [1, 2, 3, 4];
        const HIGH_VALS: [u8; 4] = [6, 7, 8, 9];

        let updated_value = cell_values[start_idx];
        if updated_value == 0 {
            return HashMap::new(); // Sanity: shouldn't be called before setting value
        }

        // Determine the parity group of the starting value
        let start_group = if LOW_VALS.contains(&updated_value) {
            0
        } else {
            1
        };

        // Forward propogation
        if start_idx < len - 1 {
            for i in start_idx + 1..len {
                let expected_group = (i - start_idx + start_group) % 2;
                let coord = self.cells[i];
                match cell_values[i] {
                    0 => {
                        let options = if expected_group == 0 {
                            LOW_VALS.to_vec()
                        } else {
                            HIGH_VALS.to_vec()
                        };
                        possibilities.entry(coord).or_insert(options);
                    }
                    val => {
                        let valid_group = if expected_group == 0 {
                            &LOW_VALS
                        } else {
                            &HIGH_VALS
                        };
                        if !valid_group.contains(&val) || val.abs_diff(cell_values[i - 1]) < 5 {
                            if cell_values[i] == 0 {
                                possibilities.insert(coord, vec![]); // Conflict
                            }
                        }
                    }
                }
            }
        }

        // Backward propogation
        if start_idx > 0 {
            for i in (0..start_idx).rev() {
                let expected_group = (start_idx - i + start_group) % 2;
                let coord = self.cells[i];
                match cell_values[i] {
                    0 => {
                        let options = if expected_group == 0 {
                            LOW_VALS.to_vec()
                        } else {
                            HIGH_VALS.to_vec()
                        };
                        possibilities.entry(coord).or_insert(options);
                    }
                    val => {
                        let valid_group = if expected_group == 0 {
                            &LOW_VALS
                        } else {
                            &HIGH_VALS
                        };
                        if !valid_group.contains(&val) || val.abs_diff(cell_values[i + 1]) < 5 {
                            if cell_values[i] != 0 {
                                possibilities.insert(coord, vec![]); // Conflict
                            }
                        }
                    }
                }
            }
        }

        // Filter with actual adjacent neighbours (including circular)
        for (i, &(r, c)) in self.cells.iter().enumerate() {
            if cell_values[i] != 0 {
                continue; // Skip already filled cells
            }
            let mut neighbours = vec![];

            if i > 0 && cell_values[i - 1] != 0 {
                neighbours.push(cell_values[i - 1]);
            }
            if i + 1 < len && cell_values[i + 1] != 0 {
                neighbours.push(cell_values[i + 1]);
            }
            // Circular checks
            if self.is_circular {
                if i == 0 && cell_values[len - 1] != 0 {
                    neighbours.push(cell_values[len - 1]);
                } else if i == len - 1 && cell_values[0] != 0 {
                    neighbours.push(cell_values[0]);
                }
            }
            if !neighbours.is_empty() {
                let entry = possibilities
                    .entry((r, c))
                    .or_insert_with(|| (1..=9).collect());
                entry.retain(|&v| neighbours.iter().all(|&n| v.abs_diff(n) >= 5));

                if entry.is_empty() {
                    // We still want this cell in the map with an empty vec
                    possibilities.insert((r, c), vec![]);
                }
            }
        }

        possibilities
    }
}

impl std::fmt::Display for GermanWhisper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cells_str = self
            .cells
            .iter()
            .map(|&(r, c)| format!("({r}, {c})"))
            .collect::<Vec<_>>()
            .join(", ");
        write!(
            f,
            "German Whispers: [{cells_str}] {}",
            if self.is_circular { " is circular" } else { "" }
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::SudokuGrid;

    fn make_grid_with_values(values: Vec<Vec<u8>>) -> SudokuGrid {
        let mut grid = SudokuGrid::empty();
        for (r, row) in values.into_iter().enumerate() {
            for (c, v) in row.into_iter().enumerate() {
                if v != 0 {
                    grid.set_cell(r, c, v);
                }
            }
        }
        grid
    }

    mod validate_solution {
        use super::*;
        use crate::variant::{GermanWhisper, Variant};

        #[test]
        fn test_linear_line() {
            let grid = make_grid_with_values(vec![vec![1, 6, 1]]); // row 0
            let line = GermanWhisper::new(vec![(0, 0), (0, 1), (0, 2)], false);
            assert!(line.validate_solution(&grid));
        }

        #[test]
        fn test_difference_too_small() {
            let grid = make_grid_with_values(vec![vec![1, 3, 9]]);
            let line = GermanWhisper::new(vec![(0, 0), (0, 1), (0, 2)], false);
            assert!(!line.validate_solution(&grid));
        }

        #[test]
        fn test_contains_zero_value() {
            let grid = make_grid_with_values(vec![vec![1, 0, 2]]);
            let line = GermanWhisper::new(vec![(0, 0), (0, 1), (0, 2)], false);
            assert!(!line.validate_solution(&grid));
        }

        #[test]
        fn test_circular_line_valid() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(3, 2, 1);
            grid.set_cell(4, 1, 9);
            grid.set_cell(4, 3, 2);
            grid.set_cell(5, 2, 8);
            let line = GermanWhisper::new(vec![(3, 2), (4, 1), (4, 3), (5, 2)], true);
            assert!(line.validate_solution(&grid));
        }

        #[test]
        fn test_circular_line_invalid() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(3, 2, 4);
            grid.set_cell(4, 1, 9);
            grid.set_cell(4, 3, 2);
            grid.set_cell(5, 2, 8);
            let line = GermanWhisper::new(vec![(3, 2), (4, 1), (4, 3), (5, 2)], true);
            assert!(!line.validate_solution(&grid));
        }
    }

    mod is_valid {
        use super::*;
        use crate::variant::{GermanWhisper, Variant};

        #[test]
        fn test_not_on_line() {
            let grid = make_grid_with_values(vec![vec![1, 2, 0]]);
            let line = GermanWhisper::new(vec![(0, 0), (0, 1)], false);
            assert!(line.is_valid(&grid, 0, 2, 3));
        }

        #[test]
        fn test_placement_in_middle_of_line() {
            let grid = make_grid_with_values(vec![
                vec![1, 0, 2], // Want to check if placing 6 at (0,1) is OK
            ]);
            let line = GermanWhisper::new(vec![(0, 0), (0, 1), (0, 2)], false);
            assert!(line.is_valid(&grid, 0, 1, 9));
            assert!(!line.is_valid(&grid, 0, 1, 6));
            assert!(!line.is_valid(&grid, 0, 1, 4));
            assert!(line.is_valid(&grid, 0, 1, 8));
            assert!(line.is_valid(&grid, 0, 1, 7));
        }

        #[test]
        fn test_placement_end_of_line() {
            let grid = make_grid_with_values(vec![vec![0, 7, 0]]);
            let line = GermanWhisper::new(vec![(0, 0), (0, 1), (0, 2)], false);
            assert!(line.is_valid(&grid, 0, 0, 1));
            assert!(line.is_valid(&grid, 0, 0, 2));
            assert!(line.is_valid(&grid, 0, 2, 1));
            assert!(line.is_valid(&grid, 0, 2, 2));
            assert!(!line.is_valid(&grid, 0, 0, 3));
            assert!(!line.is_valid(&grid, 0, 2, 3));
            assert!(!line.is_valid(&grid, 0, 0, 9));
            assert!(!line.is_valid(&grid, 0, 2, 9));
        }

        #[test]
        fn test_circular_line() {
            let grid = make_grid_with_values(vec![vec![0, 7, 2, 6]]);
            let line = GermanWhisper::new(vec![(0, 0), (0, 1), (0, 2), (0, 3)], true);
            assert!(!line.is_valid(&grid, 0, 0, 8));
            assert!(line.is_valid(&grid, 0, 0, 1));
            assert!(!line.is_valid(&grid, 0, 0, 2));
        }
    }

    mod get_possibilities {
        use crate::{
            SudokuGrid,
            variant::{GermanWhisper, Variant},
        };

        #[test]
        fn test_not_on_line() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 0, 5);
            let line = GermanWhisper::new(vec![(0, 1), (0, 2)], false);
            let result = line.get_possibilities(&grid, 0, 0);
            assert!(result.is_empty());
        }

        #[test]
        fn test_one_neighbour_not_set() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 0, 3);
            let whisper = GermanWhisper::new(vec![(0, 0), (0, 1)], false);

            let result = whisper.get_possibilities(&grid, 0, 0);
            assert_eq!(result.len(), 1);
            assert_eq!(result.get(&(0, 1)).unwrap(), &vec![8, 9]);
        }

        #[test]
        fn test_middle_of_line_value_set() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 1, 7);
            let whisper = GermanWhisper::new(vec![(0, 0), (0, 1), (0, 2)], false);

            let result = whisper.get_possibilities(&grid, 0, 1);
            assert_eq!(result.len(), 2);
            let expected = vec![1, 2];
            assert_eq!(result.get(&(0, 0)).unwrap(), &expected);
            assert_eq!(result.get(&(0, 2)).unwrap(), &expected);
        }

        #[test]
        fn test_circular_line() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 0, 6);
            let whisper = GermanWhisper::new(vec![(0, 0), (0, 1), (0, 2), (0, 3)], true);
            let result = whisper.get_possibilities(&grid, 0, 0);
            assert_eq!(result.len(), 3);
            assert_eq!(result.get(&(0, 1)).unwrap(), &vec![1]);
            assert_eq!(result.get(&(0, 3)).unwrap(), &vec![1]);
            assert_eq!(result.get(&(0, 2)).unwrap(), &vec![6, 7, 8, 9]);
        }

        #[test]
        fn test_single_cell_line() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(4, 4, 5);
            let whisper = GermanWhisper::new(vec![(4, 4)], false);
            assert!(whisper.get_possibilities(&grid, 4, 4).is_empty());
        }

        #[test]
        fn no_valid_neighbours() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 0, 5);
            let whisper = GermanWhisper::new(vec![(0, 0), (0, 1)], false);
            let result = whisper.get_possibilities(&grid, 0, 0);
            assert_eq!(result.len(), 1);
            assert_eq!(result.get(&(0, 1)), Some(&vec![]));
        }

        #[test]
        fn skip_already_filled_cells() {
            let mut grid = SudokuGrid::empty();
            let whisper = GermanWhisper {
                cells: vec![(2, 0), (2, 1), (2, 2)],
                is_circular: false,
            };

            grid.set_cell(2, 1, 6); // Set center
            grid.set_cell(2, 0, 0); // Unset
            grid.set_cell(2, 2, 7); // Already filled

            let result = whisper.get_possibilities(&grid, 2, 1);
            println!("Result: {result:?}");
            assert!(result.contains_key(&(2, 0)));
            assert!(!result.contains_key(&(2, 2)));
        }

        #[test]
        fn test_backpropogation_from_late_value() {
            let mut grid = SudokuGrid::empty();
            grid.set_cell(0, 2, 1);
            let whisper = GermanWhisper::new(vec![(0, 0), (0, 1), (0, 2)], false);
            let result = whisper.get_possibilities(&grid, 0, 2);
            assert_eq!(result.get(&(0, 1)).unwrap(), &vec![6, 7, 8, 9]);
            assert_eq!(result.get(&(0, 0)).unwrap(), &vec![1, 2, 3, 4]);
        }
    }
}
