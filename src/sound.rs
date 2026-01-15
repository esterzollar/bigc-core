use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub enum SoundCommand {
    Play(String),
    SetVolume(f32),
    Beep,
}

pub fn start_sound_engine() -> Sender<SoundCommand> {
    let (tx, rx): (Sender<SoundCommand>, Receiver<SoundCommand>) = channel();

    thread::spawn(move || {
        // Initialize Rodio
        let stream_result = OutputStream::try_default();
        if stream_result.is_err() {
            for _ in rx {}
            return;
        }

        let (_stream, stream_handle) = stream_result.unwrap();
        let sink_result = Sink::try_new(&stream_handle);

        if sink_result.is_err() {
            for _ in rx {}
            return;
        }

        let sink = sink_result.unwrap();

        for cmd in rx {
            match cmd {
                SoundCommand::Play(path) => {
                    println!("BigC Audio: Received Play Command for '{}'", path);
                    if let Ok(file) = File::open(&path) {
                        let reader = BufReader::new(file);
                        if let Ok(source) = Decoder::new(reader) {
                            println!("BigC Audio: Decoding success. Appending to sink.");
                            sink.append(source);
                        } else {
                            eprintln!("BigC Audio Error: Failed to decode '{}'", path);
                        }
                    } else {
                        eprintln!("BigC Audio Error: File not found '{}'", path);
                    }
                }
                SoundCommand::SetVolume(vol) => {
                    println!("BigC Audio: Setting Volume to {}", vol);
                    sink.set_volume(vol);
                }
                SoundCommand::Beep => {
                    println!("BigC Audio: Generating Beep");
                    let source = rodio::source::SineWave::new(440.0)
                        .take_duration(std::time::Duration::from_secs_f32(0.5))
                        .amplify(0.5);
                    sink.append(source);
                }
            }
        }
    });

    tx
}
