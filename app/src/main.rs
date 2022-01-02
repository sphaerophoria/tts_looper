use log::error;
use std::sync::mpsc;

fn clear_req_queue(req_rx: &mpsc::Receiver<gui::GuiRequest>) -> bool {
    let mut ret = false;
    loop {
        match req_rx.try_recv() {
            Ok(gui::GuiRequest::Shutdown) => ret = true,
            Ok(_) => {}
            Err(_) => break,
        }
    }

    return ret;
}

fn main() {
    env_logger::init();

    let (play_tx, play_rx) = mpsc::channel();
    let (req_tx, req_rx) = mpsc::channel();
    let (cancel_tx, cancel_rx) = mpsc::channel();

    let mut looper = tts_loop::TtsLooper::new(play_tx).expect("Failed to construct tts looper");
    let gui = gui::run(req_tx, cancel_tx, &flite::list_voices());

    std::thread::spawn(move || {
        let mut audio =
            tts_loop::AudioPlayback::new(play_rx).expect("Failed to construct audio playback");
        loop {
            if let Err(e) = audio.exec() {
                error!("Audio playback error: {}", e);
            }
        }
    });

    'outer: while let Ok(req) = req_rx.recv() {
        while cancel_rx.try_recv().is_ok() {
            // Clear outstanding requests
            if clear_req_queue(&req_rx) {
                break 'outer;
            }
        }

        match req {
            gui::GuiRequest::Start {
                text,
                num_iters,
                play_audio,
                voice,
            } => {
                gui.push_loop_start(&text, &voice, num_iters);
                let res =
                    looper.text_to_text_loop(text, play_audio, num_iters, voice, &cancel_rx, |s| {
                        gui.push_output(s)
                    });

                if let Err(e) = res {
                    error!("{}", e);
                    gui.push_error(&e.to_string());

                    if let tts_loop::Error::Canceled = e {
                        if clear_req_queue(&req_rx) {
                            break 'outer;
                        }
                    }
                }
            }
            gui::GuiRequest::Shutdown => break,
        }
    }
}
