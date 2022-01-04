fn main() {
    let mut tts_looper = tts_loop::TtsLooper::new().expect("Failed to construct looper");
    tts_loop::init_logger(&tts_looper);
    tts_looper.run();
}
