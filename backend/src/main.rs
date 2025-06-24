#![allow(dead_code)]
#![allow(unused_imports)]
use axum::{
    Json, Router,
    extract::Path,
    http::{Method, StatusCode, header::CONTENT_TYPE},
    response::IntoResponse,
    routing::{get, post},
    serve,
};
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use std::{env, path::PathBuf};
use sudoku_solver::{
    Diagonal, KillerCage, KropkiDot, QuadrupleCircle, Solver, SudokuGrid, SudokuVariant,
    Thermometer, get_examples_path,
};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

// Global state
struct AppState {
    grid: RwLock<SudokuGrid>,
}

async fn sudoku_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> impl IntoResponse {
    let grid = state.grid.read().await;
    Json(grid.clone())
}

async fn solve_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Result<Json<SudokuGrid>, StatusCode> {
    let mut grid = state.grid.write().await;
    let mut solver = Solver::new(&mut grid);

    if solver.solve(false) {
        Ok(Json(grid.clone()))
    } else {
        Err(StatusCode::UNPROCESSABLE_ENTITY)
    }
}

async fn set_cell_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Path((row, col, value)): Path<(usize, usize, u8)>,
) -> Result<Json<SudokuGrid>, StatusCode> {
    let mut grid = state.grid.write().await;

    // Validate the move
    if !grid.is_valid_move(row, col, value) {
        return Err(StatusCode::BAD_REQUEST);
    }

    grid.set_cell(row, col, value);
    Ok(Json(grid.clone()))
}

// #[tokio::main]
// async fn main() {
//     let cors = CorsLayer::new()
//         .allow_origin(Any)
//         .allow_methods([Method::GET, Method::POST])
//         .allow_headers([CONTENT_TYPE]);

//     // Initialize the grid
//     let grid = draft_day(false);
//     let state = Arc::new(AppState {
//         grid: RwLock::new(grid),
//     });

//     let app = Router::new()
//         .route("/sudoku", get(sudoku_handler))
//         .route("/solve", post(solve_handler))
//         .route("/cell/{row}/{col}/{value}", post(set_cell_handler))
//         .with_state(state)
//         .layer(cors);

//     let listener = TcpListener::bind("127.0.0.1:3000")
//         .await
//         .expect("Failed to bind listener");
//     println!("Running on http://localhost:3000");
//     serve(listener, app).await.expect("Server error");
// }

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        //killer_example();
        //building_blocks(true);
        //quadruple_circles_example(true);
        // kropki_example(true);
        // draft_day(true);
        // ultraviolet(true);
        triumvirate(true);
        return Ok(());
    }
    let filename = &args[1];
    let mut path = PathBuf::from(get_examples_path());
    path.push(filename);

    let mut sudoku_grid = SudokuGrid::read_from_file(&path).map_err(|e| {
        eprintln!("Error reading sudoku puzzle: {}", e);
        Error::other("Failed to read Sudoku puzzle")
    })?;

    let display_variants = sudoku_grid.variants().next().is_some();
    run_solve(&mut sudoku_grid, display_variants, false);

    Ok(())
}

fn building_blocks(do_solve: bool) -> SudokuGrid {
    // https://artisanalsudoku.substack.com/p/artisanal-sudoku-volume-178
    let mut sudoku_grid = SudokuGrid::default();

    // Killer Cages
    let cages = [
        (vec![(0, 0), (0, 1), (1, 0), (1, 1)], 14),
        (vec![(0, 4), (0, 5), (1, 4), (1, 5)], 28),
        (vec![(3, 7), (3, 8), (4, 7), (4, 8)], 28),
        (vec![(7, 0), (7, 1), (8, 0), (8, 1)], 26),
        (vec![(7, 7), (7, 8), (8, 7), (8, 8)], 15),
    ];
    for (cells, sum) in cages {
        sudoku_grid.add_variant(SudokuVariant::Killer(KillerCage::new(cells, sum)));
    }

    // Kropki Dots
    let dots = [
        (vec![(2, 5), (2, 6)], "black"),
        (vec![(3, 0), (3, 1)], "white"),
        (vec![(4, 7), (5, 7)], "white"),
        (vec![(8, 5), (8, 6)], "black"),
    ];
    for (cells, colour) in dots {
        sudoku_grid.add_variant(SudokuVariant::Kropki(KropkiDot::new(cells, colour)));
    }

    // Quadruple Circles
    let circles = [
        (vec![(1, 6), (1, 7), (2, 6), (2, 7)], vec![1, 2, 3]),
        (vec![(2, 3), (2, 4), (3, 3), (3, 4)], vec![1, 2, 3]),
        (vec![(3, 5), (3, 6), (4, 5), (4, 6)], vec![1, 2, 3]),
        (vec![(4, 2), (4, 3), (5, 2), (5, 3)], vec![1, 2, 3]),
        (vec![(5, 4), (5, 5), (6, 4), (6, 5)], vec![1, 2, 4]),
        (vec![(6, 1), (6, 2), (7, 1), (7, 2)], vec![1, 2, 3]),
    ];
    for (cells, required) in circles {
        sudoku_grid.add_variant(SudokuVariant::QuadrupleCircles(QuadrupleCircle::new(
            cells, required,
        )));
    }

    // Positions 1
    sudoku_grid.set_cell(1, 7, 1);
    sudoku_grid.set_cell(2, 4, 1);
    sudoku_grid.set_cell(6, 4, 2);
    sudoku_grid.set_cell(6, 5, 1);
    sudoku_grid.set_cell(7, 1, 2);
    sudoku_grid.set_cell(7, 2, 1);
    // Positions 2
    sudoku_grid.set_cell(0, 0, 2);
    sudoku_grid.set_cell(0, 1, 1);
    sudoku_grid.set_cell(1, 0, 3);
    sudoku_grid.set_cell(1, 1, 8);
    sudoku_grid.set_cell(1, 6, 2);
    sudoku_grid.set_cell(1, 7, 1);
    sudoku_grid.set_cell(2, 3, 2);
    sudoku_grid.set_cell(3, 2, 8);
    sudoku_grid.set_cell(3, 5, 2);
    sudoku_grid.set_cell(3, 6, 1);
    sudoku_grid.set_cell(4, 2, 2);
    sudoku_grid.set_cell(4, 6, 3);
    sudoku_grid.set_cell(4, 8, 8);
    sudoku_grid.set_cell(5, 2, 3);
    sudoku_grid.set_cell(5, 8, 2);
    sudoku_grid.set_cell(6, 1, 3);

    if do_solve {
        run_solve(&mut sudoku_grid, false, false);
    }
    sudoku_grid
}

fn killer_example() {
    let mut sudoku_grid = SudokuGrid::default();

    let cages = [
        (vec![(0, 0), (0, 1)], 8),
        (vec![(0, 2), (1, 2)], 11),
        (vec![(0, 3), (1, 3)], 12),
        (vec![(0, 4), (0, 5)], 4),
        (vec![(0, 6), (0, 7)], 17),
        (vec![(0, 8), (1, 8)], 11),
        (vec![(1, 0), (2, 0)], 8),
        (vec![(2, 1), (2, 2)], 17),
        (vec![(2, 3), (2, 4)], 11),
        (vec![(1, 5), (2, 5)], 16),
        (vec![(1, 6), (2, 6)], 8),
        (vec![(2, 7), (2, 8)], 6),
        (vec![(3, 0), (4, 0)], 15),
        (vec![(3, 1), (3, 2)], 5),
        (vec![(3, 3), (3, 4)], 5),
        (vec![(3, 5), (4, 5)], 10),
        (vec![(3, 6), (4, 6)], 8),
        (vec![(3, 7), (3, 8)], 16),
        (vec![(5, 0), (5, 1)], 15),
        (vec![(4, 2), (5, 2)], 6),
        (vec![(4, 3), (5, 3)], 16),
        (vec![(5, 4), (5, 5)], 9),
        (vec![(5, 6), (5, 7)], 3),
        (vec![(4, 8), (5, 8)], 12),
        (vec![(6, 0), (6, 1)], 9),
        (vec![(6, 2), (7, 2)], 8),
        (vec![(6, 3), (7, 3)], 9),
        (vec![(6, 4), (6, 5)], 10),
        (vec![(6, 6), (6, 7)], 10),
        (vec![(6, 8), (7, 8)], 10),
        (vec![(7, 0), (8, 0)], 9),
        (vec![(8, 1), (8, 2)], 12),
        (vec![(8, 3), (8, 4)], 9),
        (vec![(7, 5), (8, 5)], 9),
        (vec![(7, 6), (8, 6)], 10),
        (vec![(8, 7), (8, 8)], 6),
    ];

    for (cells, sum) in cages {
        sudoku_grid.add_variant(SudokuVariant::Killer(KillerCage::new(cells, sum)));
    }

    sudoku_grid.set_cell(1, 1, 1);
    sudoku_grid.set_cell(1, 4, 2);
    sudoku_grid.set_cell(1, 7, 3);
    sudoku_grid.set_cell(4, 1, 4);
    sudoku_grid.set_cell(4, 4, 5);
    sudoku_grid.set_cell(4, 7, 6);
    sudoku_grid.set_cell(7, 1, 7);
    sudoku_grid.set_cell(7, 4, 8);
    sudoku_grid.set_cell(7, 7, 9);

    run_solve(&mut sudoku_grid, false, false);
}

fn quadruple_circles_example(do_solve: bool) -> SudokuGrid {
    // https://nrich.maths.org/problems/quadruple-sudoku
    let mut sudoku_grid = SudokuGrid::default();
    let circles = [
        (vec![(0, 0), (0, 1), (1, 0), (1, 1)], vec![1, 6, 7, 8]),
        (vec![(0, 3), (0, 4), (1, 3), (1, 4)], vec![1, 5, 7, 8]),
        (vec![(0, 7), (0, 8), (1, 7), (1, 8)], vec![2, 3, 5, 6]),
        (vec![(1, 1), (1, 2), (2, 1), (2, 2)], vec![1, 2, 4, 9]),
        (vec![(1, 4), (1, 5), (2, 4), (2, 5)], vec![3, 4, 6, 8]),
        (vec![(1, 6), (1, 7), (2, 6), (2, 7)], vec![1, 5, 7, 9]),
        (vec![(2, 0), (2, 1), (3, 0), (3, 1)], vec![4, 5, 7, 8]),
        (vec![(2, 3), (2, 4), (3, 3), (3, 4)], vec![2, 3, 5, 6]),
        (vec![(2, 7), (2, 8), (3, 7), (3, 8)], vec![1, 4, 8, 9]),
        (vec![(3, 1), (3, 2), (4, 1), (4, 2)], vec![1, 3, 6, 8]),
        (vec![(4, 2), (4, 3), (5, 2), (5, 3)], vec![4, 4, 6, 6]),
        (vec![(4, 6), (4, 7), (5, 6), (5, 7)], vec![1, 3, 5, 8]),
        (vec![(5, 7), (5, 8), (6, 7), (6, 8)], vec![3, 5, 6, 7]),
        (vec![(6, 1), (6, 2), (7, 1), (7, 2)], vec![2, 6, 7, 8]),
        (vec![(6, 3), (6, 4), (7, 3), (7, 4)], vec![1, 4, 8, 9]),
        (vec![(6, 6), (6, 7), (7, 6), (7, 7)], vec![2, 3, 6, 9]),
        (vec![(7, 0), (7, 1), (8, 0), (8, 1)], vec![1, 3, 6, 9]),
        (vec![(7, 4), (7, 5), (8, 4), (8, 5)], vec![2, 4, 5, 6]),
    ];
    for (cells, required) in circles {
        sudoku_grid.add_variant(SudokuVariant::QuadrupleCircles(QuadrupleCircle::new(
            cells, required,
        )));
    }
    if do_solve {
        run_solve(&mut sudoku_grid, true, false);
    }
    sudoku_grid
}

fn kropki_example(do_solve: bool) -> SudokuGrid {
    // https://escape-sudoku.com/game/dots
    let mut grid = SudokuGrid::default();
    let black_dots = [
        vec![(0, 5), (0, 6)],
        vec![(1, 2), (2, 2)],
        vec![(1, 5), (2, 5)],
        vec![(3, 2), (4, 2)],
        vec![(3, 4), (4, 4)],
        vec![(6, 6), (7, 6)],
    ];
    for cells in black_dots {
        grid.add_variant(SudokuVariant::Kropki(KropkiDot::new(cells, "black")));
    }
    let white_dots = [
        vec![(0, 1), (1, 1)],
        vec![(0, 3), (1, 3)],
        vec![(0, 7), (0, 8)],
        vec![(1, 0), (2, 0)],
        vec![(2, 1), (3, 1)],
        vec![(2, 3), (3, 3)],
        vec![(2, 7), (3, 7)],
        vec![(2, 8), (3, 8)],
        vec![(3, 5), (4, 5)],
        vec![(4, 0), (5, 0)],
        vec![(5, 2), (5, 3)],
        vec![(4, 6), (5, 6)],
        vec![(5, 8), (6, 8)],
        vec![(6, 1), (7, 1)],
        vec![(6, 3), (7, 3)],
        vec![(7, 4), (8, 4)],
        vec![(8, 0), (8, 1)],
        vec![(8, 2), (8, 3)],
        vec![(7, 7), (8, 7)],
    ];
    for cells in white_dots {
        grid.add_variant(SudokuVariant::Kropki(KropkiDot::new(cells, "white")));
    }
    grid.set_cell(0, 0, 5);
    grid.set_cell(1, 4, 9);
    grid.set_cell(1, 6, 6);
    grid.set_cell(1, 7, 7);
    grid.set_cell(2, 4, 5);
    grid.set_cell(2, 5, 1);
    grid.set_cell(2, 8, 8);
    grid.set_cell(4, 6, 7);
    grid.set_cell(4, 7, 5);
    grid.set_cell(5, 0, 7);
    grid.set_cell(5, 2, 4);
    grid.set_cell(5, 4, 1);
    grid.set_cell(5, 5, 3);
    grid.set_cell(6, 6, 1);
    grid.set_cell(7, 5, 9);
    grid.set_cell(8, 2, 7);
    grid.set_cell(8, 5, 4);
    grid.set_cell(8, 6, 5);
    grid.set_cell(8, 7, 9);
    grid.set_cell(8, 8, 3);

    if do_solve {
        run_solve(&mut grid, true, false);
    }

    grid
}

fn draft_day(do_solve: bool) -> SudokuGrid {
    /*
    Solution:
    892675314
    157943268
    436182597
    621457983
    578391642
    349826751
    783564129
    915238476
    264719835
     */
    let mut grid = SudokuGrid::empty();
    grid.add_variant(SudokuVariant::Diagonal(Diagonal::new(true)));
    let killer_cages = [
        (vec![(0, 1), (0, 2)], 11),
        (vec![(1, 0), (2, 0)], 5),
        (vec![(0, 6), (0, 7), (1, 6)], 6),
        (vec![(1, 8), (2, 7), (2, 8)], 24),
        (vec![(4, 3), (5, 3), (5, 4)], 13),
        (vec![(6, 0), (6, 1), (7, 0)], 24),
        (vec![(6, 7), (6, 8)], 11),
        (vec![(8, 7), (8, 8)], 8),
    ];
    for (cells, sum) in killer_cages {
        grid.add_variant(SudokuVariant::Killer(KillerCage::new(cells, sum)));
    }
    grid.add_variant(SudokuVariant::Thermometer(Thermometer::new(vec![
        (8, 4),
        (7, 3),
        (6, 2),
        (5, 1),
        (4, 0),
        (3, 0),
    ])));
    grid.add_variant(SudokuVariant::Thermometer(Thermometer::new(vec![
        (6, 7),
        (5, 7),
        (4, 6),
        (3, 5),
        (2, 4),
        (1, 3),
    ])));

    if do_solve {
        run_solve(&mut grid, true, false);
    }

    grid
}

fn ultraviolet(do_solve: bool) {
    let filename = "ultraviolet.txt";
    let mut path = PathBuf::from(get_examples_path());
    path.push(filename);

    let mut grid = SudokuGrid::read_from_file(&path).unwrap();

    if do_solve {
        run_solve(&mut grid, true, false);
    }
}

fn triumvirate(do_solve: bool) {
    let filename = "triumvirate.txt";
    let mut path = PathBuf::from(get_examples_path());
    path.push(filename);

    let mut grid = SudokuGrid::read_from_file(&path).unwrap();

    if do_solve {
        run_solve(&mut grid, true, true);
    } else {
        grid.display(true);
    }
}

fn run_solve(grid: &mut SudokuGrid, show_variants: bool, debug: bool) {
    println!("Sudoku Puzzle::::");
    grid.display(show_variants);

    let mut solver = Solver::new(grid);
    if solver.solve(debug) {
        println!("\n<<<<<<<<<<<<<<<<<Solved Sudoku Puzzle>>>>>>>>>>>>>>>>>>>>");
        grid.display(false);
    } else {
        println!("\nNo solution found for this Sudoku puzzle");
    }
}
