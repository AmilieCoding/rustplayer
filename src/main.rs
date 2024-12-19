use std::env;
use rodio::{OutputStream};
use crate::song_utils::information::default_dir::dir_check;
use crate::song_utils::audio::play_audio::play_song;
use std::path::Path;

mod song_utils;

fn main() {
    // -> Command line arguments checker
    let args: Vec<String> = env::args().collect();
    let dir = if args.len() > 2 && args[1] == "--dir" {
        Some(args[2].clone())
    } else {
        None
    };

    // -> Uses provided directory (or the default)
    match dir_check(dir) {
        Ok(mut mp3_files) => {
            // -> Check if there are MP3 files in the directory
            if mp3_files.is_empty() {
                eprintln!("No MP3 files found in the directory.");
                return;
            }

            // -> Get the directory path used by dir_check
            let folder_path = match Path::new(&mp3_files[0]).parent() {
                Some(path) => path.to_path_buf(),
                None => {
                    eprintln!("Failed to determine the parent directory of the MP3 files.");
                    return;
                }
            };

            // -> Audio stream setup
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(stream) => stream,
                Err(e) => {
                    eprintln!("Failed to initialize audio stream: {}", e);
                    return;
                }
            };

            // -> Play songs indefinitely
            if let Err(e) = play_song(&mut mp3_files, &stream_handle, &folder_path) {
                eprintln!("Error during playback: {}", e);
            }
        }
        Err(e) => {
            // -> Error message if directory check fails
            eprintln!("{}", e);
        }
    }
}
