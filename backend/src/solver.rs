use std::collections::HashMap;

use itertools::Itertools;

use crate::variant::VariantContradiction;
use crate::{SudokuGrid, variant::PossibilityResult};

pub struct Solver<'a> {
    sudoku_grid: &'a mut SudokuGrid,
    possiblilities: HashMap<(usize, usize), Vec<u8>>,
}

impl<'a> Solver<'a> {
    pub fn new(sudoku_grid: &'a mut SudokuGrid) -> Result<Self, VariantContradiction> {
        let possiblilities = Self::get_all_possibilities(sudoku_grid)?;
        Ok(Solver {
            sudoku_grid,
            possiblilities,
        })
    }

    pub fn solve(&mut self, debug: bool) -> bool {
        let mut steps = 0;
        let max_steps = 1_000_000;
        let result = self.solve_recursive(debug, &mut steps, max_steps);
        if debug {
            println!("Returning '{result}' from solve after {steps} steps");
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
                let old_poss = self.possiblilities.clone();
                // Try filling the cell with each possible digit
                for &num in &candidates {
                    if debug {
                        println!("Trying value {num} at cell ({row}, {col})");
                    }
                    self.sudoku_grid.set_cell(row, col, num);
                    if self.update_possibilities(row, col).is_ok()
                        && self.solve_recursive(debug, steps, max_steps)
                    {
                        return true;
                    }
                    // Backtrack
                    self.sudoku_grid.set_cell(row, col, 0);
                    self.possiblilities = old_poss.clone();
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

        for (&(row, col), poss) in &self.possiblilities {
            if poss.is_empty() {
                if debug {
                    println!("WARNING: Cell ({row}, {col}) has NO candidates! Will backtrack.");
                }
                return NextCell::DeadEnd;
            }
            if poss.len() < min_options {
                best_cell = Some((row, col, poss.clone()));
                min_options = poss.len();
            }
        }

        if let Some((row, col, candidates)) = best_cell {
            NextCell::Cell(row, col, candidates)
        } else {
            NextCell::NoEmptyCells
        }
    }

    fn get_all_possibilities(sudoku_grid: &SudokuGrid) -> PossibilityResult {
        let mut possibilities = HashMap::new();
        for row in 0..9 {
            for col in 0..9 {
                if sudoku_grid.get_cell(row, col) == 0 {
                    // Start with all digits
                    let mut poss = sudoku_grid.get_standard_possibilities_for_cell(row, col);
                    // Apply all variant constraints
                    for variant in sudoku_grid.variants() {
                        let var_poss = variant.get_possibilities(sudoku_grid)?;
                        if let Some(var_vals) = var_poss.get(&(row, col)) {
                            poss.retain(|v| var_vals.contains(v));
                        }
                    }
                    if poss.is_empty() {
                        return Err(VariantContradiction::NoPossibilities {
                            cell: (row, col),
                            variant: "Solver",
                            reason: "No candidates after intersecting rules".to_string(),
                        });
                    }
                    possibilities.insert((row, col), poss);
                }
            }
        }
        Ok(possibilities)
    }

    fn update_possibilities(
        &mut self,
        _row: usize,
        _col: usize,
    ) -> Result<(), VariantContradiction> {
        // For all empty cells in the same row, col, box, or affected variant, recompute possibilities
        for r in 0..9 {
            for c in 0..9 {
                if self.sudoku_grid.get_cell(r, c) == 0 {
                    // Start with all digits
                    let mut poss = self.sudoku_grid.get_standard_possibilities_for_cell(r, c);
                    // Apply all variant constraints
                    for variant in self.sudoku_grid.variants() {
                        let var_poss = variant.get_possibilities(&self.sudoku_grid)?;
                        if let Some(var_vals) = var_poss.get(&(r, c)) {
                            poss.retain(|v| var_vals.contains(v));
                        }
                    }
                    if poss.is_empty() {
                        return Err(VariantContradiction::NoPossibilities {
                            cell: (r, c),
                            variant: "Solver",
                            reason: "No candidates after intersecting rules".to_string(),
                        });
                    }
                    self.possiblilities.insert((r, c), poss);
                } else {
                    self.possiblilities.remove(&(r, c));
                }
            }
        }
        Ok(())
    }

    pub fn possibilities_to_string(&self, row: usize, col: usize) -> String {
        match self.possiblilities.get(&(row, col)) {
            Some(vals) => {
                let vals_str = vals.iter().join(", ");
                format!("({row}, {col}) -> [{vals_str}]")
            }
            None => format!("No possibilities for ({row}, {col})"),
        }
    }
}

enum NextCell {
    Cell(usize, usize, Vec<u8>),
    NoEmptyCells,
    DeadEnd,
}
