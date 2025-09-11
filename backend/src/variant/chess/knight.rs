use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{
    SudokuGrid, SudokuVariant,
    variant::{ALL_POSSIBILITIES, Variant, chess::get_all_cells, error::PossibilityResult},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Knight {}

impl Knight {
    const DIRECTIONS: [(isize, isize); 8] = [
        (-2, -1),
        (-2, 1),
        (-1, -2),
        (-1, 2),
        (1, -2),
        (1, 2),
        (2, -1),
        (2, 1),
    ];

    pub fn new() -> Self {
        Knight {}
    }

    pub fn parse(_data: &str) -> Option<SudokuVariant> {
        Some(SudokuVariant::Knight(Knight::new()))
    }
}

impl Variant for Knight {
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

impl Display for Knight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Knight's move constraint")
    }
}

#[cfg(test)]
mod tests {
    // use std::collections::HashMap;

    use crate::{SudokuGrid, variant::Variant};

    use super::Knight;

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
            let knight = Knight::new();
            for value in 1..=9 {
                assert!(knight.is_valid(&grid, 4, 4, value));
            }
        }

        #[test]
        fn valid_empty_edge() {
            let grid = SudokuGrid::empty();
            let knight = Knight::new();
            for value in 1..=9 {
                assert!(knight.is_valid(&grid, 0, 0, value));
            }
        }

        #[test]
        fn valid_one_option() {
            let mut grid = SudokuGrid::empty();
            let knight = Knight::new();
            grid.set_cell(2, 3, 1);
            grid.set_cell(2, 5, 2);
            grid.set_cell(3, 2, 3);
            grid.set_cell(3, 6, 4);
            grid.set_cell(5, 2, 5);
            grid.set_cell(5, 6, 6);
            grid.set_cell(6, 3, 7);
            grid.set_cell(6, 5, 8);
            for value in 1..9 {
                assert!(!knight.is_valid(&grid, 4, 4, value));
            }
            assert!(knight.is_valid(&grid, 1, 1, 9));
        }

        #[test]
        fn invalid_middle() {
            let mut grid = SudokuGrid::empty();
            let knight = Knight::new();
            grid.set_cell(4, 4, 1);
            assert!(!knight.is_valid(&grid, 2, 3, 1));
            assert!(!knight.is_valid(&grid, 2, 5, 1));
            assert!(!knight.is_valid(&grid, 3, 2, 1));
            assert!(!knight.is_valid(&grid, 3, 6, 1));
            assert!(!knight.is_valid(&grid, 5, 2, 1));
            assert!(!knight.is_valid(&grid, 5, 6, 1));
            assert!(!knight.is_valid(&grid, 6, 3, 1));
            assert!(!knight.is_valid(&grid, 6, 5, 1));
        }

        #[test]
        fn invalid_edge() {
            let mut grid = SudokuGrid::empty();
            let knight = Knight::new();
            grid.set_cell(1, 2, 1);
            grid.set_cell(2, 1, 2);
            for value in 1..3 {
                assert!(!knight.is_valid(&grid, 0, 0, value));
            }
            for value in 3..=9 {
                assert!(knight.is_valid(&grid, 0, 0, value));
            }
        }
    }

    mod validate_solution {
        use super::*;

        #[test]
        fn valid_solution() {
            let values = [
                [8, 1, 2, 9, 7, 3, 5, 4, 6],
                [9, 6, 3, 5, 4, 8, 2, 7, 1],
                [7, 4, 5, 1, 6, 2, 8, 9, 3],
                [4, 8, 1, 7, 3, 5, 9, 6, 2],
                [6, 3, 7, 2, 8, 9, 4, 1, 5],
                [2, 5, 9, 6, 1, 4, 7, 3, 8],
                [1, 9, 8, 3, 5, 7, 6, 2, 4],
                [3, 7, 4, 8, 2, 6, 1, 5, 9],
                [5, 2, 6, 4, 9, 1, 3, 8, 7],
            ];
            let mut grid = SudokuGrid::empty();
            let knight = Knight::new();
            for (r, row) in values.iter().enumerate() {
                for (c, &value) in row.iter().enumerate() {
                    grid.set_cell(r, c, value);
                }
            }
            assert!(knight.validate_solution(&grid));
        }

        #[test]
        fn invalid_solution() {
            let values = [
                [4, 1, 2, 9, 7, 3, 5, 8, 6],
                [9, 6, 3, 5, 4, 8, 2, 7, 1],
                [7, 4, 5, 1, 6, 2, 8, 9, 3],
                [4, 8, 1, 7, 3, 5, 9, 6, 2],
                [6, 3, 7, 2, 8, 9, 4, 1, 5],
                [2, 5, 9, 6, 1, 4, 7, 3, 8],
                [1, 9, 8, 3, 5, 7, 6, 2, 4],
                [3, 7, 4, 8, 2, 6, 1, 5, 9],
                [5, 2, 6, 4, 9, 1, 3, 8, 7],
            ];
            let mut grid = SudokuGrid::empty();
            let knight = Knight::new();
            for (r, row) in values.iter().enumerate() {
                for (c, &value) in row.iter().enumerate() {
                    grid.set_cell(r, c, value);
                }
            }
            assert!(!knight.validate_solution(&grid));
        }
    }

    mod get_possibilities {
        use std::collections::HashMap;

        use crate::variant::{ALL_POSSIBILITIES, chess::get_all_cells};

        use super::*;

        #[test]
        fn empty() {
            let grid = SudokuGrid::empty();
            let knight = Knight::new();
            let expected = get_all_cells()
                .iter()
                .map(|&cell| (cell, ALL_POSSIBILITIES.to_vec()))
                .collect::<HashMap<_, _>>();
            assert_eq!(knight.get_possibilities(&grid), Ok(expected));
        }

        #[test]
        fn partially_filled() {
            let mut grid = SudokuGrid::empty();
            let knight = Knight::new();
            let mut value = 1;
            let mut expected = HashMap::new();
            for row in 0..3 {
                for col in 0..3 {
                    grid.set_cell(row, col, value);
                    expected.insert((row, col), vec![value]);
                    value += 1;
                }
            }
            // Add the cells around the edge to expected
            expected.insert((0, 3), vec![1, 2, 3, 4, 6, 7, 8]);
            expected.insert((0, 4), vec![1, 2, 3, 4, 5, 7, 8, 9]);
            expected.insert((1, 3), vec![1, 3, 4, 5, 6, 7, 9]);
            expected.insert((1, 4), vec![1, 2, 4, 5, 6, 7, 8]);
            expected.insert((2, 3), vec![1, 2, 4, 6, 7, 8, 9]);
            expected.insert((2, 4), vec![1, 2, 3, 4, 5, 7, 8, 9]);
            expected.insert((3, 0), vec![1, 2, 3, 4, 6, 7, 8]);
            expected.insert((3, 1), vec![1, 2, 3, 5, 7, 8, 9]);
            expected.insert((3, 2), vec![1, 2, 3, 4, 6, 8, 9]);
            expected.insert((3, 3), vec![1, 2, 3, 4, 5, 7, 9]);
            expected.insert((3, 4), vec![1, 2, 3, 4, 5, 6, 7, 8]);
            expected.insert((4, 0), vec![1, 2, 3, 4, 5, 6, 7, 9]);
            expected.insert((4, 1), vec![1, 2, 3, 4, 5, 6, 8]);
            expected.insert((4, 2), vec![1, 2, 3, 4, 5, 6, 7, 9]);
            expected.insert((4, 3), vec![1, 2, 3, 4, 5, 6, 7, 8]);
            // Remaining cells have all possibilities
            for row in 0..9 {
                for col in 0..9 {
                    expected
                        .entry((row, col))
                        .or_insert(ALL_POSSIBILITIES.to_vec());
                }
            }
            let result = knight.get_possibilities(&grid);
            // println!("Expected:");
            // print_hashmap(&expected);
            // println!("Result:");
            // print_hashmap(&result);
            assert_eq!(result, Ok(expected));
        }

        #[test]
        fn fully_filled() {
            let values = [
                [8, 1, 2, 9, 7, 3, 5, 4, 6],
                [9, 6, 3, 5, 4, 8, 2, 7, 1],
                [7, 4, 5, 1, 6, 2, 8, 9, 3],
                [4, 8, 1, 7, 3, 5, 9, 6, 2],
                [6, 3, 7, 2, 8, 9, 4, 1, 5],
                [2, 5, 9, 6, 1, 4, 7, 3, 8],
                [1, 9, 8, 3, 5, 7, 6, 2, 4],
                [3, 7, 4, 8, 2, 6, 1, 5, 9],
                [5, 2, 6, 4, 9, 1, 3, 8, 7],
            ];
            let mut grid = SudokuGrid::empty();
            let knight = Knight::new();
            let mut expected = HashMap::new();
            for (r, row) in values.iter().enumerate() {
                for (c, &value) in row.iter().enumerate() {
                    grid.set_cell(r, c, value);
                    expected.insert((r, c), vec![value]);
                }
            }
            assert_eq!(knight.get_possibilities(&grid), Ok(expected));
        }
    }
}
