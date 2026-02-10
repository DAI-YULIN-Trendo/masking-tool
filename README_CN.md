# 🛡️ PDF Secure Masker (安全墨消工具)

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-FFC107?style=for-the-badge&logo=tauri&logoColor=black)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-20232A?style=for-the-badge&logo=react&logoColor=61DAFB)](https://reactjs.org/)

**PDF Secure Masker** 是专为企业和专业人士设计的 PDF 隐私保护工具。当企业必须向外部提供含有敏感数据的文件时，本工具提供最安全的「彻底遮盖」方案。它不仅是视觉上的遮挡，更能从文件底层**物理删除**被遮盖的数据，杜绝任何复原可能。

秉持开源精神，我们也以此保障处理过程的绝对透明与安全性。

---

## 📥 下载安装 / Installation

请从发布页面下载最新版本：
[**下载 (.exe / .dmg)**](https://github.com/DAI-YULIN-Trendo/masking-tool/releases)

---

## ✨ 核心亮点

- **物理级数据销毁**: 彻底粉碎遮盖区域下方的文字、图像及矢量路径。就算是专业取证工具也无法恢复。
- **支持复杂嵌套结构**: 智能解析并清理深层嵌套的 Form XObject，确保无死角。
- **100% 离线运行**: 所有操作均在本地完成，绝无数据上传风险。完美符合企业合规要求。
- **开源透明**: 基于 Rust 构建的高性能核心，代码完全公开，安全可审计。

---

## 🛠️ 本地开发与运行 (开发者指南)

如果您希望自行构建或审查代码，请参考以下步骤：

### 1. 环境准备
- **Node.js**: v18+
- **Rust**: 最新稳定版 (通过 rustup 安装)
- **wasm-pack**: 用于构建 WebAssembly 模块 ([安装链接](https://rustwasm.github.io/wasm-pack/installer/))
- **包管理器**: npm

### 2. 构建与启动
```bash
# 进入项目目录
cd "masking tool"

# 安装前端依赖
cd web
npm install
cd ..

# 构建 WebAssembly 核心模块 (首次运行或核心代码变更时必须执行)
wasm-pack build wasm --target web

# 启动开发服务器
cd web
npm run tauri dev
```

### 3. 每个模块的测试
若只测试 Rust 核心逻辑：
```bash
cd core
cargo test
```

---

## 📖 用户操作指南

### 第一步：导入 PDF
点击界面中央区域选择文件，或直接将 PDF 拖拽至此。支持处理加密文件。

### 第二步：创建遮盖
- **创建**: 在工具栏选择「黑涂」或「白涂」，然后在页面上拖拽框选。
- **调整**: 使用「移动」工具，或点击已有的遮盖块进行位置微调。
- **常用操作**:
    - **全页应用**: 选中一个遮盖块，点击「应用到全页」，可快速遮盖每一页的相同位置（如页眉/页脚敏感信息）。
    - **删除**: 选中遮盖块后按 `Delete` 键或点击工具栏删除图标。

### 第三步：安全保存
点击右下角的 **「执行墨消 (保存 PDF)」** 按钮。系统将生成一个新的 `_masked.pdf` 文件。

---

## 🔒 安全声明与合规 (SEO)

本工具专为满足 GDPR、CCPA 及企业内部数据防泄露 (DLP) 需求而设计。
关键词: `PDF 墨消`, `敏感数据脱敏`, `企业级数据保护`, `离线 PDF 编辑器`, `Secure Redaction`, `Privacy First`.

---

*© 2026 TRENDO - Secure Coding for Future.*
