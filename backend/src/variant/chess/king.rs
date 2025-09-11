use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{
    SudokuGrid, SudokuVariant,
    variant::{ALL_POSSIBILITIES, Variant, chess::get_all_cells, error::PossibilityResult},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct King {}

impl King {
    const DIRECTIONS: [(isize, isize); 8] = [
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
    ];

    pub fn new() -> Self {
        King {}
    }

    pub fn parse(_data: &str) -> Option<SudokuVariant> {
        Some(SudokuVariant::King(King::new()))
    }
}

impl Variant for King {
    fn is_valid(&self, grid: &SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        if value == 0 {
            return true;
        }
        for &(dr, dc) in Self::DIRECTIONS.iter() {
            let check_row = row as isize + dr;
            let check_col = col as isize + dc;
            if check_row < 0 || check_row > 8 || check_col < 0 || check_col > 8 {
                continue;
            }
            if grid.get_cell(check_row as usize, check_col as usize) == value {
                return false;
            }
        }
        true
    }

    fn validate_solution(&self, grid: &SudokuGrid) -> bool {
        for &(row, col) in self.constrained_cells().iter() {
            let value = grid.get_cell(row, col);
            if value == 0 {
                continue;
            }
            if !self.is_valid(grid, row, col, value) {
                return false;
            }
        }
        true
    }

    fn constrained_cells(&self) -> Vec<(usize, usize)> {
        get_all_cells()
    }

    fn get_possibilities(&self, grid: &SudokuGrid) -> PossibilityResult {
        let mut possibilities = HashMap::new();
        for &(row, col) in self.constrained_cells().iter() {
            let value = grid.get_cell(row, col);
            if value != 0 {
                possibilities.insert((row, col), vec![value]);
            } else {
                let mut values = ALL_POSSIBILITIES.to_vec();
                for &(dr, dc) in Self::DIRECTIONS.iter() {
                    let check_row = row as isize + dr;
                    let check_col = col as isize + dc;
                    if check_row < 0 || check_row > 8 || check_col < 0 || check_col > 8 {
                        continue;
                    }
                    values.retain(|&v| v != grid.get_cell(check_row as usize, check_col as usize));
                }
                possibilities.insert((row, col), values);
            }
        }
        Ok(possibilities)
    }
}

impl Display for King {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "King's move constraint")
    }
}

#[cfg(test)]
mod tests {
    //use std::collections::HashMap;

    use crate::{SudokuGrid, variant::Variant};

    use super::King;

    // fn print_hashmap(map: &HashMap<(usize, usize), Vec<u8>>) {
    //     let mut items: Vec<_> = map.iter().collect();
    //     items.sort_by_key(|&(k, _)| k);

    //     for (k, v) in items {
    //         println!("{:?}: {:?}", k, v);
    //     }
    // }

    mod is_valid {
        use super::*;

        #[test]
        fn valid_empty_middle() {
            let grid = SudokuGrid::empty();
            let king = King::new();
            for value in 1..=9 {
                assert!(king.is_valid(&grid, 4, 4, value));
            }
        }

        #[test]
        fn valid_empty_edge() {
            let grid = SudokuGrid::empty();
            let king = King::new();
            for value in 1..=9 {
                assert!(king.is_valid(&grid, 0, 0, value));
            }
        }

        #[test]
        fn valid_one_option() {
            let mut grid = SudokuGrid::empty();
            let king = King::new();
            grid.set_cell(0, 0, 1);
            grid.set_cell(0, 1, 2);
            grid.set_cell(0, 2, 3);
            grid.set_cell(1, 0, 4);
            grid.set_cell(1, 2, 5);
            grid.set_cell(2, 0, 6);
            grid.set_cell(2, 1, 7);
            grid.set_cell(2, 2, 8);
            for value in 1..9 {
                assert!(!king.is_valid(&grid, 1, 1, value));
            }
            assert!(king.is_valid(&grid, 1, 1, 9))
        }

        #[test]
        fn invalid_middle() {
            let mut grid = SudokuGrid::empty();
            let king = King::new();
            grid.set_cell(2, 2, 1);
            assert!(!king.is_valid(&grid, 1, 1, 1));
            assert!(!king.is_valid(&grid, 1, 2, 1));
            assert!(!king.is_valid(&grid, 1, 3, 1));
            assert!(!king.is_valid(&grid, 2, 1, 1));
            assert!(!king.is_valid(&grid, 2, 3, 1));
            assert!(!king.is_valid(&grid, 3, 1, 1));
            assert!(!king.is_valid(&grid, 3, 2, 1));
            assert!(!king.is_valid(&grid, 3, 3, 1));
        }

        #[test]
        fn invalid_edge() {
            let mut grid = SudokuGrid::empty();
            let king = King::new();
            grid.set_cell(0, 1, 1);
            grid.set_cell(1, 0, 2);
            grid.set_cell(1, 1, 3);
            for value in 1..4 {
                assert!(!king.is_valid(&grid, 0, 0, value));
            }
            for value in 4..=9 {
                assert!(king.is_valid(&grid, 0, 0, value));
            }
        }
    }

    mod validate_solution {
        use super::*;

        #[test]
        fn valid_solution() {
            let values = [
                [5, 7, 1, 4, 3, 9, 6, 2, 8],
                [6, 4, 8, 5, 2, 1, 3, 7, 9],
                [9, 3, 2, 6, 8, 7, 5, 4, 1],
                [4, 5, 7, 3, 9, 2, 1, 8, 6],
                [3, 1, 9, 8, 7, 6, 4, 5, 2],
                [2, 8, 6, 1, 4, 5, 9, 3, 7],
                [7, 9, 4, 2, 6, 3, 8, 1, 5],
                [8, 6, 5, 7, 1, 4, 2, 9, 3],
                [1, 2, 3, 9, 5, 8, 7, 6, 4],
            ];
            let mut grid = SudokuGrid::empty();
            let king = King::new();
            for (r, row) in values.iter().enumerate() {
                for (c, &value) in row.iter().enumerate() {
                    grid.set_cell(r, c, value);
                }
            }
            assert!(king.validate_solution(&grid));
        }

        #[test]
        fn invalid_solution() {
            let values = [
                [5, 7, 1, 4, 3, 9, 6, 2, 8],
                [6, 4, 8, 5, 2, 1, 3, 7, 9],
                [9, 3, 2, 6, 8, 7, 5, 4, 1],
                [4, 5, 7, 2, 9, 3, 1, 8, 6],
                [3, 1, 9, 8, 7, 6, 4, 5, 2],
                [2, 8, 6, 1, 4, 5, 9, 3, 7],
                [7, 9, 4, 2, 6, 3, 8, 1, 5],
                [8, 6, 5, 7, 1, 4, 2, 9, 3],
                [1, 2, 3, 9, 5, 8, 7, 6, 4],
            ];
            let mut grid = SudokuGrid::empty();
            let king = King::new();
            for (r, row) in values.iter().enumerate() {
                for (c, &value) in row.iter().enumerate() {
                    grid.set_cell(r, c, value);
                }
            }
            assert!(!king.validate_solution(&grid));
        }
    }

    mod get_possibilities {
        use std::collections::HashMap;

        use crate::variant::{ALL_POSSIBILITIES, chess::get_all_cells};

        use super::*;

        #[test]
        fn empty() {
            let grid = SudokuGrid::empty();
            let king = King::new();
            let expected = get_all_cells()
                .iter()
                .map(|&cell| (cell, ALL_POSSIBILITIES.to_vec()))
                .collect::<HashMap<_, _>>();
            assert_eq!(king.get_possibilities(&grid), Ok(expected));
        }

        #[test]
        fn partially_filled() {
            let mut grid = SudokuGrid::empty();
            let king = King::new();
            let mut value = 1;
            let mut expected = HashMap::new();
            for row in 0..3 {
                for col in 0..3 {
                    grid.set_cell(row, col, value);
                    expected.insert((row, col), vec![value]);
                    value += 1;
                }
            }
            // Add the cells around the edge
            expected.insert((0, 3), vec![1, 2, 4, 5, 7, 8, 9]);
            expected.insert((1, 3), vec![1, 2, 4, 5, 7, 8]);
            expected.insert((2, 3), vec![1, 2, 3, 4, 5, 7, 8]);
            expected.insert((3, 3), vec![1, 2, 3, 4, 5, 6, 7, 8]);
            expected.insert((3, 0), vec![1, 2, 3, 4, 5, 6, 9]);
            expected.insert((3, 1), vec![1, 2, 3, 4, 5, 6]);
            expected.insert((3, 2), vec![1, 2, 3, 4, 5, 6, 7]);
            // Remaining cells have all possibilities
            for row in 0..9 {
                for col in 0..9 {
                    expected
                        .entry((row, col))
                        .or_insert(ALL_POSSIBILITIES.to_vec());
                }
            }
            // println!("Expected:");
            // print_hashmap(&expected);
            let result = king.get_possibilities(&grid);
            // println!("Result:");
            // print_hashmap(&result);
            assert_eq!(result, Ok(expected));
        }

        #[test]
        fn fully_filled() {
            let values = [
                [5, 7, 1, 4, 3, 9, 6, 2, 8],
                [6, 4, 8, 5, 2, 1, 3, 7, 9],
                [9, 3, 2, 6, 8, 7, 5, 4, 1],
                [4, 5, 7, 3, 9, 2, 1, 8, 6],
                [3, 1, 9, 8, 7, 6, 4, 5, 2],
                [2, 8, 6, 1, 4, 5, 9, 3, 7],
                [7, 9, 4, 2, 6, 3, 8, 1, 5],
                [8, 6, 5, 7, 1, 4, 2, 9, 3],
                [1, 2, 3, 9, 5, 8, 7, 6, 4],
            ];
            let mut grid = SudokuGrid::empty();
            let king = King::new();
            let mut expected = HashMap::new();
            for (r, row) in values.iter().enumerate() {
                for (c, &value) in row.iter().enumerate() {
                    grid.set_cell(r, c, value);
                    expected.insert((r, c), vec![value]);
                }
            }
            assert_eq!(king.get_possibilities(&grid), Ok(expected));
        }
    }
}
