use log::error;
use std::sync::mpsc;

fn main() {
    env_logger::init();

    let (play_tx, play_rx) = mpsc::channel();
    let (req_tx, req_rx) = mpsc::channel();

    let mut looper = tts_loop::TtsLooper::new(play_tx).expect("Failed to construct tts looper");
    let gui = gui::run(req_tx, &flite::list_voices());

    std::thread::spawn(move || {
        let mut audio =
            tts_loop::AudioPlayback::new(play_rx).expect("Failed to construct audio playback");
        loop {
            if let Err(e) = audio.exec() {
                error!("Audio playback error: {}", e);
            }
        }
    });

    while let Ok(req) = req_rx.recv() {
        match req {
            gui::GuiRequest::Start {
                text,
                num_iters,
                play_audio,
                voice,
            } => {
                gui.push_loop_start(&text, &voice, num_iters);
                let res =
                    looper.text_to_text_loop(text, play_audio, num_iters, voice, |s| gui.push_output(s));
                if let Err(e) = res {
                    error!("{}", e);
                    gui.push_error(&e.to_string());
                }
            }
            gui::GuiRequest::Shutdown => break,
        }
    }
}
