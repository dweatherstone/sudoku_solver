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
pub struct QuadrupleCircle {
    cells: Vec<(usize, usize)>,
    required: Vec<u8>,
    is_anti: bool,
}

impl QuadrupleCircle {
    pub fn new(cells: Vec<(usize, usize)>, required: Vec<u8>, is_anti: bool) -> Self {
        QuadrupleCircle {
            cells,
            required,
            is_anti,
        }
    }

    pub fn parse(data: &str, is_anti: bool) -> Option<SudokuVariant> {
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
            cells, required, is_anti,
        )))
    }
}

impl Variant for QuadrupleCircle {
    fn is_valid(&self, grid: &crate::SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        // If (row, col) is not in the quadruple circle, just pass
        if !self.cells.contains(&(row, col)) {
            return true;
        }
        if self.is_anti {
            // If value is any of the required numbers, then return early
            return !self.required.contains(&value);
        } else {
            // If there are 4 required numbers, and value is not one of them, then early return
            if self.required.len() == 4 && !self.required.contains(&value) {
                return false;
            }
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

    fn get_possibilities(&self, grid: &SudokuGrid) -> PossibilityResult {
        // Map of all existing cell values surrounding the quadratic circle
        let cell_values: HashMap<(usize, usize), u8> = self
            .cells
            .iter()
            .map(|&(row, col)| ((row, col), grid.get_cell(row, col)))
            .collect();

        let known_digits_in_circle = cell_values
            .iter()
            .filter_map(|(_, v)| if v == &0 { None } else { Some(*v) })
            .collect::<Vec<_>>();
        let empty_cell_count = cell_values.iter().filter(|&(_, &v)| v == 0).count();

        let missing_required_digits = self
            .required
            .iter()
            .filter(|&req| !known_digits_in_circle.contains(req))
            .copied()
            .collect::<Vec<u8>>();

        // Helper closure for creating the return possibilities
        let insert_possibilities = |values: Vec<u8>| {
            Ok(cell_values
                .iter()
                .map(|(&(r, c), &v)| {
                    if v != 0 {
                        ((r, c), vec![v])
                    } else {
                        ((r, c), values.clone())
                    }
                })
                .collect::<HashMap<_, _>>())
        };

        // Normal Quadruple Circles
        if !self.is_anti {
            // If there is no space to fit the required values, return early
            if empty_cell_count < missing_required_digits.len() {
                return Err(VariantContradiction::Inconsistent {
                    variant: "QuadrupleCircle",
                    reason: String::from("Not enough empty cells to fill required digits"),
                });
            }
            // If there is only just space to fit the required values, return these
            else if empty_cell_count == missing_required_digits.len() {
                insert_possibilities(missing_required_digits.clone())
            }
            // If there is more than enough space, then the cells can be any value
            else {
                insert_possibilities(ALL_POSSIBILITIES.to_vec())
            }
        } else {
            // Anti-Quadruple
            // If the quadruple is invalid (i.e. contains a restricted value) return empty
            if known_digits_in_circle
                .iter()
                .any(|v| self.required.contains(v))
            {
                return Err(VariantContradiction::Inconsistent {
                    variant: "Anti-QuadrupleCircle",
                    reason: String::from(
                        "Circle contains value that anti-quadruple does not allow",
                    ),
                });
            }
            // Return a set of all values not including the required values
            insert_possibilities(
                ALL_POSSIBILITIES
                    .iter()
                    .filter(|&v| !self.required.contains(v))
                    .copied()
                    .collect::<Vec<_>>(),
            )
        }
    }
}

impl std::fmt::Display for QuadrupleCircle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = if self.is_anti {
            String::from("Anti-Quadruple Circle [")
        } else {
            String::from("Quadruple Circle [")
        };
        output.push_str(
            self.cells
                .iter()
                .map(|&(r, c)| format!("({r}, {c})"))
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
        output.push_str(required_str);
        output.push(']');
        write!(f, "{output}")
    }
}

#[cfg(test)]
mod get_possibilities {

    use super::QuadrupleCircle;
    use crate::{
        SudokuGrid,
        variant::{Variant, error::PossibilityResult},
    };

    #[test]
    fn test_one_required_digit_one_cell_filled_valid() {
        let mut grid = SudokuGrid::empty();
        let circle = get_test_circle(vec![5], false);
        grid.set_cell(1, 1, 5);
        let result = circle.get_possibilities(&grid);
        test_all_possibilities_other_than_first_cell(&result, 5);
    }

    #[test]
    fn test_one_required_digit_unsatisfied() {
        let mut grid = SudokuGrid::empty();
        let circle = get_test_circle(vec![5], false);
        grid.set_cell(1, 1, 3);
        let result = circle.get_possibilities(&grid);
        test_all_possibilities_other_than_first_cell(&result, 3);
    }

    #[test]
    fn test_multiple_required_digits_one_satisfied() {
        let mut grid = SudokuGrid::empty();
        let circle = get_test_circle(vec![4, 7], false);
        grid.set_cell(1, 1, 4);
        let result = circle.get_possibilities(&grid);
        test_all_possibilities_other_than_first_cell(&result, 4);
    }

    #[test]
    fn test_impossible_to_satisfy() {
        let mut grid = SudokuGrid::empty();
        let circle = get_test_circle(vec![7, 8], false);
        grid.set_cell(1, 1, 1);
        grid.set_cell(1, 2, 2);
        grid.set_cell(2, 1, 3);
        let result = circle.get_possibilities(&grid);
        assert!(result.is_err());
    }

    #[test]
    fn test_anti_valid_state_excludes_banned_digits() {
        let mut grid = SudokuGrid::empty();
        grid.set_cell(1, 1, 1);
        let circle = get_test_circle(vec![4, 5, 6, 7], true);
        let result = circle.get_possibilities(&grid);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result.get(&(1, 1)).unwrap(), &vec![1]);
        for cell in [(1, 2), (2, 1), (2, 2)] {
            assert_eq!(result.get(&cell).unwrap(), &vec![1, 2, 3, 8, 9]);
        }
    }

    #[test]
    fn test_quadruple_anti_rule_excludes_value() {
        let mut grid = SudokuGrid::empty();
        grid.set_cell(1, 1, 5);

        let circle = get_test_circle(vec![5], true);

        let result = circle.get_possibilities(&grid);
        // Already invalid — forbidden value placed
        assert!(result.is_err());
    }

    #[test]
    fn test_corner_case() {
        let mut grid = SudokuGrid::empty();
        grid.set_cell(0, 0, 2);
        let circle = QuadrupleCircle::new(vec![(0, 0), (0, 1), (1, 0)], vec![7], false);
        let result = circle.get_possibilities(&grid);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result.get(&(0, 0)).unwrap(), &vec![2]);
        for cell in [(0, 1), (1, 0)] {
            assert!(result.get(&cell).unwrap().contains(&7));
        }
    }

    fn get_test_circle(required: Vec<u8>, is_anti: bool) -> QuadrupleCircle {
        QuadrupleCircle::new(vec![(1, 1), (1, 2), (2, 1), (2, 2)], required, is_anti)
    }

    fn test_all_possibilities_other_than_first_cell(
        result: &PossibilityResult,
        first_cell_value: u8,
    ) {
        assert!(result.is_ok());
        let result = result.clone().unwrap();
        let expected: Vec<u8> = (1..=9).collect();
        assert_eq!(result.len(), 4);
        assert_eq!(result.get(&(1, 1)).unwrap(), &vec![first_cell_value]);
        assert_eq!(result.get(&(1, 2)).unwrap(), &expected);
        assert_eq!(result.get(&(2, 1)).unwrap(), &expected);
        assert_eq!(result.get(&(2, 2)).unwrap(), &expected);
    }
}
