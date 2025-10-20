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

function renderKillerCages(cellMatrix, variants) {
  variants.forEach((variant, idx) => {
    if (variant.Killer) {
      console.log("Rendering killer cage", idx, variant.Killer);
      const cells = variant.Killer.cells;
      const sum = variant.Killer.sum;
      const cellSet = new Set(cells.map(([r, c]) => `${r},${c}`));
      cells.forEach(([r, c]) => {
        const td = cellMatrix[r][c];
        td.style.position = "relative"; // Ensure the cell is a positioning context

        // Helper to create a border div"
        function addInsetBorder(className, style) {
          const div = document.createElement("div");
          div.className = className;
          Object.assign(div.style, style);
          td.appendChild(div);
        }

        // Inset in pixels
        const inset = 2;

        if (!cellSet.has(`${r - 1},${c}`)) {
          addInsetBorder("killer-border-top", {
            position: "absolute",
            top: `${inset}px`,
            left: `${inset}px`,
            width: `calc(100% - ${2 * inset}px)`,
            height: "0",
            borderTop: "2px dashed #000",
            pointerEvents: "none",
            zIndex: 2,
          });
        }
        if (!cellSet.has(`${r + 1},${c}`)) {
          addInsetBorder("killer-border-bottom", {
            position: "absolute",
            bottom: `${inset}px`,
            left: `${inset}px`,
            width: `calc(100% - ${2 * inset}px)`,
            height: "0",
            borderBottom: "2px dashed #000",
            pointerEvents: "none",
            zIndex: 2,
          });
        }
        if (!cellSet.has(`${r},${c - 1}`)) {
          addInsetBorder("killer-border-left", {
            position: "absolute",
            top: `${inset}px`,
            left: `${inset}px`,
            height: `calc(100% - ${2 * inset}px)`,
            width: "0",
            borderLeft: "2px dashed #000",
            pointerEvents: "none",
            zIndex: 2,
          });
        }
        if (!cellSet.has(`${r},${c + 1}`)) {
          addInsetBorder("killer-border-right", {
            position: "absolute",
            top: `${inset}px`,
            right: `${inset}px`,
            height: `calc(100% - ${2 * inset}px)`,
            width: "0",
            borderRight: "2px dashed #000",
            pointerEvents: "none",
            zIndex: 2,
          });
        }
      });

      // Add  the sum to the top-left cell
      const [minRow, minCol] = cells.reduce(
        ([mr, mc], [r, c]) => {
          if (r < mr || (r === mr && c < mc)) return [r, c];
          return [mr, mc];
        },
        [cells[0][0], cells[0][1]]
      );
      const sumElement = document.createElement("div");
      sumElement.className = "killer-sum";
      sumElement.textContent = sum;
      cellMatrix[minRow][minCol].appendChild(sumElement);
    }
  });
}

function renderThermometers(cellMatrix, variants, table) {
  const thermometerVariants = variants.filter((v) => v.Thermometer);
  if (thermometerVariants.length > 0) {
    console.log("Rendering thermometers", thermometerVariants);
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("class", "thermo-svg");
    svg.style.position = "absolute";
    svg.style.left = "0";
    svg.style.top = "0";
    svg.style.pointerEvents = "none";
    svg.style.zIndex = "2";
    // Wait for layout
    setTimeout(() => {
      svg.innerHTML = "";
      svg.setAttribute("width", table.offsetWidth);
      svg.setAttribute("height", table.offsetHeight);
      thermometerVariants.forEach((variant) => {
        const cells = variant.Thermometer.cells;
        const points = cells.map(([r, c]) => {
          const td = cellMatrix[r][c];
          const rect = td.getBoundingClientRect();
          const tableRect = table.getBoundingClientRect();
          // Calculate position relative to the table's position
          return {
            x: rect.left - tableRect.left + rect.width / 2,
            y: rect.top - tableRect.top + rect.height / 2,
          };
        });
        // Draw the bulb
        const bulb = document.createElementNS(
          "http://www.w3.org/2000/svg",
          "circle"
        );
        bulb.setAttribute("cx", points[0].x);
        bulb.setAttribute("cy", points[0].y);
        bulb.setAttribute("r", "13");
        bulb.setAttribute("class", "thermo-bulb");
        svg.appendChild(bulb);
        // Draw the path
        for (let i = 0; i < points.length - 1; i++) {
          const line = document.createElementNS(
            "http://www.w3.org/2000/svg",
            "line"
          );
          line.setAttribute("x1", points[i].x);
          line.setAttribute("y1", points[i].y);
          line.setAttribute("x2", points[i + 1].x);
          line.setAttribute("y2", points[i + 1].y);
          line.setAttribute("class", "thermo-line");
          svg.appendChild(line);
        }
      });
      // Position the SVG relative to the table
      const tableRect = table.getBoundingClientRect();
      const containerRect = table.parentElement.getBoundingClientRect();
      svg.style.left = `${tableRect.left - containerRect.left}px`;
      svg.style.top = `${tableRect.top - containerRect.top}px`;
      table.parentElement.appendChild(svg);
    }, 0);
  }
}

function renderSudoku(grid, variants = []) {
  const container = document.getElementById("sudoku-container");
  container.innerHTML = ""; // Clear previous content

  const table = document.createElement("table");
  const cellMatrix = [];
  for (let row = 0; row < 9; row++) {
    const tr = document.createElement("tr");
    cellMatrix[row] = [];
    for (let col = 0; col < 9; col++) {
      const td = document.createElement("td");
      const value = grid[row][col];
      td.textContent = value === 0 ? "" : value;
      td.addEventListener("click", () => handleCellClick(row, col));
      if ((col + 1) % 3 === 0 && col !== 8) td.classList.add("block-right");
      if ((row + 1) % 3 === 0 && row !== 8) td.classList.add("block-bottom");
      tr.appendChild(td);
      cellMatrix[row][col] = td;
    }
    table.appendChild(tr);
  }
  container.appendChild(table);

  // Render killer cages
  renderKillerCages(cellMatrix, variants);
  // Render thermometers
  renderThermometers(cellMatrix, variants, table);

  // --- Other Variants ---
  variants.forEach((variant) => {
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
      circle.setAttribute(
        "data-numbers",
        variant.QuadrupleCircles.required.join(" ")
      );

      // Position the circle at the center of the 2x2 block
      const topCell = table.rows[minRow].cells[minCol];
      const rect = topCell.getBoundingClientRect();
      const tableRect = table.getBoundingClientRect();

      circle.style.left = `${rect.left - tableRect.left + 45}px`;
      circle.style.top = `${rect.top - tableRect.top + 45}px`;

      table.appendChild(circle);
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
    const res = await fetch(
      `http://localhost:3000/cell/${row}/${col}/${numValue}`,
      {
        method: "POST",
      }
    );

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

document.getElementById("solve-btn").addEventListener("click", async () => {
  const solveBtn = document.getElementById("solve-btn");
  solveBtn.disabled = true;
  solveBtn.textContent = "Solving...";

  try {
    const res = await fetch("http://localhost:3000/solve", {
      method: "POST",
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
