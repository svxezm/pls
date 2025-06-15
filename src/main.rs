use clap::Parser;
use colored::*;
use std::{fs, path::Path};
use walkdir::WalkDir;

fn get_mode(metadata: &std::fs::Metadata) -> u32 {
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

fn get_directory_size(path: &Path) -> u64 {
    let mut total_size = 0;

    if let Ok(entries) = std::fs::read_dir(path) {
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

fn classify_type(file_type: &std::fs::FileType, file_name: String) -> ColoredString {
    match file_type {
        _ if file_type.is_file() => file_name.truecolor(255, 224, 225),
        _ if file_type.is_dir() => file_name.bold().cyan(),
        _ if file_type.is_symlink() => file_name.green(),
        _ => file_name.bold().red(),
    }
}

fn permissions_to_string(mode: u32, is_dir: bool, is_symlink: bool) -> String {
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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(default_value = ".")]
    path: String,

    #[arg(short, long, help = "Iterate through directories to show their sizes")]
    recursive: bool,

    #[arg(short, long, help = "Follow system links")]
    symlinks: bool,
}

fn main() {
    let args = Args::parse();
    let is_recursive = args.recursive;
    let path = args.path;
    let can_follow_symlinks = args.symlinks;
    let mut entries = WalkDir::new(path)
        .max_depth(1)
        .follow_links(can_follow_symlinks)
        .sort_by_key(|i| (!i.file_type().is_dir(), i.file_name().to_os_string()))
        .into_iter()
        .filter_map(|e| e.ok());

    println!(
        "{:<12} {:>8}  {:<1}",
        "perm".yellow(),
        "size".yellow(),
        "name".yellow()
    );
    for entry in &mut entries {
        let link_stat = entry.path().symlink_metadata().unwrap();
        let metadata = if can_follow_symlinks {
            entry.metadata().unwrap()
        } else {
            link_stat.clone()
        };

        let file_type = link_stat.file_type();
        let is_dir = file_type.is_dir();
        let is_symlink = file_type.is_symlink();
        let file_name = entry.file_name().to_str().unwrap().to_string();
        let size = match (is_dir, is_recursive) {
            (true, true) => get_directory_size(&entry.path()),
            (true, false) => 0,
            _ => metadata.len(),
        };
        let mode = get_mode(&metadata);

        let permissions = permissions_to_string(mode, is_dir, is_symlink);
        let pretty_size = format_size(size);
        let colored_file_name = classify_type(&file_type, file_name);

        print!(
            "{:<12} {:>8}  {:<1}",
            permissions, pretty_size, colored_file_name
        );
        if is_symlink {
            if let Ok(target) = fs::read_link(entry.path()) {
                println!(" {} {}", "->".bold().red(), target.display());
            }
        } else {
            println!("");
        }
    }
}
