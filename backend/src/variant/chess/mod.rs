mod king;
mod knight;

pub use king::King;
pub use knight::Knight;

fn get_all_cells() -> Vec<(usize, usize)> {
    (0..9)
        .flat_map(|row| (0..9).map(move |col| (row, col)))
        .collect()
}
