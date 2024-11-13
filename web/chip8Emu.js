import init, {
  load_rom,
  cycle,
  get_display_buffer,
  key_down,
  key_up,
} from "./chip8/chip8.js";

let wasmMemory;
let displayBuffer;
let animationFrame;
const scale = 10;
const width = 64;
const height = 32;

const canvas = document.getElementById("screen");
const ctx = canvas.getContext("2d");

export async function initChip8() {
  const wasm = await init();
  wasmMemory = wasm.memory;

  displayBuffer = new Uint8Array(
    wasmMemory.buffer,
    get_display_buffer(),
    width * height
  );
}

export function loadROM(rom) {
  load_rom(rom);
}

export function startChip8() {
  function emulate() {
    cycle();
    renderDisplay();
    animationFrame = requestAnimationFrame(emulate);
  }
  emulate();
}

export function keyDown(key) {
  console.log("Key down:", key);
  key_down(key);
}

export function keyUp(key) {
  console.log("Key up:", key);
  key_up(key);
}

function renderDisplay() {
  // Refresh the display buffer in case the memory was resized
  displayBuffer = new Uint8Array(
    wasmMemory.buffer,
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
