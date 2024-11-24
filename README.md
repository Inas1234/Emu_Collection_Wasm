
# Emu_Collection_Wasm

This is a collection of emulators compiled to WebAssembly (WASM), enabling them to run efficiently within web browsers. This project aims to provide accessible and performant emulation experiences directly in the browser environment.

## Features

- **Multi-Platform Emulation**: Supports various classic systems, allowing users to experience a range of retro games and applications.
- **WebAssembly Integration**: Utilizes WASM for near-native performance in web browsers.
- **Web Interface**: Includes a user-friendly web interface for loading and interacting with emulated systems.

## Getting Started

To run the emulators locally:

1. **Clone the Repository**:

   ```bash
   git clone https://github.com/Inas1234/Emu_Collection_Wasm.git
   ```

2. **Navigate to the Project Directory**:

   ```bash
   cd Emu_Collection_Wasm
   ```

3. **Build the Project**:

   Ensure you have [Rust](https://www.rust-lang.org/tools/install) and [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) installed. Then, run:

   ```bash
   make
   ```

4. **Serve the Web Interface**:

   You can use a simple HTTP server to serve the `web` directory. For example, with Python:

   ```bash
   cd web
   python3 -m http.server
   ```

5. **Access the Emulators**:

   Open your web browser and navigate to `http://localhost:8000` to start using the emulators.


---

*Note: This project is under active development. Features and documentation are subject to change.* 
