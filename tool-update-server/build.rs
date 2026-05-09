use std::fs;
use std::path::Path;

fn main() {
    // Get the project root directory (where Cargo.toml is)
    let project_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let static_src = Path::new(&project_root).join("static");

    // Get the target directory and profile (debug or release)
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| {
        Path::new(&project_root).join("target").to_string_lossy().to_string()
    });
    let profile = std::env::var("PROFILE").unwrap();
    let static_dest = Path::new(&target_dir).join(&profile).join("static");

    // Copy static files if the source directory exists
    if static_src.exists() {
        // Create destination directory
        fs::create_dir_all(&static_dest).expect("Failed to create static destination directory");

        // Copy all files from source to destination
        for entry in fs::read_dir(&static_src).expect("Failed to read static source directory") {
            let entry = entry.expect("Failed to read directory entry");
            let src_path = entry.path();
            let dest_path = static_dest.join(entry.file_name());

            if src_path.is_file() {
                fs::copy(&src_path, &dest_path).expect("Failed to copy static file");
                println!("Copied: {} -> {}", src_path.display(), dest_path.display());
            } else if src_path.is_dir() {
                // Recursively copy subdirectories
                copy_dir_all(&src_path, &dest_path);
            }
        }
    }

    // Rerun if static files change
    println!("cargo:rerun-if-changed=static/");
}

fn copy_dir_all(src: &Path, dest: &Path) {
    fs::create_dir_all(dest).expect("Failed to create directory");

    for entry in fs::read_dir(src).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read directory entry");
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_file() {
            fs::copy(&src_path, &dest_path).expect("Failed to copy file");
        } else if src_path.is_dir() {
            copy_dir_all(&src_path, &dest_path);
        }
    }
}