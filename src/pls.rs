use clap::Parser;
use colored::*;
use std::{
    fs::{read_dir, FileType, Metadata},
    path::Path,
};

pub fn get_permission_mode(metadata: &Metadata) -> u32 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode()
    }

    #[cfg(not(unix))]
    {
        0o666
    }
}

pub fn get_directory_size(path: &Path) -> u64 {
    let mut total_size = 0;

    if let Ok(entries) = read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    total_size += get_directory_size(&path);
                } else {
                    total_size += &metadata.len();
                }
            }
        }
    }

    total_size
}

pub fn colorize_type(file_type: &FileType, file_name: String) -> ColoredString {
    match file_type {
        _ if file_type.is_file() => file_name.truecolor(255, 224, 225),
        _ if file_type.is_dir() => file_name.bold().cyan(),
        _ if file_type.is_symlink() => file_name.green(),
        _ => file_name.bold().red(),
    }
}

pub fn permissions_to_string(mode: u32, is_dir: bool, is_symlink: bool) -> String {
    let mut permission_str = String::new();
    let perms = mode & 0o777;

    permission_str.push(match (is_dir, is_symlink) {
        (true, false) => 'd',
        (false, true) => 'l',
        _ => '-',
    });
    for i in (0..3).rev() {
        let bits = (perms >> (i * 3)) & 0b111;
        permission_str.push(if bits & 0b100 != 0 { 'r' } else { '-' });
        permission_str.push(if bits & 0b010 != 0 { 'w' } else { '-' });
        permission_str.push(if bits & 0b001 != 0 { 'x' } else { '-' });
    }

    permission_str
}

pub fn format_size(bytes: u64) -> String {
    let units = ["By", "KB", "MB", "GB", "TB", "PB"];
    let mut size: f32 = bytes as f32;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < units.len() - 1 {
        size = size / 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, units[unit_index])
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(default_value = ".")]
    pub path: String,

    #[arg(short, long, help = "Iterate through directories to show their sizes")]
    pub recursive: bool,

    #[arg(short, long, help = "Follow system links")]
    pub symlinks: bool,
}
