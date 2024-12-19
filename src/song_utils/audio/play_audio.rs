use rodio::{Decoder, OutputStreamHandle, Sink};
use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{BufReader, Error as IoError};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;
use rand::prelude::IteratorRandom; // -> Added for `choose`
use crate::song_utils::information::audio_duration::get_audio_duration;

const RECENTLY_PLAYED_LIMIT: usize = 3; // -> Number of recent songs to remember

pub fn play_song(
    mp3_files: &mut Vec<PathBuf>,
    stream_handle: &OutputStreamHandle,
    folder_path: &Path,
) -> Result<(), IoError> {
    let mut rng = rand::thread_rng(); // -> Random number generator for shuffling
    let mut recently_played: VecDeque<String> = VecDeque::new(); // -> Track recently played songs

    // -> Set up a channel for skip signaling
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || listen_for_skip_command(tx)); // -> Spawn a thread to listen for skip commands

    loop {
        // -> Rescan the folder for MP3 files periodically
        match scan_folder_for_mp3s(folder_path) {
            Ok(updated_files) => {
                *mp3_files = updated_files; // -> Update the mutable vector directly
            }
            Err(e) => {
                eprintln!("Error rescanning folder: {}", e);
                // -> Keep using the current list of files
            }
        }

        // -> If no files are available after rescanning, break the loop
        if mp3_files.is_empty() {
            eprintln!("No MP3 files found in the directory.");
            break;
        }

        // -> Choose a random file to play, ensuring it's not in recently played
        let random_file = mp3_files
            .iter()
            .filter(|file| {
                let file_name = file.file_name().unwrap().to_string_lossy().to_string();
                let base_name = file_name.strip_suffix(".mp3").unwrap_or(&file_name).to_string();
                !recently_played.contains(&base_name)
            })
            .choose(&mut rng); // -> Use `IteratorRandom::choose`

        if let Some(random_file) = random_file {
            let file_name = random_file.file_name().unwrap().to_string_lossy().to_string();
            let base_name = file_name.strip_suffix(".mp3").unwrap_or(&file_name).to_string();

            // -> Get the duration of the song
            let duration = match get_audio_duration(random_file.to_str().unwrap()) {
                Ok(duration) => duration,
                Err(e) => {
                    eprintln!("Failed to get duration for file '{}': {}", random_file.display(), e);
                    continue; // -> Skip this song and try again
                }
            };

            // -> Convert the duration to minutes and seconds
            let minutes = duration.as_secs() / 60;
            let seconds = duration.as_secs() % 60;
            let duration_str = format!("[{}:{:02}]", minutes, seconds);

            // -> Load the song file
            let file = match File::open(&random_file) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Failed to open file '{}': {}", random_file.display(), e);
                    continue; // -> Skip this song and try again
                }
            };

            let source = match Decoder::new(BufReader::new(file)) {
                Ok(source) => source,
                Err(e) => {
                    eprintln!("Failed to decode the file '{}': {}", random_file.display(), e);
                    continue; // -> Skip this song and try again
                }
            };

            // -> Create a sink to control playback
            let sink = Sink::try_new(stream_handle).unwrap();
            sink.append(source);

            // -> Print the name of the song and the formatted duration
            println!("Now playing: '{}'", base_name);
            println!("Song Duration: {}", duration_str);

            // -> Wait for the song to finish or skip command
            if wait_for_duration_or_skip(&rx, duration) {
                println!("Song skipped: '{}'", base_name);
                sink.stop(); // -> Stop the current audio
            } else {
                // -> Update recently played list only if not skipped
                recently_played.push_back(base_name.clone());
                if recently_played.len() > RECENTLY_PLAYED_LIMIT {
                    recently_played.pop_front();
                }
            }
        } else {
            eprintln!("No MP3 files available that haven't been recently played.");
        }
    }
    
    // -> Return Ok at the end of the function
    Ok(())
}

fn scan_folder_for_mp3s(folder_path: &Path) -> Result<Vec<PathBuf>, IoError> {
    let entries = fs::read_dir(folder_path)?;
    let mp3_files = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map_or(false, |ext| ext == "mp3"))
        .collect();
    Ok(mp3_files)
}

// -> Function to listen for skip commands
fn listen_for_skip_command(tx: mpsc::Sender<()>) {
    let mut input = String::new();
    loop {
        input.clear();
        if std::io::stdin().read_line(&mut input).is_ok() {
            if input.trim() == "skip" {
                if tx.send(()).is_err() {
                    eprintln!("Failed to send skip signal. Exiting skip listener.");
                    break;
                }
            }
        }
    }
}

/// -> Wait for the duration of the song or a skip signal
fn wait_for_duration_or_skip(rx: &Receiver<()>, duration: Duration) -> bool {
    // -> Check for skip input
    let step = Duration::from_millis(100);
    let mut elapsed = Duration::new(0, 0);

    while elapsed < duration {
        if rx.try_recv().is_ok() {
            // -> Received the skip signal
            return true;
        }
        std::thread::sleep(step);
        elapsed += step;
    }
    false
}
