# Rust File Organizer CLI

A beginner-friendly command-line tool written in **Rust** that organizes files into category folders (Images, Documents, Videos, etc).  
This project is part of the **Moringa School GenAI Capstone**.

## ✅ Features
- Detects file types by extension
- Creates category folders automatically
- Copies files safely into folders (originals stay)
- Supports `--dry-run` mode
- Fast, safe & beginner-friendly

## 🛠 Requirements
- Rust & Cargo installed
- WSL (Ubuntu) on Windows

## 🚀 Run the tool

Dry run:

```bash
cargo run -- /mnt/c/Users/DELL/Downloads --dry-run
