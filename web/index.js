let wasm_memory;
import init, {
  load_rom,
  cycle,
  get_display_buffer,
  key_down,
  key_up,
} from "./chip8/chip8.js";

const canvas = document.getElementById("screen");
const ctx = canvas.getContext("2d");
const fileInput = document.getElementById("file-input");
const startButton = document.getElementById("start-button");

const scale = 10;
canvas.width = 64 * scale;
canvas.height = 32 * scale;

let displayBuffer;
let animationFrame;
let romLoaded = false;

// Initialize the emulator
async function initEmulator() {
  // Initialize the WASM module and store the memory reference
  const wasm = await init();
  wasm_memory = wasm.memory;

  // Prepare the display buffer
  displayBuffer = new Uint8Array(
    wasm_memory.buffer,
    get_display_buffer(),
    64 * 32
  );

  // Event listener for loading a ROM file
  fileInput.addEventListener("change", (event) => {
    const file = event.target.files[0];
    const reader = new FileReader();

    reader.onload = function () {
      const rom = new Uint8Array(reader.result);
      load_rom(rom);
      console.log("ROM loaded:", rom);
      romLoaded = true;
      startButton.disabled = false; // Enable the start button
    };

    reader.readAsArrayBuffer(file);
  });

  // Event listener for starting the emulation
  startButton.addEventListener("click", () => {
    if (romLoaded) {
      startEmulation();
    }
  });

  // Handle key presses
  window.addEventListener("keydown", (event) => {
    const key = mapKey(event.key);
    if (key !== null) {
      console.log("Key down:", event.key, "Mapped to:", key);
      key_down(key);
    }
  });

  window.addEventListener("keyup", (event) => {
    const key = mapKey(event.key);
    if (key !== null) {
      console.log("Key up:", event.key, "Mapped to:", key);
      key_up(key);
    }
  });
}

// Start the emulation loop
function startEmulation() {
  function emulate() {
    cycle(); // Execute a single cycle
    renderDisplay(); // Update the display
    requestAnimationFrame(emulate); // Schedule the next frame
  }
  emulate();
}

function renderDisplay() {
  const width = 64;
  const height = 32;

  // Refresh the display buffer in case the memory was resized
  displayBuffer = new Uint8Array(
    wasm_memory.buffer,
    get_display_buffer(),
    width * height
  );

  ctx.clearRect(0, 0, canvas.width, canvas.height);

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const index = y * width + x;
      const pixel = displayBuffer[index];

      ctx.fillStyle = pixel ? "white" : "black";
      ctx.fillRect(x * scale, y * scale, scale, scale);
    }
  }
}
// Map keyboard keys to Chip-8 keys
function mapKey(key) {
  const keymap = {
    1: 0x1,
    2: 0x2,
    3: 0x3,
    4: 0xc,
    q: 0x4,
    w: 0x5,
    e: 0x6,
    r: 0xd,
    a: 0x7,
    s: 0x8,
    d: 0x9,
    f: 0xe,
    z: 0xa,
    x: 0x0,
    c: 0xb,
    v: 0xf,
  };
  return keymap[key] || null;
}

initEmulator();
