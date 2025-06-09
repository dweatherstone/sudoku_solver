use std::{
    fs::File,
    io::{BufRead, BufReader, Error, ErrorKind},
};

use serde::{Deserialize, Serialize};

use crate::{KillerCage, KropkiDot, QuadrupleCircle, variant::Variant};

#[derive(Serialize, Deserialize, Clone)]
pub enum SudokuVariant {
    Killer(KillerCage),
    Kropki(KropkiDot),
    QuadrupleCircles(QuadrupleCircle),
}

impl SudokuVariant {
    pub fn is_valid(&self, grid: &SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        match self {
            SudokuVariant::Killer(cage) => cage.is_valid(grid, row, col, value),
            SudokuVariant::Kropki(dot) => dot.is_valid(grid, row, col, value),
            SudokuVariant::QuadrupleCircles(circle) => circle.is_valid(grid, row, col, value),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SudokuGrid {
    cells: [[u8; 9]; 9],
    variants: Vec<SudokuVariant>,
}

impl SudokuGrid {
    pub fn new() -> Self {
        SudokuGrid {
            cells: [[0; 9]; 9],
            variants: Vec::new(),
        }
    }

    pub fn get_cell(&self, row: usize, col: usize) -> u8 {
        self.cells[row][col]
    }

    pub fn get_cells(&self) -> [[u8; 9]; 9] {
        self.cells
    }

    pub fn variants(&self) -> impl Iterator<Item = &SudokuVariant> {
        self.variants.iter()
    }

    pub fn set_cell(&mut self, row: usize, col: usize, value: u8) {
        self.cells[row][col] = value;
    }

    pub fn add_variant(&mut self, variant: SudokuVariant) {
        self.variants.push(variant);
    }

    pub fn display(&self, show_variants: bool) {
        for row in &self.cells {
            for &cell in row {
                let cell_str = if cell == 0 {
                    ".".to_string()
                } else {
                    cell.to_string()
                };
                print!("{} ", cell_str);
            }
            println!();
        }
        if show_variants {
            println!("Variants:");
            for variant in &self.variants {
                match variant {
                    SudokuVariant::Killer(cage) => println!("Killer Cage: {:?}", cage),
                    SudokuVariant::Kropki(dot) => println!("Kropki Dot: {:?}", dot),
                    SudokuVariant::QuadrupleCircles(circle) => {
                        println!("Quadruple Circles: {:?}", circle)
                    }
                }
            }
        }
    }

    pub fn find_empty_cell(&self) -> Option<(usize, usize)> {
        for row in 0..9 {
            for col in 0..9 {
                if self.get_cell(row, col) == 0 {
                    return Some((row, col));
                }
            }
        }
        None
    }

    pub fn read_from_file(filename: &str) -> Result<Self, Error> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        let mut sudoku_grid = SudokuGrid::default();

        for (row, line) in reader.lines().enumerate() {
            let line = line?;
            let chars: Vec<char> = line.chars().collect();
            for (col, ch) in chars.iter().enumerate() {
                if let Some(num) = ch.to_digit(10) {
                    sudoku_grid.set_cell(row, col, num as u8);
                } else if *ch != '.' {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Invalid character in input",
                    ));
                }
            }
        }
        Ok(sudoku_grid)
    }

    fn used_in_col(&self, col: usize, num: u8) -> bool {
        for row in 0..9 {
            if self.get_cell(row, col) == num {
                return true;
            }
        }
        false
    }

    fn used_in_row(&self, row: usize, num: u8) -> bool {
        for col in 0..9 {
            if self.get_cell(row, col) == num {
                return true;
            }
        }
        false
    }

    fn used_in_subgrid(&self, start_row: usize, start_col: usize, num: u8) -> bool {
        for row in 0..3 {
            for col in 0..3 {
                if self.get_cell(row + start_row, col + start_col) == num {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_valid_move(&self, row: usize, col: usize, num: u8) -> bool {
        if !self.is_classic_valid(row, col, num) {
            return false;
        }
        self.variants
            .iter()
            .all(|v| v.is_valid(self, row, col, num))
        //self.is_classic_valid(row, col, num)
    }

    fn is_classic_valid(&self, row: usize, col: usize, num: u8) -> bool {
        !self.used_in_row(row, num)
            && !self.used_in_col(col, num)
            && !self.used_in_subgrid(row - row % 3, col - col % 3, num)
    }
}

impl Default for SudokuGrid {
    fn default() -> Self {
        SudokuGrid::new()
    }
}
