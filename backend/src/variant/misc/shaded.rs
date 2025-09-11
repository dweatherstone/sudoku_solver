use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    SudokuGrid, SudokuVariant,
    file_parser::parse_positions,
    variant::{Variant, VariantContradiction, error::PossibilityResult},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Shaded {
    cell: (usize, usize),
    shape: Shape,
}

impl Shaded {
    fn new(cell: (usize, usize), shape: Shape) -> Shaded {
        Shaded { cell, shape }
    }

    pub fn parse(data: &str) -> Option<SudokuVariant> {
        let parts: Vec<&str> = data.split(":").collect();
        if parts.len() != 2 {
            return None;
        }
        let cells = parse_positions(parts[0]).ok()?;
        if cells.len() != 1 {
            return None;
        }
        let shape = Shape::from_str(parts[1])?;
        Some(SudokuVariant::Shaded(Shaded::new(cells[0], shape)))
    }
}

impl Variant for Shaded {
    fn is_valid(&self, _grid: &crate::SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        if self.cell != (row, col) {
            return true;
        }
        self.shape.digit_range().contains(&value)
    }

    fn validate_solution(&self, grid: &crate::SudokuGrid) -> bool {
        self.shape
            .digit_range()
            .contains(&grid.get_cell(self.cell.0, self.cell.1))
    }

    fn constrained_cells(&self) -> Vec<(usize, usize)> {
        vec![self.cell]
    }

    fn get_possibilities(&self, grid: &SudokuGrid) -> PossibilityResult {
        let mut possibilities = HashMap::new();
        let value = grid.get_cell(self.cell.0, self.cell.1);
        if value == 0 {
            possibilities.insert(self.cell, self.shape.digit_range());
        } else if self.shape.digit_range().contains(&value) {
            possibilities.insert(self.cell, vec![value]);
        } else {
            return Err(VariantContradiction::NoPossibilities {
                cell: self.cell,
                variant: "ShadedCell",
                reason: format!("Cell must contain one of: {:?}", self.shape.digit_range()),
            });
        }
        Ok(possibilities)
    }
}

impl std::fmt::Display for Shaded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Shaded {}: ({}, {})",
            self.shape, self.cell.0, self.cell.1
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Shape {
    Square,
    Circle,
}

impl Shape {
    fn from_str(s: &str) -> Option<Shape> {
        match s.to_lowercase().trim() {
            "square" => Some(Shape::Square),
            "circle" => Some(Shape::Circle),
            _ => None,
        }
    }

    fn digit_range(&self) -> Vec<u8> {
        match self {
            Shape::Circle => vec![1, 3, 5, 7, 9],
            Shape::Square => vec![2, 4, 6, 8],
        }
    }
}

impl std::fmt::Display for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shape::Circle => write!(f, "Circle"),
            Shape::Square => write!(f, "Square"),
        }
    }
}

#[cfg(test)]
mod tests {
    mod is_valid {
        use crate::{
            SudokuGrid,
            variant::{
                Variant,
                misc::{Shaded, shaded::Shape},
            },
        };

        #[test]
        fn test_valid() {
            let grid = SudokuGrid::empty();
            let shaded = Shaded::new((0, 0), Shape::Square);
            for value in [2, 4, 6, 8] {
                assert!(shaded.is_valid(&grid, 0, 0, value));
            }
            let shaded = Shaded::new((0, 0), Shape::Circle);
            for value in [1, 3, 5, 7, 9] {
                assert!(shaded.is_valid(&grid, 0, 0, value));
            }
        }

        #[test]
        fn test_invalid() {
            let grid = SudokuGrid::empty();
            let shaded = Shaded::new((0, 0), Shape::Square);
            for value in [0, 1, 3, 5, 7, 9] {
                assert!(!shaded.is_valid(&grid, 0, 0, value));
            }
            let shaded = Shaded::new((0, 0), Shape::Circle);
            for value in [0, 2, 4, 6, 8] {
                assert!(!shaded.is_valid(&grid, 0, 0, value));
            }
        }

        #[test]
        fn test_unconstrained_cell() {
            let grid = SudokuGrid::empty();
            let shaded = Shaded::new((0, 0), Shape::Square);
            assert!(shaded.is_valid(&grid, 1, 1, 1));
        }
    }

    mod validate_solution {
        use crate::{
            Shaded, SudokuGrid,
            variant::{Variant, misc::shaded::Shape},
        };

        #[test]
        fn test_valid_solution() {
            let mut grid = SudokuGrid::empty();
            let shaded = Shaded::new((0, 0), Shape::Circle);
            for value in [1, 3, 5, 7, 9] {
                grid.set_cell(0, 0, value);
                assert!(shaded.validate_solution(&grid));
            }
        }

        #[test]
        fn test_invalid_solution() {
            let mut grid = SudokuGrid::empty();
            let shaded = Shaded::new((0, 0), Shape::Square);
            for value in [1, 3, 5, 7, 9] {
                grid.set_cell(0, 0, value);
                assert!(!shaded.validate_solution(&grid));
            }
        }
    }

    mod get_possibilities {
        use crate::{
            Shaded, SudokuGrid,
            variant::{Variant, VariantContradiction, misc::shaded::Shape},
        };

        #[test]
        fn test_square() {
            let mut grid = SudokuGrid::empty();
            let square = Shaded::new((0, 0), Shape::Square);
            grid.set_cell(0, 0, 6);
            let result = square.get_possibilities(&grid);
            assert!(result.is_ok());
            let result = result.unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result.get(&(0, 0)), Some(&vec![6]));
        }

        #[test]
        fn test_circle() {
            let mut grid = SudokuGrid::empty();
            let circle = Shaded::new((0, 0), Shape::Circle);
            grid.set_cell(0, 0, 5);
            let result = circle.get_possibilities(&grid);
            assert!(result.is_ok());
            let result = result.unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result.get(&(0, 0)), Some(&vec![5]));
        }

        #[test]
        fn test_unconstrained_cell() {
            let mut grid = SudokuGrid::empty();
            let shaded = Shaded::new((0, 0), Shape::Circle);
            grid.set_cell(1, 1, 5);
            let result = shaded.get_possibilities(&grid);
            assert!(result.is_ok());
            let result = result.unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result.get(&(0, 0)), Some(&vec![1, 3, 5, 7, 9]));
        }

        #[test]
        fn test_cell_set_success() {
            let mut grid = SudokuGrid::empty();
            let shaded = Shaded::new((0, 0), Shape::Circle);
            grid.set_cell(0, 0, 3);
            let result = shaded.get_possibilities(&grid);
            assert!(result.is_ok());
            let result = result.unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result.get(&(0, 0)).unwrap(), &vec![3]);
        }

        #[test]
        fn test_cell_set_fail() {
            let mut grid = SudokuGrid::empty();
            let shaded = Shaded::new((0, 0), Shape::Circle);
            grid.set_cell(0, 0, 2);
            let result = shaded.get_possibilities(&grid);
            assert!(matches!(
                result,
                Err(VariantContradiction::NoPossibilities { cell: (0, 0), .. })
            ));
        }
    }
}
