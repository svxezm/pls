mod pls;

use clap::Parser;
use colored::*;
use pls::*;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{fs, os::unix::fs::MetadataExt};
use walkdir::WalkDir;

fn main() {
    let args = Args::parse();
    let is_recursive = args.recursive;
    let path = args.path;
    let can_follow_symlinks = args.symlinks;
    let entries = WalkDir::new(path)
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

    entries.par_bridge().for_each(|entry| {
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
        let permissions = permissions_to_string(metadata.mode(), is_dir, is_symlink);
        let pretty_size = format_size(size);
        let colored_file_name = colorize_type(&file_type, file_name);

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
    });
}
