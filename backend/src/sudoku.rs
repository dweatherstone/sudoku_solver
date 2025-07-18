use std::{
    collections::{HashMap, HashSet},
    io::Error,
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::{
    Arrow, Diagonal, Entropic, KillerCage, KropkiDot, QuadrupleCircle, Renban, Shaded, Thermometer,
    XVDot, file_parser,
    variant::{GermanWhisper, RegionSum, Variant},
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum SudokuVariant {
    Arrow(Arrow),
    Diagonal(Diagonal),
    Entropic(Entropic),
    Killer(KillerCage),
    Kropki(KropkiDot),
    QuadrupleCircles(QuadrupleCircle),
    RegionSum(RegionSum),
    Renban(Renban),
    Thermometer(Thermometer),
    XVDot(XVDot),
    GermanWhisper(GermanWhisper),
    Shaded(Shaded),
}

impl SudokuVariant {
    pub fn parse(line: &str) -> Option<SudokuVariant> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
        if parts.len() < 2 {
            return None;
        }

        let variant_type = parts[0].trim().to_lowercase();
        let data = parts[1].trim();

        match variant_type.as_str() {
            "killer" => KillerCage::parse(data),
            "diagonal" => Diagonal::parse(data),
            "thermometer" => Thermometer::parse(data),
            "kropki" => KropkiDot::parse(data),
            "quadruple" => QuadrupleCircle::parse(data, false),
            "anti quadruple" => QuadrupleCircle::parse(data, true),
            "renban" => Renban::parse(data),
            "entropic" => Entropic::parse(data),
            "arrow" => Arrow::parse(data),
            "region sum" => RegionSum::parse(data),
            "xv" => XVDot::parse(data),
            "german whisper" => GermanWhisper::parse(data),
            "shaded" => Shaded::parse(data),
            _ => None,
        }
    }

    pub fn is_valid(&self, grid: &SudokuGrid, row: usize, col: usize, value: u8) -> bool {
        match self {
            SudokuVariant::Diagonal(diag) => diag.is_valid(grid, row, col, value),
            SudokuVariant::Killer(cage) => cage.is_valid(grid, row, col, value),
            SudokuVariant::Kropki(dot) => dot.is_valid(grid, row, col, value),
            SudokuVariant::QuadrupleCircles(circle) => circle.is_valid(grid, row, col, value),
            SudokuVariant::Renban(ren) => ren.is_valid(grid, row, col, value),
            SudokuVariant::Thermometer(therm) => therm.is_valid(grid, row, col, value),
            SudokuVariant::Entropic(ent) => ent.is_valid(grid, row, col, value),
            SudokuVariant::Arrow(arrow) => arrow.is_valid(grid, row, col, value),
            SudokuVariant::RegionSum(rs) => rs.is_valid(grid, row, col, value),
            SudokuVariant::XVDot(xv) => xv.is_valid(grid, row, col, value),
            SudokuVariant::GermanWhisper(gw) => gw.is_valid(grid, row, col, value),
            SudokuVariant::Shaded(s) => s.is_valid(grid, row, col, value),
        }
    }

    pub fn validate_solution(&self, grid: &SudokuGrid) -> bool {
        match self {
            SudokuVariant::Diagonal(diag) => diag.validate_solution(grid),
            SudokuVariant::Killer(cage) => cage.validate_solution(grid),
            SudokuVariant::Kropki(dot) => dot.validate_solution(grid),
            SudokuVariant::QuadrupleCircles(circle) => circle.validate_solution(grid),
            SudokuVariant::Renban(ren) => ren.validate_solution(grid),
            SudokuVariant::Thermometer(therm) => therm.validate_solution(grid),
            SudokuVariant::Entropic(ent) => ent.validate_solution(grid),
            SudokuVariant::Arrow(arrow) => arrow.validate_solution(grid),
            SudokuVariant::RegionSum(rs) => rs.validate_solution(grid),
            SudokuVariant::XVDot(xv) => xv.validate_solution(grid),
            SudokuVariant::GermanWhisper(gw) => gw.validate_solution(grid),
            SudokuVariant::Shaded(s) => s.validate_solution(grid),
        }
    }

    pub fn constrained_cells(&self) -> Vec<(usize, usize)> {
        match self {
            SudokuVariant::Diagonal(diag) => diag.constrained_cells(),
            SudokuVariant::Killer(cage) => cage.constrained_cells(),
            SudokuVariant::Kropki(dot) => dot.constrained_cells(),
            SudokuVariant::QuadrupleCircles(circle) => circle.constrained_cells(),
            SudokuVariant::Renban(ren) => ren.constrained_cells(),
            SudokuVariant::Thermometer(therm) => therm.constrained_cells(),
            SudokuVariant::Entropic(ent) => ent.constrained_cells(),
            SudokuVariant::Arrow(arrow) => arrow.constrained_cells(),
            SudokuVariant::RegionSum(rs) => rs.constrained_cells(),
            SudokuVariant::XVDot(xv) => xv.constrained_cells(),
            SudokuVariant::GermanWhisper(gw) => gw.constrained_cells(),
            SudokuVariant::Shaded(s) => s.constrained_cells(),
        }
    }

    pub fn get_possibilities(
        &self,
        grid: &SudokuGrid,
        row: usize,
        col: usize,
    ) -> HashMap<(usize, usize), Vec<u8>> {
        match self {
            SudokuVariant::Diagonal(diag) => diag.get_possibilities(grid, row, col),
            SudokuVariant::Killer(cage) => cage.get_possibilities(grid, row, col),
            SudokuVariant::Kropki(dot) => dot.get_possibilities(grid, row, col),
            SudokuVariant::QuadrupleCircles(circle) => circle.get_possibilities(grid, row, col),
            SudokuVariant::Renban(ren) => ren.get_possibilities(grid, row, col),
            SudokuVariant::Thermometer(therm) => therm.get_possibilities(grid, row, col),
            SudokuVariant::Entropic(ent) => ent.get_possibilities(grid, row, col),
            SudokuVariant::Arrow(arrow) => arrow.get_possibilities(grid, row, col),
            SudokuVariant::RegionSum(rs) => rs.get_possibilities(grid, row, col),
            SudokuVariant::XVDot(xv) => xv.get_possibilities(grid, row, col),
            SudokuVariant::GermanWhisper(gw) => gw.get_possibilities(grid, row, col),
            SudokuVariant::Shaded(s) => s.get_possibilities(grid, row, col),
        }
    }
}

impl std::fmt::Display for SudokuVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SudokuVariant::Diagonal(diag) => write!(f, "{diag}"),
            SudokuVariant::Killer(cage) => write!(f, "{cage}"),
            SudokuVariant::Kropki(dot) => write!(f, "{dot}"),
            SudokuVariant::QuadrupleCircles(circle) => write!(f, "{circle}"),
            SudokuVariant::Renban(ren) => write!(f, "{ren}"),
            SudokuVariant::Thermometer(therm) => write!(f, "{therm}"),
            SudokuVariant::Entropic(ent) => write!(f, "{ent}"),
            SudokuVariant::Arrow(arrow) => write!(f, "{arrow}"),
            SudokuVariant::RegionSum(rs) => write!(f, "{rs}"),
            SudokuVariant::XVDot(xv) => write!(f, "{xv}"),
            SudokuVariant::GermanWhisper(gw) => write!(f, "{gw}"),
            SudokuVariant::Shaded(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SudokuGrid {
    cells: [[u8; 9]; 9],
    possibilities: HashMap<(usize, usize), Vec<u8>>,
    variants: Vec<SudokuVariant>,
}

impl SudokuGrid {
    pub fn empty() -> Self {
        let mut possibilities = HashMap::new();
        for r in 0..9 {
            for c in 0..9 {
                possibilities.insert((r, c), (1..=9).collect());
            }
        }
        SudokuGrid {
            cells: [[0; 9]; 9],
            possibilities,
            variants: Vec::new(),
        }
    }

    pub fn get_cell(&self, row: usize, col: usize) -> u8 {
        self.cells[row][col]
    }

    pub fn get_possibilities(&self, row: usize, col: usize) -> Vec<u8> {
        self.possibilities
            .get(&(row, col))
            .unwrap_or(&vec![self.get_cell(row, col)])
            .clone()
    }

    pub fn get_cells(&self) -> [[u8; 9]; 9] {
        self.cells
    }

    pub fn variants(&self) -> impl Iterator<Item = &SudokuVariant> {
        self.variants.iter()
    }

    pub fn set_cell(&mut self, row: usize, col: usize, value: u8) {
        self.cells[row][col] = value;
        if value == 0 {
            *self
                .possibilities
                .entry((row, col))
                .or_insert(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]) = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        } else {
            *self.possibilities.entry((row, col)).or_insert(vec![value]) = vec![value];
        }
        // Update the possibilities to remove value for the row, column and box, and apply variant logic
        self.update_possibilities(row, col, value);
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
                print!("{cell_str} ");
            }
            println!();
        }
        if show_variants {
            println!("Variants:");
            for variant in &self.variants {
                println!("{variant}");
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

    pub fn read_from_file(path: &Path) -> Result<Self, Error> {
        file_parser::parse_file(path)
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
    }

    pub fn is_board_valid(&self) -> bool {
        // Check rows
        for row in 0..9 {
            if !Self::is_valid_group(&self.cells[row]) {
                return false;
            }
        }

        // Check columns
        for col in 0..9 {
            let mut column = [0u8; 9];
            for row in 0..9 {
                column[row] = self.cells[row][col];
            }
            if !Self::is_valid_group(&column) {
                return false;
            }
        }

        // Check 3x3 boxes
        for box_row in 0..3 {
            for box_col in 0..3 {
                let mut block = [0u8; 9];
                for i in 0..3 {
                    for j in 0..3 {
                        block[i * 3 + j] = self.cells[box_row * 3 + i][box_col * 3 + j];
                    }
                }
                if !Self::is_valid_group(&block) {
                    return false;
                }
            }
        }
        true
    }

    fn is_classic_valid(&self, row: usize, col: usize, num: u8) -> bool {
        !self.used_in_row(row, num)
            && !self.used_in_col(col, num)
            && !self.used_in_subgrid(row - row % 3, col - col % 3, num)
    }

    fn is_valid_group(group: &[u8; 9]) -> bool {
        let mut seen = HashSet::with_capacity(9);
        for &num in group {
            if !(1..=9).contains(&num) || !seen.insert(num) {
                return false;
            }
        }
        true
    }

    fn update_possibilities(&mut self, row: usize, col: usize, value: u8) {
        // Remove value from possibilities in the same row
        for c in 0..9 {
            if c != col {
                if let Some(poss) = self.possibilities.get_mut(&(row, c)) {
                    poss.retain(|&p| p != value);
                }
            }
        }
        // Remove value from possibilities in the same col
        for r in 0..9 {
            if r != row {
                if let Some(poss) = self.possibilities.get_mut(&(r, col)) {
                    poss.retain(|&p| p != value);
                }
            }
        }
        // Remove value from possibilities in the same box
        let box_row = row / 3 * 3;
        let box_col = col / 3 * 3;
        for r in box_row..box_row + 3 {
            for c in box_col..box_col + 3 {
                if r != row || c != col {
                    if let Some(poss) = self.possibilities.get_mut(&(r, c)) {
                        poss.retain(|&p| p != value);
                    }
                }
            }
        }
        // Now apply variant constraints to further reduce possibilies
        for variant in &self.variants {
            if variant.constrained_cells().contains(&(row, col)) {
                for (&(r, c), var_poss) in variant.get_possibilities(&self, row, col).iter() {
                    if let Some(poss) = self.possibilities.get_mut(&(r, c)) {
                        poss.retain(|p| var_poss.contains(p));
                    }
                }
            }
        }
    }
}

impl Default for SudokuGrid {
    fn default() -> Self {
        SudokuGrid::empty()
    }
}
