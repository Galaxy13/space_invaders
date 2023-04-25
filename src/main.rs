use std::error::Error;
use rusty_audio::Audio;
use std::fs;

fn main() -> Result<(), Box<dyn Error>>{
    let mut audio = Audio::new();
    let audio_files = fs::read_dir("sounds/").unwrap();
    for file in audio_files {
        let path = file.unwrap().path().into_os_string().into_string().unwrap();
        println!("{}", &path.trim_end_matches(".wav")[7..]);
        audio.add(&path.trim_end_matches(".wav")[7..], &path)
    };
    audio.play("explode");

    //Cleanup
    audio.wait();
    return Ok(())
}
