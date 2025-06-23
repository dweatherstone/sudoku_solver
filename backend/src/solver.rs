use std::collections::HashSet;

use crate::SudokuGrid;

pub struct Solver<'a> {
    sudoku_grid: &'a mut SudokuGrid,
    priority_cells: HashSet<(usize, usize)>,
}

impl<'a> Solver<'a> {
    pub fn new(sudoku_grid: &'a mut SudokuGrid) -> Self {
        let priority_cells = sudoku_grid
            .variants()
            .flat_map(|variant| variant.constrained_cells())
            .collect();
        Solver {
            sudoku_grid,
            priority_cells,
        }
    }

    pub fn solve(&mut self, debug: bool) -> bool {
        let mut steps = 0;
        let max_steps = 1_000_000;
        let result = self.solve_recursive(debug, &mut steps, max_steps);
        if debug {
            println!("Returning '{}' from solve after {} steps", result, steps);
        }
        result
    }

    fn solve_recursive(&mut self, debug: bool, steps: &mut usize, max_steps: usize) -> bool {
        *steps += 1;

        if *steps > max_steps {
            if debug {
                println!("Solver aborted after {} steps (limit reached)", *steps);
            }
            return false;
        }

        // Find the next empty cell (if any)
        match self.find_most_constrained_cell(debug) {
            NextCell::Cell(row, col, candidates) => {
                // Try filling the cell with each possible digit
                for &num in &candidates {
                    if debug {
                        println!("Trying value {} at cell ({}, {})", num, row, col);
                    }

                    // Check if the current digit is valid for the cell
                    if self.sudoku_grid.is_valid_move(row, col, num) {
                        // If valid, set the cell value and recursively solve
                        self.sudoku_grid.set_cell(row, col, num);
                        if self.solve_recursive(debug, steps, max_steps) {
                            return true;
                        }
                        // If the recursive call returns false, backtrack and try the next digit
                        if debug {
                            println!("Backtracking from value {} at cell ({}, {})", num, row, col);
                        }
                        self.sudoku_grid.set_cell(row, col, 0);
                    }
                }
                // If no valid digit leads to a solution, backtrack
                false
            }
            NextCell::NoEmptyCells => self.validate_solution(),
            NextCell::DeadEnd => false,
        }
    }

    fn validate_solution(&self) -> bool {
        // Check that the sudoku grid is valid
        if !self.sudoku_grid.is_board_valid() {
            return false;
        }
        // Check that all variants are satisfied
        for variant in self.sudoku_grid.variants() {
            if !variant.validate_solution(self.sudoku_grid) {
                return false;
            }
        }
        true
    }

    fn find_most_constrained_cell(&self, debug: bool) -> NextCell {
        let mut best_cell = None;
        let mut min_options = 10; // More than max possible digits (1-9)

        for row in 0..9 {
            for col in 0..9 {
                if self.sudoku_grid.get_cell(row, col) == 0 {
                    let is_priority = self.priority_cells.contains(&(row, col));

                    let candidates: Vec<u8> = (1..=9)
                        .filter(|&num| self.sudoku_grid.is_valid_move(row, col, num))
                        .collect();

                    if candidates.is_empty() {
                        if debug {
                            println!(
                                "WARNING: Cell ({}, {}) has NO candidates! Will backtrack.",
                                row, col
                            );
                        }
                        return NextCell::DeadEnd;
                    }

                    let score = candidates.len() - if is_priority { 1 } else { 0 };
                    if score < min_options {
                        best_cell = Some((row, col, candidates.clone()));
                        min_options = score;
                    }
                }
            }
        }

        if let Some((row, col, candidates)) = best_cell {
            NextCell::Cell(row, col, candidates)
        } else {
            NextCell::NoEmptyCells
        }
    }
}

enum NextCell {
    Cell(usize, usize, Vec<u8>),
    NoEmptyCells,
    DeadEnd,
}
