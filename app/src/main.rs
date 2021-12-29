fn main() {
    let looper = tts_loop::TtsLooper::new();
    let mut gui = gui::Gui::new(looper);
    gui.exec();
}
