mod pls;

use clap::Parser;
use colored::*;
use pls::*;
use std::{fs, path::Path};
use walkdir::{DirEntry, WalkDir};

fn main() {
    let args = Args::parse();
    let path = &args.path;
    let can_follow_symlinks = args.symlinks;
    let entries = WalkDir::new(&path)
        .max_depth(1)
        .follow_links(can_follow_symlinks)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path() != Path::new(&path))
        .collect::<Vec<DirEntry>>();

    let mut results: Vec<pls::Entry> = get_results(entries, args);

    results.sort_by_key(|e| e.file_name_str.clone());

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
