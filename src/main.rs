mod pls;

use clap::Parser;
use colored::*;
use pls::*;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{fs, os::unix::fs::MetadataExt, path::Path};
use walkdir::WalkDir;

struct Entry {
    permissions: String,
    file_name: ColoredString,
    size: u64,
    size_str: String,
    is_dir: bool,
    is_symlink: bool,
    file_name_str: String,
    path: String,
}

fn main() {
    let args = Args::parse();
    let is_recursive = args.recursive;
    let path = args.path;
    let can_follow_symlinks = args.symlinks;
    let entries = WalkDir::new(&path)
        .max_depth(1)
        .follow_links(can_follow_symlinks)
        .sort_by_key(|i| (!i.file_type().is_dir(), i.file_name().to_os_string()))
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path() != Path::new(&path));

    let mut results: Vec<Entry> = entries
        .par_bridge()
        .map(|entry| {
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
            let path = entry.path();

            let permissions = permissions_to_string(metadata.mode(), is_dir, is_symlink);
            let pretty_size = format_size(size);
            let colored_file_name = colorize_type(&file_type, file_name.clone());

            Entry {
                permissions,
                file_name: colored_file_name,
                size,
                size_str: pretty_size,
                is_dir,
                is_symlink,
                file_name_str: file_name,
                path: path.to_str().unwrap().to_string(),
            }
        })
        .collect();

    results.sort_by_key(|e| (!e.is_dir, e.file_name_str.clone()));

    let target_directory_size = results.iter().map(|e| e.size.clone()).sum();
    let formated_directory_size = format_size(target_directory_size);
    println!("{}: {}\n", "Total size".magenta(), formated_directory_size);

    println!(
        "{:<12} {:>8}  {:<1}",
        "perm".yellow(),
        "size".yellow(),
        "name".yellow()
    );

    results.into_iter().for_each(|entry| {
        print!(
            "{:<12} {:>8}  {:<1}",
            entry.permissions, entry.size_str, entry.file_name
        );
        if entry.is_symlink {
            if let Ok(target) = fs::read_link(entry.path) {
                println!(" {} {}", "->".bold().red(), target.display());
            }
        } else {
            println!("");
        }
    });
}
