import init, { Emulator } from "./gameboy/gameboy.js";

let emulator;
let wasm; // Declare wasm globally
const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");
const imageData = ctx.createImageData(160, 144);
let isRunning = false;

/**
 * Initialize the WebAssembly module
 */
async function initializeWasm() {
  wasm = await init(); // Initialize and assign the wasm instance
  console.log("WASM initialized successfully");
}

/**
 * Load and initialize the emulator with a ROM file
 * @param {File} file - The ROM file
 */
async function loadRomFile(file) {
  const arrayBuffer = await file.arrayBuffer();
  const romData = new Uint8Array(arrayBuffer);

  if (emulator) {
    emulator.load_rom(romData);
  } else {
    emulator = new Emulator(romData);
  }

  console.log("ROM loaded successfully");
  drawFrame(); // Draw the first frame after loading the ROM

  // Start the game loop after loading the ROM
  if (!isRunning) {
    isRunning = true;
    gameLoop();
  }
}

/**
 * Draws the current frame from the emulator onto the canvas
 */
function drawFrame() {
  if (!emulator || !wasm) {
    console.warn("WASM or emulator is not initialized.");
    return;
  }

  const wasmMemory = new Uint8Array(wasm.memory.buffer);
  const frameBufferPtr = emulator.get_frame_buffer();
  const length = emulator.get_frame_buffer_length();

  const frameBuffer = wasmMemory.subarray(
    frameBufferPtr,
    frameBufferPtr + length
  );

  for (let i = 0; i < frameBuffer.length; i++) {
    const color = frameBuffer[i];
    imageData.data[i * 4] = color;
    imageData.data[i * 4 + 1] = color;
    imageData.data[i * 4 + 2] = color;
    imageData.data[i * 4 + 3] = 255;
  }

  ctx.putImageData(imageData, 0, 0);
}

/**
 * The game loop that continuously steps through the emulator
 */
function gameLoop() {
  if (emulator) {
    try {
      emulator.step();
      drawFrame();
    } catch (e) {
      console.error("Emulator step failed:", e);
      isRunning = false; // Stop the game loop on error
    }
  }

  if (isRunning) {
    requestAnimationFrame(gameLoop);
  }
}

/**
 * Event listener for loading a ROM file
 */
document
  .getElementById("romInput")
  .addEventListener("change", async (event) => {
    const file = event.target.files[0];
    if (file) {
      await loadRomFile(file);
    }
  });

// Initialize WebAssembly on page load
initializeWasm();
