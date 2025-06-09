// const puzzle = [
//   [0, 0, 9, 0, 0, 0, 0, 0, 4],
//   [0, 2, 4, 0, 9, 0, 0, 0, 0],
//   [0, 0, 0, 4, 0, 0, 3, 9, 2],
//   [1, 7, 2, 6, 0, 8, 9, 0, 3],
//   [4, 5, 3, 9, 7, 1, 0, 0, 8],
//   [0, 9, 0, 2, 0, 3, 7, 0, 0],
//   [0, 0, 0, 7, 0, 0, 5, 0, 9],
//   [0, 3, 0, 0, 8, 0, 0, 0, 0],
//   [0, 0, 1, 0, 0, 0, 0, 0, 6],
// ];

function renderSudoku(grid, variants = []) {
  const container = document.getElementById("sudoku-container");
  container.innerHTML = ""; // Clear previous content

  const table = document.createElement("table");

  for (let row = 0; row < 9; row++) {
    const tr = document.createElement("tr");
    for (let col = 0; col < 9; col++) {
      const td = document.createElement("td");
      const value = grid[row][col];
      td.textContent = value === 0 ? "" : value;
      
      // Make cells clickable for input
      td.addEventListener('click', () => handleCellClick(row, col));

      // Add bold borders for 3x3 block edges
      if ((col + 1) % 3 === 0 && col !== 8) td.classList.add("block-right");
      if ((row + 1) % 3 === 0 && row !== 8) td.classList.add("block-bottom");

      tr.appendChild(td);
    }
    table.appendChild(tr);
  }

  container.appendChild(table);

  // Add variants after the table is created
  variants.forEach(variant => {
    if (variant.QuadrupleCircles) {
      const cells = variant.QuadrupleCircles.cells;
      // Calculate the center position between the 4 cells
      const minRow = Math.min(...cells.map(([r]) => r));
      const maxRow = Math.max(...cells.map(([r]) => r));
      const minCol = Math.min(...cells.map(([, c]) => c));
      const maxCol = Math.max(...cells.map(([, c]) => c));
      
      // Create the circle element
      const circle = document.createElement("div");
      circle.className = "quadruple-circle";
      circle.setAttribute('data-numbers', variant.QuadrupleCircles.required.join(' '));
      
      // Position the circle at the center of the 2x2 block
      const topCell = table.rows[minRow].cells[minCol];
      const rect = topCell.getBoundingClientRect();
      const tableRect = table.getBoundingClientRect();
      
      circle.style.left = `${rect.left - tableRect.left + 45}px`;
      circle.style.top = `${rect.top - tableRect.top + 45}px`;
      
      table.appendChild(circle);
    } else if (variant.Killer) {
      const cells = variant.Killer.cells;
      const sum = variant.Killer.sum;
      
      // Calculate the cage boundaries
      const minRow = Math.min(...cells.map(([r]) => r));
      const maxRow = Math.max(...cells.map(([r]) => r));
      const minCol = Math.min(...cells.map(([, c]) => c));
      const maxCol = Math.max(...cells.map(([, c]) => c));
      
      // Create the cage element
      const cage = document.createElement("div");
      cage.className = "killer-cage";
      
      // Position the cage
      const topCell = table.rows[minRow].cells[minCol];
      const rect = topCell.getBoundingClientRect();
      const tableRect = table.getBoundingClientRect();
      
      cage.style.left = `${rect.left - tableRect.left}px`;
      cage.style.top = `${rect.top - tableRect.top}px`;
      cage.style.width = `${(maxCol - minCol + 1) * 40}px`;
      cage.style.height = `${(maxRow - minRow + 1) * 40}px`;
      
      // Add the sum
      const sumElement = document.createElement("div");
      sumElement.className = "killer-sum";
      sumElement.textContent = sum;
      topCell.appendChild(sumElement);
      
      table.appendChild(cage);
    } else if (variant.Kropki) {
      const cells = variant.Kropki.cells;
      const color = variant.Kropki.colour.toLowerCase();
      
      // Calculate the position between the two cells
      const [row1, col1] = cells[0];
      const [row2, col2] = cells[1];
      
      // Create the dot element
      const dot = document.createElement("div");
      dot.className = `kropki-dot ${color}`;
      
      // Position the dot between the cells
      const cell1 = table.rows[row1].cells[col1];
      const cell2 = table.rows[row2].cells[col2];
      const rect1 = cell1.getBoundingClientRect();
      const rect2 = cell2.getBoundingClientRect();
      const tableRect = table.getBoundingClientRect();
      
      let left, top;
      if (row1 === row2) {
        left = (rect1.right + rect2.left) / 2 - tableRect.left - 6;
        top = rect1.top - tableRect.top + 20;
      } else {
        left = rect1.left - tableRect.left + 20;
        top = (rect1.bottom + rect2.top) / 2 - tableRect.top - 6;
      }
      
      dot.style.left = `${left}px`;
      dot.style.top = `${top}px`;
      
      table.appendChild(dot);
    }
  });
}

async function fetchAndRenderSudoku() {
  try {
    const res = await fetch("http://localhost:3000/sudoku");
    const data = await res.json();
    renderSudoku(data.cells, data.variants || []);
  } catch (err) {
    console.error("Failed to fetch or render Sudoku:", err);
  }
}

async function handleCellClick(row, col) {
  const value = prompt("Enter a number (1-9) or 0 to clear:");
  if (value === null) return; // User cancelled
  
  const numValue = parseInt(value);
  if (isNaN(numValue) || numValue < 0 || numValue > 9) {
    alert("Please enter a number between 0 and 9");
    return;
  }

  try {
    const res = await fetch(`http://localhost:3000/cell/${row}/${col}/${numValue}`, {
      method: "POST"
    });
    
    if (!res.ok) {
      alert("Invalid move!");
      return;
    }
    
    const data = await res.json();
    renderSudoku(data.cells, data.variants || []);
  } catch (err) {
    console.error("Failed to set cell value:", err);
    alert("Error setting cell value");
  }
}

document.getElementById('solve-btn').addEventListener('click', async() => {
  const solveBtn = document.getElementById('solve-btn');
  solveBtn.disabled = true;
  solveBtn.textContent = "Solving...";
  
  try {
    const res = await fetch("http://localhost:3000/solve", {
      method: "POST"
    });

    if (!res.ok) {
      alert("No solution found!");
      return;
    }

    const data = await res.json();
    renderSudoku(data.cells, data.variants || []);
  } catch (err) {
    console.error("Error solving Sudoku:", err);
    alert("Error solving Sudoku.");
  } finally {
    solveBtn.disabled = false;
    solveBtn.textContent = "Solve";
  }
});

// Initial load
fetchAndRenderSudoku();
