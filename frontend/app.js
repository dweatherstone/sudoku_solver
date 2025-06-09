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
      
      circle.style.left = `${rect.left - tableRect.left + 45}px`; // 40px is cell width
      circle.style.top = `${rect.top - tableRect.top + 45}px`; // 40px is cell height
      
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
      const color = variant.Kropki.colour.toLowerCase(); // Convert to lowercase to match CSS classes
      
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
      
      // Calculate center point between cells
      // If the dots are in the same row, position horizontally between columns
      // If the dots are in the same column, position vertically between rows
      let left, top;
      if (row1 === row2) {
        // Same row, position horizontally
        left = (rect1.right + rect2.left) / 2 - tableRect.left - 6; // -6 to center the 12px dot
        top = rect1.top - tableRect.top + 20; // 20 is half cell height
      } else {
        // Same column, position vertically
        left = rect1.left - tableRect.left + 20; // 20 is half cell width
        top = (rect1.bottom + rect2.top) / 2 - tableRect.top - 6; // -6 to center the 12px dot
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

fetchAndRenderSudoku();

document.getElementById('solve-btn').addEventListener('click', async() => {
  const solveBtn = document.getElementById('solve-btn');
  solveBtn.disabled = true;
  solveBtn.textContent = "Solving...";
  
  const table = document.querySelector("#sudoku-container table");
  if (!table) {
    alert("Sudoku grid not found!");
    return;
  }

  const rows = table.querySelectorAll("tr");
  const grid = Array.from(rows).map(tr => 
    Array.from(tr.querySelectorAll("td")).map(td =>
      td.textContent.trim() === "" ? 0 : parseInt(td.textContent)
    )
  );

  // Get variants from the current state
  const variants = [];

  // Get Killer cages
  table.querySelectorAll('.killer-cage').forEach(cage => {
    const cells = [];
    const sum = parseInt(cage.querySelector('.killer-sum').textContent);
    
    // Get all cells within the cage's boundaries
    const rect = cage.getBoundingClientRect();
    const tableRect = table.getBoundingClientRect();
    const startRow = Math.floor((rect.top - tableRect.top) / 40);
    const startCol = Math.floor((rect.left - tableRect.left) / 40);
    const width = Math.floor(rect.width / 40);
    const height = Math.floor(rect.height / 40);
    
    for (let row = startRow; row < startRow + height; row++) {
      for (let col = startCol; col < startCol + width; col++) {
        cells.push([row, col]);
      }
    }
    
    variants.push({
      Killer: {
        cells: cells,
        sum: sum
      }
    });
  });

  // Get Kropki dots
  table.querySelectorAll('.kropki-dot').forEach(dot => {
    const rect = dot.getBoundingClientRect();
    const tableRect = table.getBoundingClientRect();
    const dotLeft = rect.left - tableRect.left;
    const dotTop = rect.top - tableRect.top;
    
    // Find the two cells this dot connects
    const cells = [];
    for (let row = 0; row < 9; row++) {
      for (let col = 0; col < 9; col++) {
        const cell = table.rows[row].cells[col];
        const cellRect = cell.getBoundingClientRect();
        const cellLeft = cellRect.left - tableRect.left;
        const cellTop = cellRect.top - tableRect.top;
        
        // Check if this cell is adjacent to the dot
        if (Math.abs(cellLeft + 20 - dotLeft) < 10 && Math.abs(cellTop + 20 - dotTop) < 10) {
          cells.push([row, col]);
        }
      }
    }
    
    if (cells.length === 2) {
      variants.push({
        Kropki: {
          cells: cells,
          colour: dot.classList.contains('black') ? 'Black' : 'White'
        }
      });
    }
  });

  // Get Quadruple circles
  const quadrupleCells = new Set();
  table.querySelectorAll('.quadruple-circle').forEach(td => {
    const rect = td.getBoundingClientRect();
    const tableRect = table.getBoundingClientRect();
    const row = Math.floor((rect.top - tableRect.top - 45) / 40);
    const col = Math.floor((rect.left - tableRect.left - 45) / 40);
    quadrupleCells.add(`${row},${col}`);
  });

  // Group cells into 2x2 blocks
  const blocks = new Map();
  quadrupleCells.forEach(cell => {
    const [row, col] = cell.split(',').map(Number);
    const blockKey = `${Math.floor(row/2)*2},${Math.floor(col/2)*2}`;
    if (!blocks.has(blockKey)) {
      blocks.set(blockKey, []);
    }
    blocks.get(blockKey).push([row, col]);
  });

  // Create variant objects for complete 2x2 blocks
  blocks.forEach((cells, key) => {
    if (cells.length === 4) {
      const [row, col] = key.split(',').map(Number);
      const td = table.rows[row].cells[col];
      const numbers = td.getAttribute('data-numbers')?.split(' ').map(Number) || [];
      variants.push({
        QuadrupleCircles: {
          cells: cells,
          required: numbers
        }
      });
    }
  });

  console.log("Sending grid to backend:", { cells: grid, variants });

  try {
    const res = await fetch("http://localhost:3000/solve", {
      method: "POST",
      headers: {"Content-Type": "application/json"},
      body: JSON.stringify({ cells: grid, variants }),
    });

    if (!res.ok) {
      alert("No solution found!");
      return;
    }

    const solvedGrid = await res.json();
    renderSudoku(solvedGrid.cells, solvedGrid.variants || []);
  } catch (err) {
    console.error("Error solving Sudoku:", err);
    alert("Error solving Sudoku.");
  } finally {
    solveBtn.disabled = false;
    solveBtn.textContent = "Solve";
  }
});
