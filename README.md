# 🔍 Recall

> **Instantly search and recall your shell history with style**

Recall is a fast, beautiful command-line tool written in Rust that helps you search through your zsh history. Never forget that complex command you ran last week again!

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Version](https://img.shields.io/badge/Version-0.1.4-green?style=for-the-badge)

## ✨ Features

- 🎯 **Smart Filtering** — Search with multiple keywords using AND logic (all keywords must match)
- 🕐 **Timestamped Results** — See exactly when each command was executed
- 📊 **Frequency Analysis** — Find your most-used commands with `recall freq`
- 🎨 **Syntax Highlighting** — Keywords are highlighted in the output for easy scanning
- 🧹 **Deduplicated Results** — Shows only unique commands, keeping the most recent occurrence
- ⚡ **Blazingly Fast** — Built with Rust for maximum performance

## 📦 Installation

### From Source

Make sure you have [Rust](https://rustup.rs/) installed, then:

```bash
git clone https://github.com/YOUR_USERNAME/recall.git
cd recall
cargo install --path .
```

### From Cargo (coming soon)

```bash
cargo install recall
```

## 🚀 Usage

### Basic Search

Search your history for commands containing specific keywords:

```bash
# Find all commands containing "docker"
recall docker

# Find commands containing BOTH "docker" AND "compose" (AND logic)
recall docker compose

# Find commands with multiple keywords
recall git push origin
```

### View Recent Commands

Run without arguments to see your recent command history:

```bash
recall
```

### Frequency Analysis

See your most frequently used commands:

```bash
# Show top 10 most frequent commands
recall freq

# Show most frequent commands containing a keyword
recall freq docker
```

## 📸 Example Output

```
--- RECENT COMMANDS ---
2026-02-06 15:57:08 | docker compose up -d
2026-02-06 16:23:41 | docker compose logs -f api
2026-02-06 17:45:22 | docker compose down
```

```
--- MOST FREQUENT COMMANDS ---
42    | git status
38    | cargo build --release
27    | docker compose up -d
15    | npm run dev
```

## 🛠️ Requirements

- **zsh** shell with history enabled
- History file at `~/.zsh_history` (default zsh history location)
- macOS or Linux

## 🙏 Acknowledgments

Built with these awesome Rust crates:
- [clap](https://crates.io/crates/clap) — Command line argument parsing
- [chrono](https://crates.io/crates/chrono) — Date and time handling
- [colored](https://crates.io/crates/colored) — Terminal colors
- [regex](https://crates.io/crates/regex) — Regular expressions

---

<p align="center">
  Made with ❤️ and 🦀
</p>
