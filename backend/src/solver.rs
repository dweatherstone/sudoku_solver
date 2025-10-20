use std::collections::{HashMap, HashSet};

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
                    if self.update_possibilities(row, col).is_ok() {
                        self.apply_naked_subsets();
                        self.apply_pointing_pairs();
                        self.apply_hidden_pairs();
                        if self.solve_recursive(debug, steps, max_steps) {
                            return true;
                        }
                    }
                    // Backtrack
                    if debug {
                        println!("Backtracking cell ({row}, {col}), value {num}");
                    }
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

    /// Applies naked pairs/triples/quads logic to all rows, columns, and boxes.
    /// This eliminates candidates from other cells in the same unit.
    pub fn apply_naked_subsets(&mut self) {
        for unit in self.get_all_units() {
            self.apply_naked_subsets_to_unit(&unit);
        }
    }

    /// Returns a Vec of Vec<(usize, usize)> for all rows, columns, and boxes.
    fn get_all_units(&self) -> Vec<Vec<(usize, usize)>> {
        let mut units = Vec::new();
        // Rows
        for r in 0..9 {
            units.push((0..9).map(|c| (r, c)).collect());
        }
        // Columns
        for c in 0..9 {
            units.push((0..9).map(|r| (r, c)).collect());
        }
        // Boxes
        units.extend(self.get_all_boxes());
        units
    }

    /// Applies naked subset logic to a single unit (row, col, or box).
    fn apply_naked_subsets_to_unit(&mut self, unit: &[(usize, usize)]) {
        // Only consider cells with 2-4 candidates
        let cell_poss: Vec<((usize, usize), Vec<u8>)> = unit
            .iter()
            .filter_map(|&(r, c)| {
                self.possiblilities
                    .get(&(r, c))
                    .map(|poss| ((r, c), poss.clone()))
            })
            .filter(|(_, poss): &((usize, usize), Vec<u8>)| (2..=4).contains(&poss.len()))
            .collect();

        // For N in 2..=4 (pairs, triples, quads)
        for n in 2..=4 {
            // Find all combinations of n cells
            for combo in cell_poss.iter().combinations(n) {
                let cells: Vec<_> = combo.iter().map(|((r, c), _)| (*r, *c)).collect();
                let mut all_candidates = combo
                    .iter()
                    .flat_map(|(_, poss)| poss.iter().copied())
                    .collect::<Vec<_>>();
                all_candidates.sort_unstable();
                all_candidates.dedup();
                if all_candidates.len() == n {
                    // Naked subset found: eliminate these candidates from other cells in the unit
                    for &(r, c) in unit {
                        if !cells.contains(&(r, c)) {
                            if let Some(poss) = self.possiblilities.get_mut(&(r, c)) {
                                //let before = poss.len();
                                poss.retain(|v| !all_candidates.contains(v));
                                //let after = poss.len();
                                //if before != after {
                                // Optionally, print debug info here...
                                //}
                            }
                        }
                    }
                }
            }
        }
    }

    /// Applies the logic of pointing pairs. I.e. if a particular value's possibilities in
    /// a particular box are all in the same row/column, then that value cannot be present
    /// in any cells in that row/column outside the box.
    pub fn apply_pointing_pairs(&mut self) {
        for value in 1..=9 {
            for a_box in self.get_all_boxes() {
                let candidates = a_box
                    .iter()
                    .filter_map(|&(r, c)| {
                        self.possiblilities
                            .get(&(r, c))
                            .map(|poss| ((r, c), poss.clone()))
                    })
                    .filter(|(_, poss)| poss.contains(&value))
                    .collect::<Vec<((usize, usize), Vec<u8>)>>();
                if !candidates.is_empty() {
                    let all_in_one_row = candidates
                        .iter()
                        .map(|&((r, _), _)| r)
                        .all(|r| r == candidates[0].0.0);
                    let all_in_one_col = candidates
                        .iter()
                        .map(|&((_, c), _)| c)
                        .all(|c| c == candidates[0].0.1);

                    if all_in_one_row {
                        // All candidates are in the same row: eliminate `value` from other cells in the row
                        let row = candidates[0].0.0;
                        let poss_cols: Vec<usize> =
                            candidates.iter().map(|&((_, c), _)| c).collect();
                        self.remove_possibility_from_row(value, row, &poss_cols);
                    }
                    if all_in_one_col {
                        // All candidates are in the same column: eliminate `value` from other cells in that column outside this box
                        let col = candidates[0].0.1;
                        let poss_rows: Vec<usize> =
                            candidates.iter().map(|&((r, _), _)| r).collect();
                        self.remove_possibility_from_col(value, col, &poss_rows);
                    }
                }
            }
        }
    }

    /// https://www.sudokuwiki.org/Hidden_Candidates#HP
    pub fn apply_hidden_pairs(&mut self) {
        for unit in self.get_all_units() {
            self.apply_hidden_subsets_to_unit(&unit, 2); // pairs
            self.apply_hidden_subsets_to_unit(&unit, 3); // triples
        }
    }

    fn apply_hidden_subsets_to_unit(&mut self, unit: &[(usize, usize)], subset_size: usize) {
        for combo in (1u8..=9).combinations(subset_size) {
            // Collect all cells in the unit that contain any digit in the combo
            let mut cells_with_combo = HashSet::new();
            for &(row, col) in unit {
                if let Some(poss) = self.possiblilities.get(&(row, col))
                    && combo.iter().any(|d| poss.contains(d))
                {
                    cells_with_combo.insert((row, col));
                }
            }
            // If exactly subset_size cells, and all contain digits in combo
            if cells_with_combo.len() == subset_size
                && cells_with_combo.iter().all(|&(row, col)| {
                    let poss = self.possiblilities.get(&(row, col)).unwrap();
                    combo.iter().all(|d| poss.contains(d))
                })
            {
                for &(row, col) in &cells_with_combo {
                    self.possiblilities
                        .entry((row, col))
                        .and_modify(|poss| *poss = combo.clone());
                }
            }
        }
    }

    fn get_all_boxes(&self) -> Vec<Vec<(usize, usize)>> {
        let mut boxes = Vec::new();
        for br in 0..3 {
            for bc in 0..3 {
                let mut box_cells = Vec::new();
                for dr in 0..3 {
                    for dc in 0..3 {
                        box_cells.push((br * 3 + dr, bc * 3 + dc));
                    }
                }
                boxes.push(box_cells);
            }
        }

        boxes
    }

    fn remove_possibility_from_row(&mut self, value: u8, row: usize, allowed_cols: &[usize]) {
        for c in 0..9 {
            if !allowed_cols.contains(&c) {
                self.possiblilities
                    .entry((row, c))
                    .and_modify(|v| v.retain(|val| val != &value));
            }
        }
    }

    fn remove_possibility_from_col(&mut self, value: u8, col: usize, allowed_rows: &[usize]) {
        for r in 0..9 {
            if !allowed_rows.contains(&r) {
                self.possiblilities
                    .entry((r, col))
                    .and_modify(|v| v.retain(|val| val != &value));
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    mod apply_naked_subsets {
        use super::*;

        #[test]
        fn naked_pair_in_row() {
            let mut grid = SudokuGrid::empty();
            // Set up possibilities: row 0 col 0 and 1 have [1,2], others have [1,2,3]
            let mut solver = Solver::new(&mut grid).unwrap();
            solver
                .possiblilities
                .entry((0, 0))
                .and_modify(|v| *v = vec![1, 2]);
            solver
                .possiblilities
                .entry((0, 1))
                .and_modify(|v| *v = vec![1, 2]);
            // Only test row 0 for simplicity
            solver.apply_naked_subsets();
            // The naked pair [1,2] in (0,0) and (0,1) should remove 1,2 from other cells in row 0
            for c in 0..9 {
                let poss = solver.possiblilities.get(&(0, c)).unwrap();
                if c == 0 || c == 1 {
                    assert_eq!(
                        poss,
                        &vec![1, 2],
                        "Cell (0, {c}) should only have [1, 2] left, but has: {:?}",
                        poss
                    );
                } else {
                    assert_eq!(
                        poss,
                        &vec![3, 4, 5, 6, 7, 8, 9],
                        "Cell (0, {c}) should only have [3,4,5,6,7,8,9] left, but has: {:?}",
                        poss
                    );
                }
            }
            // Now also check the cells in Box 1 of the grid - top left box.
            for dr in 0..3 {
                for dc in 0..3 {
                    let poss = solver.possiblilities.get(&(0 + dr, 0 + dc)).unwrap();
                    if dr == 0 && (dc == 0 || dc == 1) {
                        assert_eq!(
                            poss,
                            &vec![1, 2],
                            "Cell ({}, {}) should only have [1, 2] left, but has: {:?}",
                            0 + dr,
                            0 + dc,
                            poss
                        );
                    } else {
                        assert_eq!(
                            poss,
                            &vec![3, 4, 5, 6, 7, 8, 9],
                            "Cell ({}, {}) should only have [3,4,5,6,7,8,9] left, but has: {:?}",
                            0 + dr,
                            0 + dc,
                            poss
                        );
                    }
                }
            }
            // Check column 0
            for r in 0..9 {
                let poss = solver.possiblilities.get(&(r, 0)).unwrap();
                let expected = match r {
                    0 => vec![1, 2],
                    1..=2 => vec![3, 4, 5, 6, 7, 8, 9],
                    3..9 => vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    _ => panic!("Row out of bounds"),
                };

                assert_eq!(
                    poss, &expected,
                    "Cell ({r}, 0) should only have {:?} left, but has: {:?}",
                    expected, poss
                );
            }
        }

        #[test]
        fn naked_triple_in_column() {
            let mut grid = SudokuGrid::empty();
            let mut solver = Solver::new(&mut grid).unwrap();
            // Setup a naked triple in column 0, rows 0, 3, 6 - so only the column should be affected
            solver
                .possiblilities
                .entry((0, 0))
                .and_modify(|v| *v = vec![1, 2, 3]);
            solver
                .possiblilities
                .entry((3, 0))
                .and_modify(|v| *v = vec![1, 2, 3]);
            solver
                .possiblilities
                .entry((6, 0))
                .and_modify(|v| *v = vec![1, 2, 3]);

            solver.apply_naked_subsets();

            // Check that the column is as expected
            for r in 0..9 {
                let poss = solver.possiblilities.get(&(r, 0)).unwrap();
                if r == 0 || r == 3 || r == 6 {
                    assert_eq!(
                        poss,
                        &vec![1, 2, 3],
                        "Cell ({r}, 0) should only have [1,2,3] left, but has {:?}",
                        poss
                    );
                } else {
                    assert_eq!(
                        poss,
                        &vec![4, 5, 6, 7, 8, 9],
                        "Cell ({r}, 0) should only have [4,5,6,7,8,9] left, but has {:?}",
                        poss
                    );
                }
            }
            // Check row 0 is unaffected
            for c in 0..9 {
                let poss = solver.possiblilities.get(&(0, c)).unwrap();
                if c == 0 {
                    assert_eq!(
                        poss,
                        &vec![1, 2, 3],
                        "Cell (0, {c}) should only have [1,2,3] left, but has {:?}",
                        poss
                    );
                } else {
                    assert_eq!(
                        poss,
                        &vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                        "Cell (0, {c}) should have all possibilities left, but has {:?}",
                        poss
                    );
                }
            }
            // Check box 0 is only affected down column 0
            for dr in 0..3 {
                for dc in 0..3 {
                    let poss = solver.possiblilities.get(&(0 + dr, 0 + dc)).unwrap();
                    if dr == 0 && dc == 0 {
                        assert_eq!(
                            poss,
                            &vec![1, 2, 3],
                            "Cell ({}, {}) should only have [1, 2, 3] left, but has: {:?}",
                            0 + dr,
                            0 + dc,
                            poss
                        );
                    } else if dc == 0 {
                        assert_eq!(
                            poss,
                            &vec![4, 5, 6, 7, 8, 9],
                            "Cell ({}, {}) should only have [4,5,6,7,8,9] left, but has {:?}",
                            0 + dr,
                            0 + dc,
                            poss
                        );
                    } else {
                        assert_eq!(
                            poss,
                            &vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                            "Cell ({}, {}) should have all possibilities left, but has: {:?}",
                            0 + dr,
                            0 + dc,
                            poss
                        );
                    }
                }
            }
        }

        #[test]
        fn naked_triple_with_subset() {
            let mut grid = SudokuGrid::empty();
            let mut solver = Solver::new(&mut grid).unwrap();
            // Cells (0, 0) and (0, 1) have [1,2,3] as possibilities, and cell (0, 2) has [2,3].
            // This should have the affect of the three cells being a triple
            solver
                .possiblilities
                .entry((0, 0))
                .and_modify(|v| *v = vec![1, 2, 3]);
            solver
                .possiblilities
                .entry((0, 1))
                .and_modify(|v| *v = vec![1, 2, 3]);
            solver
                .possiblilities
                .entry((0, 2))
                .and_modify(|v| *v = vec![1, 2]);
            solver.apply_naked_subsets();

            // Test that row 0 has a triple
            for c in 0..9 {
                let poss = solver.possiblilities.get(&(0, c)).unwrap();
                if c == 0 || c == 1 {
                    assert_eq!(
                        poss,
                        &vec![1, 2, 3],
                        "Cell (0, {c} should only have [1,2,3] left, but has {:?}",
                        poss
                    );
                } else if c == 2 {
                    assert_eq!(
                        poss,
                        &vec![1, 2],
                        "Cell (0, {c} should only have [1,2] left, but has {:?}",
                        poss
                    );
                } else {
                    assert_eq!(
                        poss,
                        &vec![4, 5, 6, 7, 8, 9],
                        "Cell (0, {c} should only have [4,5,6,7,8,9] left, but has {:?}",
                        poss
                    );
                }
            }

            // Test that box 0 has a triple
            for dr in 0..3 {
                for dc in 0..3 {
                    let poss = solver.possiblilities.get(&(0 + dr, 0 + dc)).unwrap();
                    if dr == 0 && (dc == 0 || dc == 1) {
                        assert_eq!(
                            poss,
                            &vec![1, 2, 3],
                            "Cell ({}, {}) should only have [1, 2, 3] left, but has: {:?}",
                            0 + dr,
                            0 + dc,
                            poss
                        );
                    } else if dr == 0 && dc == 2 {
                        assert_eq!(
                            poss,
                            &vec![1, 2],
                            "Cell ({}, {}) should only have [1, 2] left, but has: {:?}",
                            0 + dr,
                            0 + dc,
                            poss
                        );
                    } else {
                        assert_eq!(
                            poss,
                            &vec![4, 5, 6, 7, 8, 9],
                            "Cell ({}, {}) should only have [4,5,6,7,8,9] left, but has: {:?}",
                            0 + dr,
                            0 + dc,
                            poss
                        );
                    }
                }
            }

            // Check column 0
            for r in 0..9 {
                let poss = solver.possiblilities.get(&(r, 0)).unwrap();
                let expected = match r {
                    0 => vec![1, 2, 3],
                    1..=2 => vec![4, 5, 6, 7, 8, 9],
                    3..9 => vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    _ => panic!("Row out of bounds"),
                };

                assert_eq!(
                    poss, &expected,
                    "Cell ({r}, 0) should only have {:?} left, but has: {:?}",
                    expected, poss
                );
            }
        }

        #[test]
        fn naked_triple_less_options() {
            let mut grid = SudokuGrid::empty();
            let mut solver = Solver::new(&mut grid).unwrap();
            // Row 0: Cells 0, 1, 2 have a [1,2,3] triple
            // Other cells have combinations of the rest of the available values
            let expected: [Vec<u8>; 9] = [
                vec![1, 2, 3],
                vec![1, 2, 3],
                vec![1, 2, 3],
                vec![4, 5],
                vec![5, 6],
                vec![6, 7],
                vec![7, 8],
                vec![8, 9],
                vec![4, 9],
            ];
            for (c, e) in expected.iter().enumerate() {
                solver
                    .possiblilities
                    .entry((0, c))
                    .and_modify(|v| *v = e.clone());
            }
            solver.apply_naked_subsets();
            // Check row 0. Shouldn't have any changes
            for (c, e) in expected.iter().enumerate() {
                let poss = solver.possiblilities.get(&(0, c)).unwrap();
                assert_eq!(
                    poss, e,
                    "Cell (0, {c}) should only have {:?}, but has {:?}",
                    e, poss
                );
            }
            // Check column 0
            for r in 0..9 {
                let poss = solver.possiblilities.get(&(r, 0)).unwrap();
                if r == 0 {
                    assert_eq!(
                        poss,
                        &vec![1, 2, 3],
                        "Cell ({r}, 0) should only have [1,2,3], but has {:?}",
                        poss
                    );
                } else if r < 3 {
                    assert_eq!(
                        poss,
                        &vec![4, 5, 6, 7, 8, 9],
                        "Cell ({r}, 0) should only have [4,5,6,7,8,9], but has {:?}",
                        poss
                    );
                } else {
                    assert_eq!(
                        poss,
                        &vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                        "Cell ({r}, 0) should have all possibilities, but has {:?}",
                        poss
                    );
                }
            }
            // Check column 3
            for r in 0..9 {
                let poss = solver.possiblilities.get(&(r, 3)).unwrap();
                if r == 0 {
                    assert_eq!(
                        poss, &expected[3],
                        "Cell ({r}, 3) should only have {:?}, but has {:?}",
                        &expected[3], poss
                    );
                } else {
                    assert_eq!(
                        poss,
                        &vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                        "Cell ({r}, 3) should have all possibilities, but has {:?}",
                        poss
                    );
                }
            }
        }

        #[test]
        fn naked_triple_and_double() {
            let mut grid = SudokuGrid::empty();
            let mut solver = Solver::new(&mut grid).unwrap();
            // Row 0: Cells 0, 1, 2 have a [1,2,3] triple
            // Cells 5, 6 have a [4,5] double
            // Cells 3, 6, 7, 8 contain rest of [6,7,8,9]
            let row_0_possibilities: [Vec<u8>; 9] = [
                vec![1, 2, 3],
                vec![1, 2, 3],
                vec![1, 2, 3],
                vec![1, 2, 3, 4, 5, 6, 7],
                vec![4, 5],
                vec![4, 5],
                vec![1, 2, 3, 4, 5, 7, 8],
                vec![1, 2, 3, 4, 5, 8, 9],
                vec![1, 2, 3, 4, 5, 6, 9],
            ];
            let expected: [Vec<u8>; 9] = [
                vec![1, 2, 3],
                vec![1, 2, 3],
                vec![1, 2, 3],
                vec![6, 7],
                vec![4, 5],
                vec![4, 5],
                vec![7, 8],
                vec![8, 9],
                vec![6, 9],
            ];
            for (c, p) in row_0_possibilities.iter().enumerate() {
                solver
                    .possiblilities
                    .entry((0, c))
                    .and_modify(|v| *v = p.clone());
            }
            solver.apply_naked_subsets();
            // Check row 0.
            for (c, e) in expected.iter().enumerate() {
                let poss = solver.possiblilities.get(&(0, c)).unwrap();
                assert_eq!(
                    poss, e,
                    "Cell (0, {c}) should only have {:?}, but has {:?}",
                    e, poss
                );
            }
            // Check column 0
            for r in 0..9 {
                let poss = solver.possiblilities.get(&(r, 0)).unwrap();
                if r == 0 {
                    assert_eq!(
                        poss,
                        &vec![1, 2, 3],
                        "Cell ({r}, 0) should only have [1,2,3], but has {:?}",
                        poss
                    );
                } else if r < 3 {
                    assert_eq!(
                        poss,
                        &vec![4, 5, 6, 7, 8, 9],
                        "Cell ({r}, 0) should only have [4,5,6,7,8,9], but has {:?}",
                        poss
                    );
                } else {
                    assert_eq!(
                        poss,
                        &vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                        "Cell ({r}, 0) should have all possibilities, but has {:?}",
                        poss
                    );
                }
            }
            // Check column 3
            for r in 0..9 {
                let poss = solver.possiblilities.get(&(r, 3)).unwrap();
                if r == 0 {
                    assert_eq!(
                        poss,
                        &vec![6, 7],
                        "Cell ({r}, 3) should only have [6,7], but has {:?}",
                        poss
                    );
                } else if r < 3 {
                    assert_eq!(
                        poss,
                        &vec![1, 2, 3, 6, 7, 8, 9],
                        "Cell ({r}, 3) should only have [1,2,3,6,7,8,9], but has {:?}",
                        poss
                    );
                } else {
                    assert_eq!(
                        poss,
                        &vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                        "Cell ({r}, 3) should have all possibilities, but has {:?}",
                        poss
                    );
                }
            }
        }
    }

    mod solve {
        use super::*;

        #[test]
        fn solves_with_naked_pairs() {
            let mut grid = SudokuGrid::empty();
            let givens = [
                (0, 0, 5),
                (0, 1, 1),
                (0, 2, 7),
                (0, 3, 6),
                (0, 7, 3),
                (0, 8, 4),
                (1, 0, 2),
                (1, 1, 8),
                (1, 2, 9),
                (1, 5, 4),
                (2, 0, 3),
                (2, 1, 4),
                (2, 2, 6),
                (2, 3, 2),
                (2, 5, 5),
                (2, 7, 9),
                (3, 0, 6),
                (3, 2, 2),
                (3, 7, 1),
                (4, 1, 3),
                (4, 2, 8),
                (4, 5, 6),
                (4, 7, 4),
                (4, 8, 7),
                (6, 1, 9),
                (6, 7, 7),
                (6, 8, 8),
                (7, 0, 7),
                (7, 2, 3),
                (7, 3, 4),
                (7, 6, 5),
                (7, 7, 6),
            ];
            for &(r, c, v) in &givens {
                grid.set_cell(r, c, v);
            }

            let mut solver = Solver::new(&mut grid).unwrap();
            let solved = solver.solve(false);
            assert!(solved, "Solver should solve the puzzle using naked pairs");
            // Assert the final grid state matches the expected solution.
            let solution = [
                [5, 1, 7, 6, 9, 8, 2, 3, 4],
                [2, 8, 9, 1, 3, 4, 7, 5, 6],
                [3, 4, 6, 2, 7, 5, 8, 9, 1],
                [6, 7, 2, 8, 4, 9, 3, 1, 5],
                [1, 3, 8, 5, 2, 6, 9, 4, 7],
                [9, 5, 4, 7, 1, 3, 6, 8, 2],
                [4, 9, 5, 3, 6, 2, 1, 7, 8],
                [7, 2, 3, 4, 8, 1, 5, 6, 9],
                [8, 6, 1, 9, 5, 7, 4, 2, 3],
            ];
            for (r, row) in solution.iter().enumerate() {
                for (c, val) in row.iter().enumerate() {
                    assert_eq!(
                        &grid.get_cell(r, c),
                        val,
                        "Cell ({r}, {c}) expected: {val}, got: {}",
                        grid.get_cell(r, c)
                    );
                }
            }
        }

        #[test]
        fn solves_with_pointing_pairs() {
            // https://sudoku.com/sudoku-rules/pointing-pairs/
            let mut grid = SudokuGrid::empty();
            let givens = [
                (0, 2, 9),
                (0, 4, 7),
                (1, 1, 8),
                (1, 3, 4),
                (2, 2, 3),
                (2, 7, 2),
                (2, 8, 8),
                (3, 0, 1),
                (3, 6, 6),
                (3, 7, 7),
                (4, 1, 2),
                (4, 4, 1),
                (4, 5, 3),
                (4, 7, 4),
                (5, 1, 4),
                (5, 5, 7),
                (5, 6, 8),
                (6, 0, 6),
                (6, 4, 3),
                (7, 1, 1),
                (8, 6, 2),
                (8, 7, 8),
                (8, 8, 4),
            ];
            for &(r, c, v) in &givens {
                grid.set_cell(r, c, v);
            }
            let mut solver = Solver::new(&mut grid).unwrap();
            let solved = solver.solve(false);
            assert!(
                solved,
                "Solver should solve the puzzle using pointing pairs"
            );
            // Assert the final grid state matches the expected solution.
            let solution = [
                [2, 6, 9, 3, 7, 8, 4, 1, 5],
                [5, 8, 1, 4, 2, 9, 7, 6, 3],
                [4, 7, 3, 5, 6, 1, 9, 2, 8],
                [1, 3, 5, 9, 8, 4, 6, 7, 2],
                [7, 2, 8, 6, 1, 3, 5, 4, 9],
                [9, 4, 6, 2, 5, 7, 8, 3, 1],
                [6, 9, 4, 8, 3, 2, 1, 5, 7],
                [8, 1, 2, 7, 4, 5, 3, 9, 6],
                [3, 5, 7, 1, 9, 6, 2, 8, 4],
            ];
            for (r, row) in solution.iter().enumerate() {
                for (c, val) in row.iter().enumerate() {
                    assert_eq!(
                        &grid.get_cell(r, c),
                        val,
                        "Cell ({r}, {c}) expected: {val}, got: {}",
                        grid.get_cell(r, c)
                    );
                }
            }
        }
    }

    mod pointing_pairs {
        use super::*;

        #[test]
        fn poining_pair_row_removal() {
            let mut grid = SudokuGrid::empty();
            let mut solver = Solver::new(&mut grid).unwrap();

            // Set up: value 5 is a candidate in (0, 0) and (0, 1) (both in row 0, box 0)
            // Do this by removing 5 as a possibilitiy from all other cells in box 0
            solver
                .possiblilities
                .entry((0, 2))
                .and_modify(|v| v.retain(|&val| val != 5));
            solver
                .possiblilities
                .entry((1, 0))
                .and_modify(|v| v.retain(|&val| val != 5));
            solver
                .possiblilities
                .entry((1, 1))
                .and_modify(|v| v.retain(|&val| val != 5));
            solver
                .possiblilities
                .entry((1, 2))
                .and_modify(|v| v.retain(|&val| val != 5));
            solver
                .possiblilities
                .entry((2, 0))
                .and_modify(|v| v.retain(|&val| val != 5));
            solver
                .possiblilities
                .entry((2, 1))
                .and_modify(|v| v.retain(|&val| val != 5));
            solver
                .possiblilities
                .entry((2, 2))
                .and_modify(|v| v.retain(|&val| val != 5));
            solver.apply_pointing_pairs();
            // Check all of row 0 to ensure 5 is only a possibility in cells (0, 0) and (0, 1)
            for c in 0..9 {
                let poss = solver.possiblilities.get(&(0, c)).unwrap();
                if c == 0 || c == 1 {
                    assert!(
                        poss.contains(&5),
                        "Cell (0, {c}) possibilities should contain a 5"
                    );
                } else {
                    assert!(
                        !poss.contains(&5),
                        "Cell (0, {c}) possibilities should NOT contain a 5. Possibilities are: {:?}",
                        poss
                    );
                }
            }
            // Check all of row 1, to ensure 5 is not possible in first 3 cells (box 0), but is possible in all other cells
            for c in 0..9 {
                let poss = solver.possiblilities.get(&(1, c)).unwrap();
                if c < 3 {
                    assert!(
                        !poss.contains(&5),
                        "Cell (1, {c}) possibilities should NOT contain a 5. Possibilities are: {:?}",
                        poss
                    );
                } else {
                    assert!(
                        poss.contains(&5),
                        "Cell (1, {c}) possibilities should contain a 5"
                    );
                }
            }
            // Check row 3, to ensure 5 is a possibility in all cells
            for c in 0..9 {
                let poss = solver.possiblilities.get(&(3, c)).unwrap();
                assert!(
                    poss.contains(&5),
                    "Cell (1, {c}) possibilities should contain a 5"
                );
            }
        }

        #[test]
        fn pointing_pair_column_removal() {
            let mut grid = SudokuGrid::empty();
            let mut solver = Solver::new(&mut grid).unwrap();

            // Set up: value 5 is a candidate in (0, 6) and (1, 6) (both in col 6, box 2)
            // Do this by removing 5 as a possibilitiy from all other cells in box 2
            solver
                .possiblilities
                .entry((0, 7))
                .and_modify(|v| v.retain(|&val| val != 5));
            solver
                .possiblilities
                .entry((0, 8))
                .and_modify(|v| v.retain(|&val| val != 5));
            solver
                .possiblilities
                .entry((1, 7))
                .and_modify(|v| v.retain(|&val| val != 5));
            solver
                .possiblilities
                .entry((1, 8))
                .and_modify(|v| v.retain(|&val| val != 5));
            solver
                .possiblilities
                .entry((2, 6))
                .and_modify(|v| v.retain(|&val| val != 5));
            solver
                .possiblilities
                .entry((2, 7))
                .and_modify(|v| v.retain(|&val| val != 5));
            solver
                .possiblilities
                .entry((2, 8))
                .and_modify(|v| v.retain(|&val| val != 5));
            solver.apply_pointing_pairs();
            // Check all of col 6 to ensure 5 is only a possibility in cells (0, 6) and (1, 6)
            for r in 0..9 {
                let poss = solver.possiblilities.get(&(r, 6)).unwrap();
                if r == 0 || r == 1 {
                    assert!(
                        poss.contains(&5),
                        "Cell ({r}, 6) possibilities should contain a 5"
                    );
                } else {
                    assert!(
                        !poss.contains(&5),
                        "Cell ({r}, 6) possibilities should NOT contain a 5. Possibilities are: {:?}",
                        poss
                    );
                }
            }
            // Check all of col 7, to ensure 5 is not possible in first 3 cells (box 2), but is possible in all other cells
            for r in 0..9 {
                let poss = solver.possiblilities.get(&(r, 7)).unwrap();
                if r < 3 {
                    assert!(
                        !poss.contains(&5),
                        "Cell ({r}, 7) possibilities should NOT contain a 5. Possibilities are: {:?}",
                        poss
                    );
                } else {
                    assert!(
                        poss.contains(&5),
                        "Cell ({r}, 7) possibilities should contain a 5"
                    );
                }
            }
            // Check col 0, to ensure 5 is a possibility in all cells
            for r in 0..9 {
                let poss = solver.possiblilities.get(&(r, 0)).unwrap();
                assert!(
                    poss.contains(&5),
                    "Cell ({r}, 7) possibilities should contain a 5"
                );
            }
        }
    }

    mod hidden_pairs {
        use super::*;

        #[test]
        fn hidden_pair_row() {
            // Example from: https://www.sudokuwiki.org/Hidden_Candidates#HP
            // The 6's and 7's in boxes 1 and 2, and column 7, mean that 6 and 7
            // must be in row 1, column 8 and 9.
            let mut grid = SudokuGrid::empty();
            // Givens: (row, col, value)
            let givens = [
                (1, 0, 9),
                (1, 2, 4),
                (1, 3, 6),
                (1, 5, 7),
                (2, 1, 7),
                (2, 2, 6),
                (2, 3, 8),
                (2, 5, 4),
                (2, 6, 1),
                (3, 0, 3),
                (3, 2, 9),
                (3, 3, 7),
                (3, 5, 1),
                (3, 7, 8),
                (4, 2, 8),
                (4, 6, 3),
                (5, 1, 5),
                (5, 3, 3),
                (5, 5, 8),
                (5, 6, 7),
                (5, 8, 2),
                (6, 2, 7),
                (6, 3, 5),
                (6, 5, 2),
                (6, 6, 6),
                (6, 7, 1),
                (7, 3, 4),
                (7, 5, 3),
                (7, 6, 2),
                (7, 8, 8),
            ];
            for &(r, c, v) in &givens {
                grid.set_cell(r, c, v);
            }
            // grid.display(false);
            let mut solver = Solver::new(&mut grid).unwrap();
            // Check that the possibilities in (0, 7) and (0, 8) have many options before `apply_hidden_pairs` called
            assert_eq!(
                solver.possiblilities.get(&(0, 7)),
                Some(&vec![2, 3, 4, 5, 6, 7, 9]),
                "Incorrect possibilities for (0, 7) before function call."
            );
            assert_eq!(
                solver.possiblilities.get(&(0, 8)),
                Some(&vec![3, 4, 5, 6, 7, 9]),
                "Incorrect possibilities for (0, 8) before function call."
            );
            solver.apply_hidden_pairs();
            // Check that the possibilities of (0, 7) and (0, 8) are now minimised
            assert_eq!(
                solver.possiblilities.get(&(0, 7)),
                Some(&vec![6, 7]),
                "After fn call: cell (0, 7) possibilities: {:?}, should be [6, 7]",
                solver.possiblilities.get(&(0, 7))
            );
            assert_eq!(
                solver.possiblilities.get(&(0, 8)),
                Some(&vec![6, 7]),
                "After fn call: cell (0, 8) possibilities: {:?}, should be [6, 7]",
                solver.possiblilities.get(&(0, 8))
            );
            // Check that no other cells in the row or box can contain a 6 or a 7
            // Check row 0, but ignore columns 7 and 8 (already tested above)
            for col in 0..7 {
                if let Some(poss) = solver.possiblilities.get(&(0, col)) {
                    assert!(!poss.contains(&6), "Cell (0, {col}) should not contain a 6");
                    assert!(!poss.contains(&7), "Cell (0, {col}) should not contain a 7");
                }
            }
            for dr in 0..3 {
                for dc in 0..3 {
                    let row = 0 + dr;
                    let col = 6 + dc;
                    if let Some(poss) = solver.possiblilities.get(&(row, col)) {
                        if row == 0 && (col == 7 || col == 8) {
                            continue;
                        }
                        assert!(
                            !poss.contains(&6),
                            "Cell ({row}, {col}) should not contain a 6"
                        );
                        assert!(
                            !poss.contains(&7),
                            "Cell ({row}, {col}) should not contain a 7"
                        );
                    }
                }
            }
        }

        #[test]
        fn hidden_pair_columns() {
            let mut grid = SudokuGrid::empty();
            let givens = [
                (2, 3, 9),
                (2, 7, 8),
                (3, 1, 8),
                (5, 2, 9),
                (7, 2, 8),
                (8, 1, 9),
            ];
            for &(r, c, v) in &givens {
                grid.set_cell(r, c, v);
            }
            let mut solver = Solver::new(&mut grid).unwrap();
            for row in 0..2 {
                if let Some(poss) = solver.possiblilities.get(&(row, 0)) {
                    assert_eq!(
                        poss,
                        &vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                        "Cell ({row}, 0) should have all possibilities. Got: {:?}",
                        poss
                    );
                }
            }
            solver.apply_hidden_pairs();
            for row in 0..2 {
                if let Some(poss) = solver.possiblilities.get(&(row, 0)) {
                    assert_eq!(
                        poss,
                        &vec![8, 9],
                        "Cell ({row}, 0) should have possibilities [8, 9]. Got: {:?}",
                        poss
                    );
                }
            }
        }

        #[test]
        fn hidden_pair_box() {
            // Expecting to see that [1, 2] can only go in cells (0, 6) and (2, 8) in box 3
            let mut grid = SudokuGrid::empty();
            let givens = [
                (1, 1, 1),
                (1, 5, 2),
                (4, 7, 2),
                (7, 7, 1),
                (0, 8, 5),
                (2, 6, 3),
            ];
            for &(r, c, v) in &givens {
                grid.set_cell(r, c, v);
            }
            let mut solver = Solver::new(&mut grid).unwrap();
            assert_eq!(
                solver.possiblilities.get(&(0, 6)),
                Some(&vec![1, 2, 4, 6, 7, 8, 9]),
                "Cell (0, 6) before should be Some(&[1,2,4,6,7,8,9]). Got: {:?}",
                solver.possiblilities.get(&(0, 6))
            );
            assert_eq!(
                solver.possiblilities.get(&(2, 8)),
                Some(&vec![1, 2, 4, 6, 7, 8, 9]),
                "Cell (2, 8) before should be Some(&[1,2,4,6,7,8,9]). Got: {:?}",
                solver.possiblilities.get(&(2, 8))
            );
            solver.apply_hidden_pairs();
            assert_eq!(
                solver.possiblilities.get(&(0, 6)),
                Some(&vec![1, 2]),
                "Cell (0, 6) before should be Some(&[1,2]). Got: {:?}",
                solver.possiblilities.get(&(0, 6))
            );
            assert_eq!(
                solver.possiblilities.get(&(2, 8)),
                Some(&vec![1, 2]),
                "Cell (2, 8) before should be Some(&[1,2]). Got: {:?}",
                solver.possiblilities.get(&(2, 8))
            );
        }

        #[test]
        fn no_hidden_pair() {
            let mut grid = SudokuGrid::empty();
            let givens = [
                (1, 1, 1),
                (1, 2, 2),
                (2, 3, 1),
                (2, 4, 2),
                (4, 6, 1),
                (5, 7, 2),
            ];
            for &(r, c, v) in &givens {
                grid.set_cell(r, c, v);
            }
            let mut solver = Solver::new(&mut grid).unwrap();
            let expected = [
                &vec![2, 3, 4, 5, 6, 7, 8, 9],
                &vec![1, 3, 4, 5, 6, 7, 8, 9],
                &vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
            ];
            for (i, &(r, c)) in [(0, 6), (0, 7), (0, 8)].iter().enumerate() {
                let poss = solver.possiblilities.get(&(r, c));
                assert_eq!(
                    poss,
                    Some(expected[i]),
                    "Cell ({r}, {c}) should have Some({:?}), but has {:?}",
                    expected[i],
                    poss
                );
            }
            solver.apply_hidden_pairs();
            // Shouldn't change
            for (i, &(r, c)) in [(0, 6), (0, 7), (0, 8)].iter().enumerate() {
                let poss = solver.possiblilities.get(&(r, c));
                assert_eq!(
                    poss,
                    Some(expected[i]),
                    "Cell ({r}, {c}) should have Some({:?}), but has {:?}",
                    expected[i],
                    poss
                );
            }
        }

        #[test]
        fn overlapping_hidden_and_naked_pairs() {
            let mut grid = SudokuGrid::empty();
            let givens = [
                (1, 7, 1),
                (1, 8, 2),
                (3, 5, 1),
                (4, 5, 2),
                (6, 4, 1),
                (7, 4, 2),
            ];
            for &(r, c, v) in &givens {
                grid.set_cell(r, c, v);
            }
            let mut solver = Solver::new(&mut grid).unwrap();
            solver
                .possiblilities
                .entry((0, 1))
                .and_modify(|poss| *poss = vec![3, 4]);
            solver
                .possiblilities
                .entry((0, 2))
                .and_modify(|poss| *poss = vec![3, 4]);
            // Check the possibilities in row 0 from initial setup
            for col in 0..9 {
                let poss = solver.possiblilities.get(&(0, col));
                if col == 1 || col == 2 {
                    assert_eq!(
                        poss,
                        Some(&vec![3, 4]),
                        "Before: Cell (0, {col}) should be Some([3,4]), but got: {:?}",
                        poss
                    );
                } else if col == 0 || col == 3 {
                    assert_eq!(
                        poss,
                        Some(&vec![1, 2, 3, 4, 5, 6, 7, 8, 9]),
                        "Before: Cell (0, {col}) should be any value, but got: {:?}",
                        poss
                    );
                } else {
                    assert_eq!(
                        poss,
                        Some(&vec![3, 4, 5, 6, 7, 8, 9]),
                        "Before: Cell (0, {col} should be Some([3,4,5,6,7,8,9]), but got: {:?}",
                        poss
                    );
                }
            }
            // Apply naked subsets and hidden pairs
            solver.apply_naked_subsets();
            solver.apply_hidden_pairs();
            // Check possibilities for row 0
            for col in 0..9 {
                let poss = solver.possiblilities.get(&(0, col));
                if col == 1 || col == 2 {
                    // Naked pair
                    assert_eq!(
                        poss,
                        Some(&vec![3, 4]),
                        "After: Cell (0, {col}) should be Some([3,4]), but got: {:?}",
                        poss
                    );
                } else if col == 0 || col == 3 {
                    // Hidden pair
                    assert_eq!(
                        poss,
                        Some(&vec![1, 2]),
                        "After: Cell (0, {col}) should be Some([1,2]), but got: {:?}",
                        poss
                    );
                } else {
                    // The rest
                    assert_eq!(
                        poss,
                        Some(&vec![5, 6, 7, 8, 9]),
                        "After: Cell (0, {col} should be Some([5,6,7,8,9]), but got: {:?}",
                        poss
                    );
                }
            }
        }

        #[test]
        fn hidden_pair_solve() {
            let mut grid = SudokuGrid::empty();
            // Givens: (row, col, value)
            let givens = [
                (1, 0, 9),
                (1, 2, 4),
                (1, 3, 6),
                (1, 5, 7),
                (2, 1, 7),
                (2, 2, 6),
                (2, 3, 8),
                (2, 5, 4),
                (2, 6, 1),
                (3, 0, 3),
                (3, 2, 9),
                (3, 3, 7),
                (3, 5, 1),
                (3, 7, 8),
                (4, 2, 8),
                (4, 6, 3),
                (5, 1, 5),
                (5, 3, 3),
                (5, 5, 8),
                (5, 6, 7),
                (5, 8, 2),
                (6, 2, 7),
                (6, 3, 5),
                (6, 5, 2),
                (6, 6, 6),
                (6, 7, 1),
                (7, 3, 4),
                (7, 5, 3),
                (7, 6, 2),
                (7, 8, 8),
            ];
            for &(r, c, v) in &givens {
                grid.set_cell(r, c, v);
            }
            // grid.display(false);
            let mut solver = Solver::new(&mut grid).unwrap();
            let solution = [
                [5, 8, 3, 2, 1, 9, 4, 6, 7],
                [9, 1, 4, 6, 3, 7, 8, 2, 5],
                [2, 7, 6, 8, 5, 4, 1, 3, 9],
                [3, 4, 9, 7, 2, 1, 5, 8, 6],
                [7, 2, 8, 9, 6, 5, 3, 4, 1],
                [6, 5, 1, 3, 4, 8, 7, 9, 2],
                [4, 9, 7, 5, 8, 2, 6, 1, 3],
                [1, 6, 5, 4, 9, 3, 2, 7, 8],
                [8, 3, 2, 1, 7, 6, 9, 5, 4],
            ];
            let solved = solver.solve(false);
            // grid.display(false);
            assert!(solved, "Solver should solve the puzzle");
            for (r, row) in solution.iter().enumerate() {
                for (c, val) in row.iter().enumerate() {
                    assert_eq!(
                        &grid.get_cell(r, c),
                        val,
                        "Cell ({r}, {c}) expected: {val}, got, {}",
                        grid.get_cell(r, c)
                    );
                }
            }
        }
    }
}
