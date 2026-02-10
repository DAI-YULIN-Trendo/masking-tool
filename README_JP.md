# 🛡️ PDF Secure Masker (セキュア墨消しツール)

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-FFC107?style=for-the-badge&logo=tauri&logoColor=black)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-20232A?style=for-the-badge&logo=react&logoColor=61DAFB)](https://reactjs.org/)

**PDF Secure Masker** は、機密文書のセキュリティを守るためのプロフェッショナルなオープンソースツールです。単なる「上書き」ではなく、PDFの内部構造から指定データを**物理的に削除**します。

[English (README.md)](README.md) | [中文版 (README_CN.md)](README_CN.md)

---

## 📥 インストール / Installation

リリースページから最新バージョンをダウンロードしてください：
[**ダウンロード (.exe / .dmg)**](https://github.com/DAI-YULIN-Trendo/masking-tool/releases)

---

## ✨ 主な機能

- **物理的なデータ削除**: 墨消し範囲のテキストや画像データをバイナリレベルで破棄します。
- **多層構造 (N-layer) 対応**: 複雑な Form XObject の中身も再帰的に解析して削除します。
- **100% オフライン**: クラウドには一切送信しません。企業の機密保持基準に適合します。
- **高速なプレビュー**: Rust と WebAssembly による軽快な動作を実現。

---

## 🛠️ ローカルでの起動・テスト方法 (開発者向け)

開発環境を構築し、ローカルで実行する手順は以下の通りです。

### 1. 準備するもの
- **Node.js**: v18以上
- **Rust**: 最新の安定版 (rustup でインストール)
- **wasm-pack**: WebAssembly ビルド用 ([インストールはこちら](https://rustwasm.github.io/wasm-pack/installer/))
- **パッケージマネージャー**: npm

### 2. セットアップと実行
```bash
# プロジェクトフォルダへ移動
cd "masking tool"

# web フォルダの依存関係をインストール
cd web
npm install
cd ..

# WebAssembly モジュールのビルド（初回および core/wasm 変更用）
wasm-pack build wasm --target web

# 開発モードでアプリを起動
cd web
npm run tauri dev
```

### 3. コアロジックのテスト
PDF 墨消しのロジック（Rust部分）のみをテストする場合：
```bash
cd core
cargo test
```

---

## 📖 ユーザー操作マニュアル

一般ユーザー向けの操作方法は以下の通りです。

### ステップ 1：PDF の読み込み
画面中央へファイルをドラッグ＆ドロップするか、クリックして選択します。

### ステップ 2：墨消し範囲の指定
- **作成**: マウスで範囲をドラッグします。
- **移動**: 作成された枠をドラッグして位置を調整します。
- **削除**: 選択した状態で `Delete` キーを押します。

### ステップ 3：保存
右上の **「墨消し実行 (PDF保存)」** をクリックすると、処理済みの新しい PDF が生成されます。

---

## 🔒 セキュリティとプライバシー (SEO)

本プロジェクトは `プライバシーバイデザイン` に基づいて構築されています。
キーワード: `PDF 墨消し`, `マイナンバー 隠す`, `機密文書 マスキング`, `個人情報保護`, `Tauri PDF Editor`, `Secure Redaction`.

---

*© 2026 TRENDO - Secure Coding for Future.*
