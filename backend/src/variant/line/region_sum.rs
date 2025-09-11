/*
Region sum lines: box borders divide each blue line into segments with the same sum.
*/

use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use serde::{Deserialize, Serialize};

use crate::{
    SudokuGrid, SudokuVariant,
    file_parser::parse_positions,
    variant::{
        Variant,
        error::{PossibilityResult, VariantContradiction},
    },
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RegionSum {
    // box_cells: box number: Vec<(row, col)>
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

    fn get_possibilities(&self, grid: &SudokuGrid) -> PossibilityResult {
        let mut possibilities = HashMap::new();
        let mut target_sum: Option<u8> = None;

        // 1: Try to find a target sum for any box with fully known values
        for cells in self.box_cells.values() {
            let known_vals: Vec<u8> = cells
                .iter()
                .map(|&(r, c)| grid.get_cell(r, c))
                .filter(|&val| val != 0)
                .collect();

            if known_vals.len() == cells.len() {
                target_sum = Some(known_vals.iter().sum());
                break;
            }
        }

        // 2: Try to infer a possible target sum if none is known
        let mut candidate_sums: HashSet<u8> = HashSet::new();
        if target_sum.is_none() {
            let mut sets_per_box = Vec::new();
            for cells in self.box_cells.values() {
                let known_vals: Vec<u8> = cells
                    .iter()
                    .map(|&(r, c)| grid.get_cell(r, c))
                    .filter(|&val| val != 0)
                    .collect();

                let unknown_count = cells.len() - known_vals.len();
                if unknown_count == 0 {
                    // Already handled above
                    continue;
                }
                let known_sum: u8 = known_vals.iter().sum();
                let min_possible_sum = known_sum + unknown_count as u8; // All 1s
                let max_possible_sum = known_sum + (9 * unknown_count) as u8; // All 9s

                sets_per_box.push((min_possible_sum..=max_possible_sum).collect::<HashSet<_>>());
            }

            // Intersect candidate sets across all boxes
            if !sets_per_box.is_empty() {
                let mut iter = sets_per_box.into_iter();
                let first = iter.next().unwrap();
                candidate_sums =
                    iter.fold(first, |acc, set| acc.intersection(&set).copied().collect());
            }

            // No valid common target
            if candidate_sums.is_empty() {
                for cells in self.box_cells.values() {
                    for &(r, c) in cells {
                        let val = grid.get_cell(r, c);
                        if val != 0 {
                            possibilities.insert((r, c), vec![val]);
                        } else {
                            possibilities.insert((r, c), vec![]);
                        }
                    }
                }
                return Ok(possibilities);
            }
        }

        // 3: For each box, determine possible values for unknowns
        for cells in self.box_cells.values() {
            let known_vals: Vec<u8> = cells
                .iter()
                .map(|&(r, c)| grid.get_cell(r, c))
                .filter(|&v| v != 0)
                .collect();

            let known_sum: u8 = known_vals.iter().sum();
            let unknown_cells: Vec<(usize, usize)> = cells
                .iter()
                .copied()
                .filter(|&(r, c)| grid.get_cell(r, c) == 0)
                .collect();

            let remaining_cells = unknown_cells.len();

            // For already filled cells - just that value
            for &(r, c) in cells {
                let val = grid.get_cell(r, c);
                if val != 0 {
                    possibilities.insert((r, c), vec![val]);
                }
            }

            // For unknown cells - compute possibilities
            for &(r, c) in &unknown_cells {
                let mut range = HashSet::new();
                let possible_sums = if let Some(ts) = target_sum {
                    std::iter::once(ts).collect()
                } else {
                    candidate_sums.clone()
                };
                for sum in possible_sums {
                    if known_sum > sum {
                        // impossible
                        continue;
                    }

                    let remaining_sum = sum - known_sum;
                    let min_val =
                        1.max(remaining_sum.saturating_sub((remaining_cells - 1) as u8 * 9));
                    let max_val = 9.min(remaining_sum.saturating_sub((remaining_cells - 1) as u8));
                    for v in min_val..=max_val {
                        range.insert(v);
                    }
                }

                if range.is_empty() {
                    return Err(VariantContradiction::NoPossibilities {
                        cell: (r, c),
                        variant: "RegionSum",
                        reason: String::from("No possible range"),
                    });
                }
                let mut vec_range: Vec<u8> = range.into_iter().collect();
                vec_range.sort_unstable();
                possibilities.insert((r, c), vec_range);
            }
        }

        Ok(possibilities)
    }
}

impl Display for RegionSum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut box_cell_entries: Vec<(&usize, &Vec<(usize, usize)>)> =
            self.box_cells.iter().collect();
        box_cell_entries.sort_by_key(|(box_num, _)| *box_num);
        let mut output = String::from("Region Sum Line:");
        for (box_num, cells) in box_cell_entries.iter() {
            output.push_str(&format!(" region {box_num}: ["));
            output.push_str(
                cells
                    .iter()
                    .map(|&(r, c)| format!("({r}, {c})"))
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
    use crate::{
        SudokuGrid,
        variant::{Variant, VariantContradiction},
    };

    use super::RegionSum;

    #[test]
    fn test_get_possibilities_basic() {
        let mut grid = SudokuGrid::empty();
        // Pre-fill known cells
        grid.set_cell(1, 0, 2);
        grid.set_cell(2, 0, 3);
        let region_sum = RegionSum::new(vec![(1, 0), (2, 0), (0, 3), (1, 3)]);
        let result = region_sum.get_possibilities(&grid);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result.get(&(1, 0)), Some(&vec![2]));
        assert_eq!(result.get(&(2, 0)), Some(&vec![3]));
        assert_eq!(result.get(&(0, 3)).unwrap(), &vec![1, 2, 3, 4]);
        assert_eq!(result.get(&(1, 3)).unwrap(), &vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_get_possibilities_highly_constrained() {
        let mut grid = SudokuGrid::empty();
        // Pre-fill one cell in each region
        grid.set_cell(1, 0, 1);
        grid.set_cell(0, 3, 9);
        let rs = RegionSum::new(vec![(1, 0), (2, 0), (0, 3), (1, 3)]);
        let result = rs.get_possibilities(&grid);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result.get(&(1, 0)), Some(&vec![1]));
        assert_eq!(result.get(&(2, 0)), Some(&vec![9]));
        assert_eq!(result.get(&(2, 0)).unwrap(), &vec![9]);
        assert_eq!(result.get(&(1, 3)).unwrap(), &vec![1]);
    }

    #[test]
    fn test_get_possibilities_not_fully_known() {
        let mut grid = SudokuGrid::empty();
        // Pre-fill one cell in each region
        grid.set_cell(1, 0, 4);
        grid.set_cell(0, 3, 5);
        let rs = RegionSum::new(vec![(1, 0), (2, 0), (0, 3), (1, 3)]);
        let result = rs.get_possibilities(&grid);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result.get(&(1, 0)), Some(&vec![4]));
        assert_eq!(result.get(&(0, 3)), Some(&vec![5]));
        assert_eq!(result.get(&(2, 0)).unwrap(), &vec![2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(result.get(&(1, 3)).unwrap(), &vec![1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_get_possibilities_fully_known() {
        let mut grid = SudokuGrid::empty();
        grid.set_cell(1, 0, 2);
        grid.set_cell(2, 0, 5);
        grid.set_cell(0, 3, 4);
        grid.set_cell(1, 3, 3);

        let rs = RegionSum::new(vec![(1, 0), (2, 0), (0, 3), (1, 3)]);
        let result = rs.get_possibilities(&grid);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result.get(&(1, 0)), Some(&vec![2]));
        assert_eq!(result.get(&(2, 0)), Some(&vec![5]));
        assert_eq!(result.get(&(0, 3)), Some(&vec![4]));
        assert_eq!(result.get(&(1, 3)), Some(&vec![3]));
    }

    #[test]
    fn test_get_possibilities_inconsistent_sum() {
        let mut grid = SudokuGrid::empty();
        grid.set_cell(1, 0, 1); // Box 1 - partial sum is only 1 so far
        grid.set_cell(0, 3, 9);
        grid.set_cell(1, 3, 8); // Box 2 partial sum = 17

        let rs = RegionSum::new(vec![(1, 0), (2, 0), (0, 3), (1, 3)]);
        let result = rs.get_possibilities(&grid);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(VariantContradiction::NoPossibilities { cell: (2, 0), .. })
        ));
    }

    #[test]
    fn test_get_possibilities_partial_and_known_boxes() {
        let mut grid = SudokuGrid::empty();
        grid.set_cell(1, 0, 3);
        grid.set_cell(2, 0, 4); // sum = 7 (known box)
        grid.set_cell(0, 3, 0);
        grid.set_cell(1, 3, 0); // sum = ?? (unknowns)

        let rs = RegionSum::new(vec![(1, 0), (2, 0), (0, 3), (1, 3)]);
        let result = rs.get_possibilities(&grid);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result.get(&(1, 0)), Some(&vec![3]));
        assert_eq!(result.get(&(2, 0)), Some(&vec![4]));
        assert_eq!(result.get(&(0, 3)).unwrap(), &vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(result.get(&(1, 3)).unwrap(), &vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_get_possibilities_single_cell_region() {
        let grid = SudokuGrid::empty();
        let rs = RegionSum::new(vec![(0, 0)]);

        let result = rs.get_possibilities(&grid);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.get(&(0, 0)).unwrap(),
            &vec![1, 2, 3, 4, 5, 6, 7, 8, 9]
        );
    }

    #[test]
    fn test_get_possibilities_cell_not_on_line() {
        let mut grid = SudokuGrid::empty();
        grid.set_cell(1, 0, 3);

        let rs = RegionSum::new(vec![(0, 0), (0, 1)]);
        let result = rs.get_possibilities(&grid);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 2);
        let expected = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(result.get(&(0, 0)), Some(&expected));
        assert_eq!(result.get(&(0, 1)), Some(&expected));
    }

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
