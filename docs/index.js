import {
  initChip8,
  loadROM as loadChip8ROM,
  startChip8,
  keyDown as chip8KeyDown,
  keyUp as chip8KeyUp,
} from "./chip8Emu.js";

const canvas = document.getElementById("screen");
const ctx = canvas.getContext("2d");
const fileInput = document.getElementById("file-input");
const startButton = document.getElementById("start-button");
const emulatorSelect = document.getElementById("emulator-select");

const scale = 10;
canvas.width = 64 * scale;
canvas.height = 32 * scale;

let currentEmulator = null;
let romLoaded = false;

// Emulator configurations
const emulators = {
  chip8: {
    init: initChip8,
    loadROM: loadChip8ROM,
    start: startChip8,
    keyDown: chip8KeyDown,
    keyUp: chip8KeyUp,
  },
  // Future emulators can be added here:
  // nes: {
  //   init: initNES,
  //   loadROM: loadNESROM,
  //   start: startNES,
  //   keyDown: nesKeyDown,
  //   keyUp: nesKeyUp,
  // },
};

// Function to initialize the selected emulator
async function initializeEmulator() {
  const selectedEmulator = emulatorSelect.value;
  const emulator = emulators[selectedEmulator];

  if (!emulator) return;

  await emulator.init();
  currentEmulator = emulator;

  fileInput.addEventListener("change", (event) => {
    const file = event.target.files[0];
    const reader = new FileReader();

    reader.onload = function () {
      const rom = new Uint8Array(reader.result);
      currentEmulator.loadROM(rom);
      console.log("ROM loaded:", rom);
      romLoaded = true;
      startButton.disabled = false;
    };

    reader.readAsArrayBuffer(file);
  });

  startButton.addEventListener("click", () => {
    if (romLoaded) {
      currentEmulator.start();
    }
  });

  window.addEventListener("keydown", (event) => {
    const key = mapKey(event.key);
    if (key !== null) {
      currentEmulator.keyDown(key);
    }
  });

  window.addEventListener("keyup", (event) => {
    const key = mapKey(event.key);
    if (key !== null) {
      currentEmulator.keyUp(key);
    }
  });
}

// Map keyboard keys to Chip-8 keys (adjust if needed for other emulators)
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

// Reinitialize the emulator when a different one is selected
emulatorSelect.addEventListener("change", initializeEmulator);

// Initialize the default emulator (Chip-8) on page load
initializeEmulator();
