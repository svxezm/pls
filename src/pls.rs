use crate::permissions;
use clap::Parser;
use colored::*;
use rayon::prelude::*;
use std::{
    fs::{read_dir, FileType},
    path::Path,
};

fn iterate_dir(entry: std::fs::DirEntry) -> u64 {
    let Ok(metadata) = entry.metadata() else {
        return 0;
    };

    if metadata.is_dir() {
        get_directory_size(&entry.path())
    } else {
        metadata.len()
    }
}

pub fn get_directory_size(path: &Path) -> u64 {
    let Ok(entries) = read_dir(path) else {
        return 0;
    };

    entries.flatten().par_bridge().map(iterate_dir).sum()
}

pub fn colorize_type(file_type: &FileType, file_name: String) -> ColoredString {
    match file_type {
        _ if file_type.is_file() => file_name.truecolor(255, 224, 225),
        _ if file_type.is_dir() => file_name.bold().cyan(),
        _ if file_type.is_symlink() => file_name.green(),
        _ => file_name.bold().red(),
    }
}

pub fn format_size(bytes: u64) -> String {
    let units = ["By", "KB", "MB", "GB", "TB", "PB"];
    let mut size: f32 = bytes as f32;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < units.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, units[unit_index])
}

fn parse_entries(entry: walkdir::DirEntry, args: &Args) -> Entry {
    let link_stat = entry
        .path()
        .symlink_metadata()
        .expect("failed to find symlink");
    let metadata = if args.symlinks {
        entry.metadata().expect("Failed to fetch entry metadata")
    } else {
        link_stat.clone()
    };

    let file_type = link_stat.file_type();
    let is_dir = file_type.is_dir();
    let is_symlink = file_type.is_symlink();
    let file_name = entry
        .file_name()
        .to_str()
        .expect("file name not found")
        .to_string();
    let size = match (is_dir, args.recursive) {
        (true, true) => get_directory_size(entry.path()),
        (true, false) => 0,
        _ => metadata.len(),
    };
    let path = entry
        .path()
        .to_str()
        .expect("entry path not found")
        .to_string();

    let permissions = permissions::to_string(metadata, is_dir, is_symlink);
    let pretty_size = format_size(size);
    let colored_file_name = colorize_type(&file_type, file_name.clone());

    Entry {
        permissions,
        file_name: colored_file_name,
        size,
        size_str: pretty_size,
        is_symlink,
        file_name_str: file_name,
        path,
    }
}

pub fn get_results(entries: Vec<walkdir::DirEntry>, args: Args) -> Vec<Entry> {
    entries
        .into_iter()
        .par_bridge()
        .map(|entry| parse_entries(entry, &args))
        .collect()
}

pub struct Entry {
    pub permissions: String,
    pub file_name: ColoredString,
    pub size: u64,
    pub size_str: String,
    pub is_symlink: bool,
    pub file_name_str: String,
    pub path: String,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(default_value = ".")]
    pub path: String,

    #[arg(
        short,
        long,
        help = "Iterate through directories to show their sizes"
    )]
    pub recursive: bool,

    #[arg(short, long, help = "Follow system links")]
    pub symlinks: bool,
}
