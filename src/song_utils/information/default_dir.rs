use std::fs::{read_dir};
use std::path::{Path, PathBuf};

pub fn dir_check(dir: Option<String>) -> Result<Vec<PathBuf>, String> {
    // -> Use provided dir from command OR the default.
    let dir = match dir {
        Some(d) => d,  // If a directory is provided, use it.
        None => match get_default_directory() {  // If not, get the default directory.
            Some(default_dir) => default_dir,
            None => return Err("No valid directory provided or found.".to_string()),
        }
    };

    // -> Does directory exist?
    let dir_path = Path::new(&dir);
    if !dir_path.is_dir() {
        return Err(format!("The specified path '{}' is not a valid directory.", dir));
    }

    // -> List MP3 files in the directory.
    let entries = match read_dir(dir_path) {
        Ok(entries) => entries,
        Err(e) => return Err(format!("Failed to read directory '{}': {}", dir, e)),
    };

    // -> Filter MP3 files from the directory.
    let mp3_files: Vec<PathBuf> = entries
        .filter_map(|entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                // Check if the file has an "mp3" extension.
                if path.extension().map(|e| e == "mp3").unwrap_or(false) {
                    return Some(path);
                }
            }
            None
        })
        .collect();

    // -> If no MP3 files were found, return an error.
    if mp3_files.is_empty() {
        return Err(format!("No MP3 files found in the directory '{}'.", dir));
    }

    // -> Return the list of MP3 files.
    Ok(mp3_files)
}

// -> Default get directory function for macOS, Windows, and Linux.
pub fn get_default_directory() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        dirs::home_dir().map(|path| path.join("Music").to_string_lossy().into())
    }

    #[cfg(target_os = "linux")]
    {
        dirs::home_dir().map(|path| path.join("Music").to_string_lossy().into())
    }

    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|path| path.join("Music").to_string_lossy().into())
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        None
    }
}
