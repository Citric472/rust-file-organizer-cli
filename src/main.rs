use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Returns lowercase extension string for a path, e.g. "jpg" or "" if none.
fn file_extension_lowercase(path: &Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "".to_string())
}

/// Build a mapping of category -> Vec<extensions>
fn build_category_map() -> HashMap<&'static str, Vec<&'static str>> {
    let mut m = HashMap::new();
    m.insert("Images", vec!["jpg", "jpeg", "png", "gif", "svg", "bmp", "webp"]);
    m.insert("Documents", vec!["pdf", "doc", "docx", "txt", "xls", "xlsx", "ppt", "pptx"]);
    m.insert("Videos", vec!["mp4", "mov", "mkv", "webm", "avi"]);
    m.insert("Audio", vec!["mp3", "wav", "flac", "aac"]);
    m.insert("Archives", vec!["zip", "rar", "tar", "gz", "7z"]);
    m.insert("Code", vec!["rs", "py", "js", "ts", "go", "java", "c", "cpp", "html", "css", "json", "yaml", "yml"]);
    m
}

/// Given an extension, find category name, or "Others"
fn category_for_extension<'a>(ext: &str, categories: &'a HashMap<&str, Vec<&str>>) -> &'a str {
    for (cat, exts) in categories {
        if exts.iter().any(|e| e == &ext) {
            return cat;
        }
    }
    "Others"
}

fn copy_file_to_category(src: &Path, dest_dir: &Path) -> io::Result<PathBuf> {
    // Ensure destination directory exists
    fs::create_dir_all(dest_dir)?;
    // Build destination file path
    let file_name = src.file_name().expect("file should have a name");
    let mut dest_path = dest_dir.join(file_name);

    // If a file with the same name already exists in destination, append a counter
    if dest_path.exists() {
        let mut count = 1;
        let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let ext = src.extension().and_then(|e| e.to_str()).map(|s| format!(".{}", s)).unwrap_or_else(|| "".to_string());
        loop {
            let new_name = format!("{}_{}{}", stem, count, ext);
            dest_path = dest_dir.join(new_name);
            if !dest_path.exists() {
                break;
            }
            count += 1;
        }
    }

    fs::copy(src, &dest_path)?;
    Ok(dest_path)
}

fn print_usage_and_exit(program: &str) {
    println!("Usage:");
    println!("  {} <folder-path> [--dry-run]", program);
    println!();
    println!("Examples:");
    println!("  cargo run -- /mnt/c/Users/DELL/Downloads");
    println!("  cargo run -- /mnt/c/Users/DELL/Downloads --dry-run");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args.get(0).map(|s| s.as_str()).unwrap_or("file_organizer");

    if args.len() < 2 {
        print_usage_and_exit(program);
    }

    let folder_path = &args[1];
    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n");

    // Resolve canonical path (handles symlinks)
    let path = Path::new(folder_path);
    let canonical = match fs::canonicalize(path) {
        Ok(p) => p,
        Err(_) => {
            eprintln!("❌ Error: '{}' is not a valid directory!", folder_path);
            std::process::exit(1);
        }
    };

    if !canonical.is_dir() {
        eprintln!("❌ Error: '{}' is not a directory!", canonical.display());
        std::process::exit(1);
    }

    println!("📁 Organizing folder: {}", canonical.display());
    if dry_run {
        println!("🔎 Running in DRY-RUN mode (no files will be copied).");
    } else {
        println!("⚠️ Safe Mode: files will be COPIED (originals left intact).");
    }

    let categories = build_category_map();

    // Counters for summary
    let mut counters: HashMap<String, usize> = HashMap::new();
    counters.insert("Images".to_string(), 0);
    counters.insert("Documents".to_string(), 0);
    counters.insert("Videos".to_string(), 0);
    counters.insert("Audio".to_string(), 0);
    counters.insert("Archives".to_string(), 0);
    counters.insert("Code".to_string(), 0);
    counters.insert("Others".to_string(), 0);
    counters.insert("Errors".to_string(), 0);

    // Iterate entries in the directory (non-recursive)
    let read_dir = match fs::read_dir(&canonical) {
        Ok(rd) => rd,
        Err(e) => {
            eprintln!("❌ Failed to read directory: {}", e);
            std::process::exit(1);
        }
    };

    for entry in read_dir {
        match entry {
            Ok(dir_entry) => {
                let file_type = match dir_entry.file_type() {
                    Ok(ft) => ft,
                    Err(e) => {
                        eprintln!("⚠️ Could not read file type: {}", e);
                        *counters.get_mut("Errors").unwrap() += 1;
                        continue;
                    }
                };

                // Skip directories; we only process regular files
                if file_type.is_dir() {
                    continue;
                }
                if file_type.is_symlink() {
                    // skip symlinks for safety
                    continue;
                }

                let path = dir_entry.path();
                let ext = file_extension_lowercase(&path);
                let category = category_for_extension(&ext, &categories);
                let dest_dir = canonical.join(category);

                if dry_run {
                    println!("➡️ Would copy: '{}' -> '{}'", path.display(), dest_dir.display());
                    *counters.get_mut(category).unwrap() += 1;
                    continue;
                }

                match copy_file_to_category(&path, &dest_dir) {
                    Ok(dest_path) => {
                        println!("✅ Copied: '{}' -> '{}'", path.display(), dest_path.display());
                        *counters.get_mut(category).unwrap() += 1;
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to copy '{}': {}", path.display(), e);
                        *counters.get_mut("Errors").unwrap() += 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️ Failed to read an entry: {}", e);
                *counters.get_mut("Errors").unwrap() += 1;
            }
        }
    }

    // Summary
    println!("\n📊 Summary:");
    for key in &["Images", "Documents", "Videos", "Audio", "Archives", "Code", "Others", "Errors"] {
        let count = counters.get(*key).cloned().unwrap_or_default();
        println!("  - {:<9} : {}", key, count);
    }

    println!("\n🎉 Done! (Safe Mode copy completed.)");
}
