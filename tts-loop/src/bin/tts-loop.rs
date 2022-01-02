fn main() {
    env_logger::init();
    tts_loop::run().expect("Failed to start tts looper");
}
