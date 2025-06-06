use colored::*;
use std::{
    env,
    fs::{self, read_dir},
    path::Path,
};

#[cfg(unix)]
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};

#[cfg(unix)]
fn get_mode(metadata: &fs::Metadata) -> u32 {
    metadata.permissions().mode()
}

#[cfg(not(unix))]
fn get_mode(_: &fs::Metadata) -> u32 {
    0o666
}

#[cfg(unix)]
fn get_file_size(metadata: &fs::Metadata) -> u64 {
    metadata.size()
}

#[cfg(not(unix))]
fn get_file_size(metadata: &fs::Metadata) -> u64 {
    metadata.len()
}

fn get_directory_size(path: &Path) -> u64 {
    let mut total_size = 0;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    total_size += get_directory_size(&path);
                } else {
                    total_size += get_file_size(&metadata);
                }
            }
        }
    }

    total_size
}

#[cfg(unix)]
fn classify_type(file_type: &std::fs::FileType, file_name: String) -> ColoredString {
    match file_type {
        _ if file_type.is_file() => file_name.white(),
        _ if file_type.is_dir() => file_name.bold().cyan(),
        _ if file_type.is_symlink() => file_name.green(),
        _ if file_type.is_block_device() => file_name.yellow(),
        _ if file_type.is_fifo() => file_name.bold().blue(),
        _ if file_type.is_socket() => file_name.bold().magenta(),
        _ => file_name.bold().red(),
    }
}

#[cfg(not(unix))]
fn classify_type(file_type: &std::fs::FileType, file_name: String) -> ColoredString {
    match file_type {
        _ if file_type.is_file() => file_name.white(),
        _ if file_type.is_dir() => file_name.bold().cyan(),
        _ if file_type.is_symlink() => file_name.green(),
        _ => file_name.bold().red(),
    }
}

fn to_rwx_string(mode: u32, is_dir: bool) -> String {
    let mut permission_str = String::new();
    let perms = mode & 0o777;

    permission_str.push(if is_dir { 'd' } else { '-' });
    for i in (0..3).rev() {
        let bits = (perms >> (i * 3)) & 0b111;
        permission_str.push(if bits & 0b100 != 0 { 'r' } else { '-' });
        permission_str.push(if bits & 0b010 != 0 { 'w' } else { '-' });
        permission_str.push(if bits & 0b001 != 0 { 'x' } else { '-' });
    }
    permission_str
}

fn format_size(bytes: u64) -> String {
    let units = ["By", "KB", "MB", "GB", "TB", "PB"];
    let mut size: f32 = bytes as f32;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < units.len() - 1 {
        size = size / 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, units[unit_index])
}

fn parse_args() -> (bool, String) {
    let mut recursive = false;
    let mut path = ".".to_string();

    for arg in env::args().skip(1) {
        if arg == "-r" || arg == "--recursive" {
            recursive = true;
        } else {
            path = arg;
        }
    }

    (recursive, path)
}

fn main() {
    let (is_recursive, path) = parse_args();
    let mut entries = read_dir(path)
        .expect("Dir not found")
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .expect("Failed to read dir");

    entries.sort();
    entries.sort_by_key(|p| !p.is_dir());

    println!(
        "{:<10} {:>8} {}",
        "perm".yellow(),
        "size".yellow(),
        "name".yellow()
    );
    for entry in &mut entries {
        let metadata = fs::metadata(&entry).unwrap_or_else(|_| {
            eprintln!("Failed to read metadata for {:?}", entry);
            std::process::exit(1);
        });

        let file_type = metadata.file_type();
        let is_dir = metadata.is_dir();
        let file_name = entry.file_name().unwrap().to_string_lossy().into_owned();
        let size = if is_dir && is_recursive {
            get_directory_size(entry.as_path())
        } else if is_dir && !is_recursive {
            0
        } else {
            get_file_size(&metadata)
        };
        let mode = get_mode(&metadata);

        let permissions = to_rwx_string(mode, is_dir);
        let pretty_size = format_size(size);
        let colored_file_name = classify_type(&file_type, file_name);

        println!(
            "{:^10} {:>8} {}",
            permissions, pretty_size, colored_file_name
        );
    }
}
