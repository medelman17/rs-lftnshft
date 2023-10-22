use clap::Parser;
use std::collections::HashMap;
use std::{fs};
use std::path::{Path};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Opts {
    #[arg(short, long, default_value = "./")]
    start_directory: String,

    #[arg(short, long)]
    target_directory: Option<String>,

    #[arg(short, long, default_value = "mp4,mp3,jpg,png,docx,wma,pdf,psd,jpeg,gif,doc,avi,wmv,flv,mov")]
    media_extensions: String,
}

fn find_and_copy_media_files(
    start_directory: &Path,
    target_directory: Option<&Path>,
    extensions: &[&str],
    metrics: &mut HashMap<String, (u64, u64)>,
    depth: usize,
) {
    let indent = "--|".repeat(depth);
    println!("{}> Entering directory: {:?}", indent, start_directory.display());

    if let Ok(entries) = fs::read_dir(start_directory) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    find_and_copy_media_files(&path, target_directory, extensions, metrics, depth + 1);
                } else if let Some(extension) = path.extension() {
                    let ext_str = extension.to_str().unwrap();
                    if extensions.contains(&ext_str) {
                        if let Ok(metadata) = fs::metadata(&path) {
                            let file_size = metadata.len();

                            let counter = metrics.entry(ext_str.to_string()).or_insert((0, 0));
                            counter.0 += 1;
                            counter.1 += file_size;

                            if let Some(target_directory) = target_directory {
                                let subfolder_path = target_directory.join(ext_str);
                                if let Err(_) = fs::create_dir_all(&subfolder_path) {
                                    println!("Failed to create subdirectory for extension: {}", ext_str);
                                    continue;
                                }
                                let target_path = subfolder_path.join(path.file_name().unwrap());
                                if let Err(e) = fs::copy(&path, &target_path) {
                                    println!("Failed to copy {:?}: {}", path, e);
                                } else {
                                    println!("{}Found and copied media file: {:?}", indent, path);
                                }
                            } else {
                                println!("{}Found media file but not copying: {:?}", indent, path);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let opts = Opts::parse();

    let start_directory = opts.start_directory.as_str();
    let target_directory = opts.target_directory.as_ref();  // This is now an Option<&str>
    let media_extensions: Vec<&str> = opts.media_extensions.split(',').collect();

    let start_time = Instant::now();
    let mut metrics = HashMap::new();

    let target_directory_path = target_directory.map(Path::new);

    if let Some(target_path) = target_directory_path {
        if let Err(e) = fs::create_dir_all(target_path) {
            println!("Failed to create target directory: {}", e);
            return;
        }
    }

    println!("Starting search for media files in directory: {}", start_directory);
    find_and_copy_media_files(Path::new(start_directory), target_directory_path, &media_extensions, &mut metrics, 0);

    let duration = start_time.elapsed();

    println!("\nSearch and copy completed in {:.2?}", duration);
    println!("=== Summary ===");

    let mut total_files = 0;
    let mut total_size = 0;

    for (ext, (count, size)) in &metrics {
        println!("Extension: .{}, Count: {}, Aggregate Size: {} bytes", ext, count, size);
        total_files += count;
        total_size += size;
    }

    println!("Total Files: {}", total_files);
    println!("Total Size: {} bytes", total_size);
}
