# 🛡️ PDF Secure Masker

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-FFC107?style=for-the-badge&logo=tauri&logoColor=black)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-20232A?style=for-the-badge&logo=react&logoColor=61DAFB)](https://reactjs.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)

**PDF Secure Masker** is a high-security, **100% offline** solution for redacting sensitive information from PDF files. It goes beyond visual masking by physically deleting the underlying data, ensuring compliance with strict enterprise privacy standards.

[日本語版 (Japanese)](README_JP.md) | [中文版 (Chinese)](README_CN.md)

---

## 📥 Installation

Download the latest version from the Releases page:
[**Download (.exe / .dmg)**](https://github.com/DAI-YULIN-Trendo/masking-tool/releases)

---

## 🌟 Key Features

- **100% Offline Processing**: Files are processed locally on your machine. No data is ever sent to external servers.
- **Deep Deletion**: Physically removes text, images, and **nested vectors inside Form XObjects** under the masked area.
- **Advanced PDF Engine**: Built with a high-performance, memory-safe Rust core.
- **Intuitive UI**: Simple drag-and-drop interface with real-time preview (React + Tauri).
- **Forensic-Proof**: Simply covering data with a black box is not enough. This tool destroys the binary data beneath it.

---

## 🏗️ Architecture

The project consists of three main modules:

- `core/`: Rust crate containing the core PDF processing and redaction logic.
- `wasm/`: WebAssembly wrapper for the core logic, enabling high-speed operations in the frontend.
- `web/`: User interface built with React, Vite, and Tauri.

---

## 🚀 Local Development Setup

Follow these steps to set up and run the project locally.

### 1. Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/) (stable)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) (for building the WebAssembly module)
- OS-specific dependencies for Tauri (macOS: Xcode, Windows: WebView2/C++ Build Tools)

### 2. Installation

```bash
# Clone the repository (or navigate to the folder)
cd "masking tool"

# Install frontend dependencies
cd web
npm install
cd ..

# Build the WebAssembly module (Required for first run or after changing core logic)
wasm-pack build wasm --target web
```

### 3. Run & Test

Launch the development application with hot-reloading.

```bash
# Start Tauri dev server
cd web
npm run tauri dev
```

To run only the Rust core tests:

```bash
cd core
cargo test
```

---

## 🔒 Security Policy (Keywords)

This tool is optimized for use cases requiring strict data protection:
`PDF Redaction`, `GDPR Compliance`, `CCPA`, `Sensitive Document Processing`, `Offline PDF Editor`, `Secure Data Deletion`, `Privacy First`.

---

## 📄 License

MIT License - See the [LICENSE](LICENSE) file for details.

---

## 🤝 Contributing

Bug reports and pull requests are welcome on GitHub.

---

*Developed by TRENDO with ❤️ for a more secure digital world.*
