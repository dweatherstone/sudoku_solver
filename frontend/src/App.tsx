import React, { useState } from "react";
import "./App.css";

// Type for the Sudoku grid (9x9)
type SudokuGrid = number[][];

const GRID_SIZE = 9;

const App: React.FC = () => {
  // Initialize a 9x9 grid filled with zeros (empty)
  const [grid, setGrid] = useState<SudokuGrid>(
    Array.from({ length: GRID_SIZE }, () => Array(GRID_SIZE).fill(0))
  );

  // Handle a change in a single cell
  const handleChange = (row: number, col: number, value: string) => {
    const num = parseInt(value);
    if (isNaN(num) || num < 1 || num > 9) return; // only allow 1-9
    const newGrid = grid.map((r, i) =>
      r.map((cell, j) => (i === row && j === col ? num : cell))
    );
    setGrid(newGrid);
  };

  const handleSolve = () => {
    console.log("Current grid:", grid);
    alert("Solve button clicked! (Later: call backend API)");
  };

  return (
    <div style={{ padding: "1rem" }}>
      <h1>Sudoku Solver UI</h1>
      <div
        className="sudoku-grid"
        style={{
          display: "grid",
          gridTemplateColumns: `repeat(${GRID_SIZE}, 40px)`,
          gap: "0px",
        }}
      >
        {grid.map((row, rowIndex) =>
          row.map((cell, colIndex) => {
            // Determine border thickness for 3x3 subgrids
            const style: React.CSSProperties = {
              width: "40px",
              height: "40px",
              textAlign: "center",
              fontSize: "1.2rem",
              borderTop:
                rowIndex % 3 === 0 ? "2px solid black" : "1px solid #999",
              borderLeft:
                colIndex % 3 === 0 ? "2px solid black" : "1px solid #999",
              borderRight:
                colIndex === GRID_SIZE - 1
                  ? "2px solid black"
                  : "1px solid #999",
              borderBottom:
                rowIndex === GRID_SIZE - 1
                  ? "2px solid black"
                  : "1px solid #999",
              backgroundColor: undefined, // leave default; focus CSS will handle highlight
            };
            return (
              <input
                key={`${rowIndex}-${colIndex}`}
                type="text"
                value={cell === 0 ? "" : cell}
                onChange={(e) =>
                  handleChange(rowIndex, colIndex, e.target.value)
                }
                style={style}
                maxLength={1}
              />
            );
          })
        )}
      </div>
      <button
        onClick={handleSolve}
        style={{
          marginTop: "1rem",
          padding: "0.5rem 1rem",
          fontSize: "1rem",
          borderRadius: "4px",
          cursor: "pointer",
        }}
      >
        Solve
      </button>
    </div>
  );
};

export default App;
