use record_audio::audio_clip::AudioClip as ac;

fn main() {
    let file_name = "hello.wav";
    match ac::record(None) {
        Ok(clip) => {
            println!("Successfully recorded!");
            match clip.export(format!("{}" , file_name).as_str()) {
                Ok(_) => {
                    println!("Successfully saved as hello.wav");
                    
                    //to play immediately, after ctrl-c
                    //clip.play().unwrap();
                }
                Err(err) => println!("Error {}", err),
            }
        }
        Err(err) => println!("Error {}", err),
    }

}