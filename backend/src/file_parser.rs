use crate::{SudokuGrid, SudokuVariant};
use std::{
    fs::File,
    io::{BufRead, BufReader, Error, ErrorKind},
};

pub fn parse_file(filename: &str) -> Result<SudokuGrid, Error> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut sudoku_grid = SudokuGrid::default();

    for row in 0..9 {
        let line = lines.next().ok_or_else(|| {
            Error::new(
                ErrorKind::UnexpectedEof,
                "Unexpected end of file while reading grid",
            )
        })??;

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

    // Process any variants in the file
    for line in lines {
        let line = line?.trim().to_string();
        if let Some(variant) = SudokuVariant::parse(&line) {
            sudoku_grid.add_variant(variant);
        } else if line.eq_ignore_ascii_case("solution:") {
            break;
        } else if !line.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid variant line: {}", line),
            ));
        }
    }
    Ok(sudoku_grid)
}

pub fn parse_positions(data: &str) -> Result<Vec<(usize, usize)>, Error> {
    let mut positions = Vec::new();
    let re = regex::Regex::new(r"\((\d+),\s*(\d+)\)").unwrap();

    for cap in re.captures_iter(data) {
        let row = cap[1]
            .parse::<usize>()
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid row index in position"))?;
        let col = cap[2]
            .parse::<usize>()
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid column index in position"))?;
        positions.push((row, col));
    }

    if positions.is_empty() {
        return Err(Error::new(ErrorKind::InvalidData, "No valid positions"));
    }

    Ok(positions)
}

#[cfg(test)]
mod tests {
    use crate::{Diagonal, KillerCage, SudokuVariant, Thermometer, file_parser::parse_positions};

    use super::parse_file;

    #[test]
    fn test_read_no_variants() {
        let filename = "sudoku.txt";
        let grid = parse_file(filename).unwrap();
        let expected_grid = [
            [0, 0, 9, 0, 0, 0, 0, 0, 4],
            [0, 2, 4, 0, 9, 0, 0, 0, 0],
            [0, 0, 0, 4, 0, 0, 3, 9, 2],
            [1, 7, 2, 6, 0, 8, 9, 0, 3],
            [4, 5, 3, 9, 7, 1, 0, 0, 8],
            [0, 9, 0, 2, 0, 3, 7, 0, 0],
            [0, 0, 0, 7, 0, 0, 5, 0, 9],
            [0, 3, 0, 0, 8, 0, 0, 0, 0],
            [0, 0, 1, 0, 0, 0, 0, 0, 6],
        ];
        #[allow(clippy::needless_range_loop)]
        for row in 0..9 {
            for col in 0..9 {
                assert_eq!(
                    grid.get_cell(row, col),
                    expected_grid[row][col],
                    "Unexpected cell value ({}, {}). Expected: {}. Got: {}",
                    row,
                    col,
                    expected_grid[row][col],
                    grid.get_cell(row, col)
                );
            }
        }
        assert!(
            grid.variants().next().is_none(),
            "Variants should be empty."
        );
    }

    #[test]
    fn test_read_draft_day() {
        let filename = "draft_day.txt";
        let grid = parse_file(filename).unwrap();
        #[allow(clippy::needless_range_loop)]
        for row in 0..9 {
            for col in 0..9 {
                assert_eq!(
                    grid.get_cell(row, col),
                    0,
                    "Unexpected cell value ({}, {}). Expected: {}. Got: {}",
                    row,
                    col,
                    0,
                    grid.get_cell(row, col)
                );
            }
        }
        let expected_variants = vec![
            SudokuVariant::Killer(KillerCage::new(vec![(0, 1), (0, 2)], 11)),
            SudokuVariant::Killer(KillerCage::new(vec![(0, 6), (0, 7), (1, 6)], 6)),
            SudokuVariant::Killer(KillerCage::new(vec![(1, 8), (2, 7), (2, 8)], 24)),
            SudokuVariant::Killer(KillerCage::new(vec![(1, 0), (1, 2)], 5)),
            SudokuVariant::Killer(KillerCage::new(vec![(4, 3), (5, 3), (5, 4)], 13)),
            SudokuVariant::Killer(KillerCage::new(vec![(6, 0), (6, 1), (7, 0)], 24)),
            SudokuVariant::Killer(KillerCage::new(vec![(6, 7), (6, 8)], 11)),
            SudokuVariant::Killer(KillerCage::new(vec![(8, 7), (8, 8)], 8)),
            SudokuVariant::Diagonal(Diagonal::new(true)),
            SudokuVariant::Thermometer(Thermometer::new(vec![
                (8, 4),
                (7, 3),
                (6, 2),
                (5, 1),
                (4, 0),
                (3, 0),
            ])),
            SudokuVariant::Thermometer(Thermometer::new(vec![
                (6, 7),
                (5, 7),
                (4, 6),
                (3, 5),
                (2, 4),
                (1, 3),
            ])),
        ];
        // Compare number of parsed variants
        let parsed_variants: Vec<&SudokuVariant> = grid.variants().collect();
        assert_eq!(
            parsed_variants.len(),
            expected_variants.len(),
            "Mismatch in number of variants. Expected: {}. Got: {}",
            expected_variants.len(),
            parsed_variants.len(),
        );

        // Compare each variant
        for (idx, (expected, actual)) in expected_variants.iter().zip(parsed_variants).enumerate() {
            assert_eq!(
                actual, expected,
                "Variant at index {} did not match.\nExpected: {:?}\nGor: {:?}",
                idx, expected, actual
            );
        }
    }

    #[test]
    fn test_parse_positions_valid_input() {
        let input = "((0, 1), (0, 2))";
        let expected = vec![(0, 1), (0, 2)];
        let result = parse_positions(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_positions_empty() {
        let input = "()";
        let result = parse_positions(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_positions_malformed() {
        let input = "(a, 1)";
        let result = parse_positions(input);
        assert!(result.is_err());
    }
}
