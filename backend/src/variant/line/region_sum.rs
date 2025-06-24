/*
Region sum lines: box borders divide each blue line into segments with the same sum.
*/

use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{SudokuVariant, file_parser::parse_positions, variant::Variant};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RegionSum {
    // cells: box number: Vec<(row, col)>
    box_cells: HashMap<usize, Vec<(usize, usize)>>,
}

impl RegionSum {
    pub fn new(cells: Vec<(usize, usize)>) -> Self {
        let mut box_cells = HashMap::new();
        for &(row, col) in &cells {
            let box_number = Self::get_box_number(row, col);
            box_cells
                .entry(box_number)
                .or_insert_with(Vec::new)
                .push((row, col));
        }
        RegionSum { box_cells }
    }

    pub fn parse(data: &str) -> Option<SudokuVariant> {
        let cells = parse_positions(data).ok()?;
        Some(SudokuVariant::RegionSum(RegionSum::new(cells)))
    }

    fn get_box_number(row: usize, col: usize) -> usize {
        (row / 3) * 3 + (col / 3)
    }

    fn min_possible_sum(current_sum: u8, unknowns: usize) -> u8 {
        current_sum + (1..=9).take(unknowns).sum::<u8>()
    }

    fn max_possible_sum(current_sum: u8, unknowns: usize) -> u8 {
        current_sum + (1..=9).rev().take(unknowns).sum::<u8>()
    }
}

impl Variant for RegionSum {
    fn is_valid(&self, grid: &crate::SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        // If the proposed cell is not on this region sum line, then continue
        if !self.constrained_cells().contains(&(row, col)) {
            return true;
        }
        // Find which box this cell belongs to
        let current_box = Self::get_box_number(row, col);
        let current_segment = match self.box_cells.get(&current_box) {
            Some(cells) => cells,
            // None = cell not on the region sum line
            None => return true,
        };

        // Replace the current cell with the proposed value in the segment
        let current_values: Vec<u8> = current_segment
            .iter()
            .map(|&(r, c)| {
                if r == row && c == col {
                    value
                } else {
                    grid.get_cell(r, c)
                }
            })
            .collect();

        // Find a target sum from any fully filled segment (excluding current)
        let target_sum_opt = self
            .box_cells
            .iter()
            .filter(|(b, _)| **b != current_box)
            .map(|(_, cells)| {
                cells
                    .iter()
                    .map(|&(r, c)| grid.get_cell(r, c))
                    .collect::<Vec<u8>>()
            })
            .find_map(|values| {
                if values.iter().all(|&v| v != 0) {
                    Some(values.iter().sum::<u8>())
                } else {
                    None
                }
            });

        if let Some(target_sum) = target_sum_opt {
            let filled = current_values.iter().all(|&v| v != 0);
            let sum = current_values.iter().sum::<u8>();

            if filled {
                return sum == target_sum;
            }
        }

        let current_known_sum: u8 = current_values.iter().sum();
        let current_unknowns = current_values.iter().filter(|&&v| v == 0).count();
        let current_min = Self::min_possible_sum(current_known_sum, current_unknowns);
        let current_max = Self::max_possible_sum(current_known_sum, current_unknowns);

        // Now check if this overlaps with all other segment ranges
        for (&box_num, segment) in self.box_cells.iter() {
            if box_num == current_box {
                continue;
            }

            let vals: Vec<u8> = segment.iter().map(|&(r, c)| grid.get_cell(r, c)).collect();
            let known_sum = vals.iter().sum();
            let unknowns = vals.iter().filter(|&&v| v == 0).count();

            // Skip totally unknown segments
            if known_sum == 0 {
                continue;
            }

            let min = Self::min_possible_sum(known_sum, unknowns);
            let max = Self::max_possible_sum(known_sum, unknowns);

            // If ranges do not overlap, this is invalid
            if current_max < min || current_min > max {
                return false;
            }
        }

        // No complete segments -> future improvement here
        true
    }

    fn validate_solution(&self, grid: &crate::SudokuGrid) -> bool {
        // If any of the cells do not have a value set, then invalid
        if self
            .box_cells
            .values()
            .flat_map(|cells| cells.iter())
            .any(|&(r, c)| grid.get_cell(r, c) == 0)
        {
            return false;
        }

        // Get the sum of the first box-segment as the target sum
        let mut iter = self.box_cells.values();
        let first_sum = if let Some(sum) = iter
            .next()
            .map(|cells| cells.iter().map(|&(r, c)| grid.get_cell(r, c)).sum::<u8>())
        {
            sum
        } else {
            // If there are no box segments in the constraint, just continue
            return true;
        };

        // Compare all other segments to this sum
        for cells in iter {
            let sum = cells.iter().map(|&(r, c)| grid.get_cell(r, c)).sum::<u8>();
            if sum != first_sum {
                return false;
            }
        }

        true
    }

    fn constrained_cells(&self) -> Vec<(usize, usize)> {
        self.box_cells
            .values()
            .flat_map(|cells| cells.clone())
            .collect()
    }
}

impl Display for RegionSum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut box_cell_entries: Vec<(&usize, &Vec<(usize, usize)>)> =
            self.box_cells.iter().collect();
        box_cell_entries.sort_by_key(|(box_num, _)| *box_num);
        let mut output = String::from("Region Sum Line:");
        for (box_num, cells) in box_cell_entries.iter() {
            output.push_str(&format!(" region {}: [", box_num));
            output.push_str(
                cells
                    .iter()
                    .map(|&(r, c)| format!("({}, {})", r, c))
                    .collect::<Vec<_>>()
                    .join(", ")
                    .as_str(),
            );
            output.push_str("],");
        }
        write!(f, "{}", output.trim_end_matches(","))
    }
}

#[cfg(test)]
mod tests {
    use crate::{SudokuGrid, variant::Variant};

    use super::RegionSum;

    #[test]
    fn test_validate_correct_solution() {
        let mut grid = SudokuGrid::empty();
        let line = vec![
            (0, 0),
            (0, 1), // Box 0
            (0, 3),
            (0, 4), // Box 1
            (0, 6),
            (0, 7), // Box 2
        ];
        let region_sum = RegionSum::new(line);

        // Fill all segments to sum to 10
        grid.set_cell(0, 0, 4);
        grid.set_cell(0, 1, 6);
        grid.set_cell(0, 3, 1);
        grid.set_cell(0, 4, 9);
        grid.set_cell(0, 6, 2);
        grid.set_cell(0, 7, 8);

        assert!(region_sum.validate_solution(&grid));
    }

    #[test]
    fn test_validate_incorrect_solution() {
        let mut grid = SudokuGrid::empty();
        let line = vec![
            (0, 0),
            (0, 1), // Box 0
            (0, 3),
            (0, 4), // Box 1
            (0, 6),
            (0, 7), // Box 2
        ];
        let region_sum = RegionSum::new(line);

        grid.set_cell(0, 0, 3); // This one is wrong!
        grid.set_cell(0, 1, 6);
        grid.set_cell(0, 3, 1);
        grid.set_cell(0, 4, 9);
        grid.set_cell(0, 6, 2);
        grid.set_cell(0, 7, 8);

        assert!(!region_sum.validate_solution(&grid));
    }

    #[test]
    fn test_validate_solution_with_zeros() {
        let mut grid = SudokuGrid::empty();
        let line = vec![
            (0, 0),
            (0, 1), // Box 0
            (0, 3),
            (0, 4), // Box 1
            (0, 6),
            (0, 7), // Box 2
        ];
        let region_sum = RegionSum::new(line);

        grid.set_cell(0, 0, 4);
        grid.set_cell(0, 1, 6);
        grid.set_cell(0, 3, 1);
        grid.set_cell(0, 4, 9);
        grid.set_cell(0, 6, 2);

        assert!(!region_sum.validate_solution(&grid));
    }

    #[test]
    fn test_is_valid_with_partial_data() {
        let mut grid = SudokuGrid::empty();
        let line = vec![(0, 0), (0, 1), (0, 3), (0, 4), (0, 6), (0, 7)];
        let region_sum = RegionSum::new(line);

        // Box 0 partial
        grid.set_cell(0, 0, 2);

        // Other boxes empty
        // Now simulate placing 3 at (0, 1)
        let is_valid = region_sum.is_valid(&grid, 0, 1, 3);
        assert!(is_valid);
    }

    #[test]
    fn test_is_valid_exceeds_known_sum() {
        let mut grid = SudokuGrid::empty();
        let line = vec![(0, 0), (0, 1), (0, 3), (0, 4), (0, 6), (0, 7)];
        let region_sum = RegionSum::new(line);

        // Fill one segment (Box 1)
        grid.set_cell(0, 3, 4);
        grid.set_cell(0, 4, 6);

        // Now partially fill Box 0
        grid.set_cell(0, 0, 8);
        // Now try placing values in box 0 that would exceed this
        let is_valid = region_sum.is_valid(&grid, 0, 1, 3);
        assert!(!is_valid);
    }

    #[test]
    fn test_is_valid_exceeds_known_sum_incomplete_line() {
        let mut grid = SudokuGrid::empty();
        let line = vec![(0, 0), (0, 1), (0, 3), (0, 4), (0, 6), (0, 7)];
        let region_sum = RegionSum::new(line);

        // Fill one segment (Box 0) to 7
        grid.set_cell(0, 0, 6);
        grid.set_cell(0, 1, 1);

        // Try setting a cell in box 1 to 8
        let is_valid = region_sum.is_valid(&grid, 0, 3, 8);
        assert!(!is_valid);

        // Check that setting a cell in box 1 to 5 passes
        let is_valid = region_sum.is_valid(&grid, 0, 3, 5);
        assert!(is_valid);
    }

    #[test]
    fn test_is_valid_cell_not_in_line() {
        let grid = SudokuGrid::empty();
        let region_sum = RegionSum::new(vec![(0, 0), (0, 1)]); // Only those two

        let is_valid = region_sum.is_valid(&grid, 4, 4, 9); // not part of any segment
        assert!(is_valid);
    }

    #[test]
    fn test_valid_value_within_overlapping_ranges() {
        let mut grid = SudokuGrid::empty();
        let region = RegionSum::new(vec![(0, 0), (0, 1), (0, 2), (0, 3)]);

        // 3 cell region with values 4 + 2 = 6, so range between 7 and 15
        grid.set_cell(0, 0, 4);
        grid.set_cell(0, 1, 2);

        // Try filling 1-cell region with a 7 - should pass
        let is_valid = region.is_valid(&grid, 0, 3, 7);
        assert!(is_valid)
    }

    #[test]
    fn test_reject_value_exceeding_range_overlap() {
        let mut grid = SudokuGrid::empty();
        let region = RegionSum::new(vec![(0, 0), (0, 1), (0, 3), (0, 4), (0, 5)]);

        // Fill box 0 (2-cell region) with 1, _
        // Fill box 1 (3-cell region) with 6, _, _
        grid.set_cell(0, 0, 1);
        grid.set_cell(0, 3, 6);

        // Check that trying to add a 5 to box 1 will fail
        let is_valid = region.is_valid(&grid, 0, 4, 5);
        assert!(!is_valid);
    }

    #[test]
    fn test_reject_value_too_small() {
        let mut grid = SudokuGrid::empty();
        let region = RegionSum::new(vec![(0, 0), (0, 1), (0, 3), (0, 4)]);
        // Box 1 segment is already filled: 6 + 7 = 13
        grid.set_cell(0, 3, 6);
        grid.set_cell(0, 4, 7);

        // Box 0, try placing a 1, making the max sum (1+9=10) too low
        assert!(!region.is_valid(&grid, 0, 0, 1));
    }

    #[test]
    fn test_conflicting_ranges_due_to_value() {
        let mut grid = SudokuGrid::empty();
        let region = RegionSum::new(vec![
            (0, 0),
            (0, 1), // Box 0
            (0, 3),
            (0, 4), // Box 1
        ]);

        // Fill Box 1: 3 + 4 = 7
        grid.set_cell(0, 3, 3);
        grid.set_cell(0, 4, 4);

        // Now in Box 0, placing 9 in (0,0) and 9 in (0,1) → segment sum = 18
        grid.set_cell(0, 1, 9);
        let is_valid = region.is_valid(&grid, 0, 0, 9);
        assert!(!is_valid); // Segment sum = 18, no overlap with 7
    }

    #[test]
    fn test_one_cell_segment_limits_possible_sums() {
        let mut grid = SudokuGrid::empty();
        let region = RegionSum::new(vec![
            (0, 0), // Box 0 (1 cell)
            (0, 3),
            (0, 4),
            (0, 5), // Box 1 (3 cells)
        ]);

        // Place 6 in the 1-cell segment
        grid.set_cell(0, 0, 6);

        // Now try placing a value in box 1 that cannot lead to sum 6 with 3 distinct digits
        // e.g., (0,3)=9, max possible sum for 3 digits starting with 9 is 9+8+7 = 24, but we care about hitting **6**
        grid.set_cell(0, 4, 1);
        let is_valid = region.is_valid(&grid, 0, 3, 9);
        assert!(!is_valid); // Segment too long to match 6
    }

    #[test]
    fn test_different_length_segments_with_valid_overlap() {
        let mut grid = SudokuGrid::empty();
        let region = RegionSum::new(vec![
            (0, 0), // Box 0 (1 cell)
            (0, 3),
            (0, 4), // Box 1 (2 cells)
            (0, 6),
            (0, 7),
            (0, 8), // Box 2 (3 cells)
        ]);

        // Fill the 2-cell segment with 4+5=9
        grid.set_cell(0, 3, 4);
        grid.set_cell(0, 4, 5);

        // Try placing 9 in the 1-cell segment
        let is_valid = region.is_valid(&grid, 0, 0, 9);
        assert!(is_valid); // All segments can plausibly hit 9
    }

    #[test]
    fn test_display() {
        let region = RegionSum::new(vec![
            // Box 0
            (0, 1),
            (0, 2),
            // Box 1
            (1, 3),
            (2, 3),
            // Box 4
            (3, 4),
            (4, 5),
            // Box 5
            (4, 6),
        ]);
        let expected_str = String::from(
            "Region Sum Line: region 0: [(0, 1), (0, 2)], region 1: [(1, 3), (2, 3)], region 4: [(3, 4), (4, 5)], region 5: [(4, 6)]",
        );
        assert_eq!(region.to_string(), expected_str);
    }
}
