use std::time::Duration;
use ffmpeg_next::format;

pub fn get_audio_duration(file_path: &str) -> Result<Duration, String> {
    if let Err(e) = ffmpeg_next::init() {
        return Err(format!("Failed to initialize FFmpeg: {}", e));
    }

    // -> Get's file duration in μs
    let ictx = match format::input(file_path) {
        Ok(ctx) => ctx,
        Err(e) => return Err(format!("Failed to open file '{}': {}", file_path, e)),
    };

    let duration = ictx.duration();

    // -> Converting to appropriate duration object.
    Ok(Duration::from_micros(duration as u64))
}
