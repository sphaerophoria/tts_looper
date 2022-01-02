use log::error;
use tts_loop::channel::{self, Item};

fn main() {
    env_logger::init();

    let (req_tx, req_rx) = channel::channel();
    let mut looper = tts_loop::TtsLooper::new().expect("Failed to construct tts looper");
    let gui = gui::run(req_tx, &flite::list_voices());

    loop {
        let req = req_rx.recv();
        match req {
            Item::Some(gui::GuiRequest::Start {
                text,
                num_iters,
                play_audio,
                voice,
            }) => {
                gui.push_loop_start(&text, &voice, num_iters);
                let res = looper.text_to_text_loop(
                    text,
                    play_audio,
                    num_iters,
                    voice,
                    || req_rx.peek_cancel(),
                    |s| gui.push_output(s),
                );

                match res {
                    // Special case for cancel, we'll log it in the next loop
                    Err(tts_loop::Error::Canceled) => (),
                    Err(ref e) => {
                        error!("{}", e);
                        gui.push_error(&e.to_string());
                    }
                    _ => (),
                }
            }
            Item::Some(gui::GuiRequest::Shutdown) => break,
            Item::Cancel => {
                gui.push_output("Canceled outstanding work");
            }
        }
    }
}
