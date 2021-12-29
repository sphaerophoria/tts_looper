use deepspeech::{Model as DsModel, errors::DeepspeechError};
use std::path::Path;


pub struct TtsLooper {
    stt_model: DsModel,
    sample_rate: i32
}

// DsModel has a pointer in it that rust doesn't know can be moved between
// threads. This is annoying in our gui where it's simplest to move the
// TtsLooper into a new thread to do the processing while the gui lives on the
// main loop. We'll need to keep an eye out for issues here, but it's probably
// fine
unsafe impl Send for TtsLooper {}

impl TtsLooper {
    pub fn new() -> TtsLooper {
        let stt_model = DsModel::load_from_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("res/deepspeech-0.9.3-models.tflite")).unwrap();
        let sample_rate = stt_model.get_sample_rate();
        TtsLooper {
            stt_model,
            sample_rate,
        }
    }

    pub fn speech_to_text<B: AsRef<[i16]>>(&mut self, buf: B) -> Result<String, DeepspeechError> {
        let buf = buf.as_ref();
        self.stt_model.speech_to_text(buf)
    }

    pub fn text_to_speech(&self, s: String) -> flite::FliteWav {
        flite::text_to_wave(s, self.sample_rate)
    }

    pub fn play_buf(&self, _buf: &flite::FliteWav) {
        // TODO
    }

    pub fn loop_tts(&mut self, text: String, num_iters: usize) -> Vec<String> {
        let mut ret = Vec::new();
        let mut s = text;
        ret.push(s.clone());
        for _ in 0..num_iters {
            let buf = self.text_to_speech(s);
            s = self.speech_to_text(&*buf).unwrap();
            ret.push(s.clone())
        }

        return ret;
    }
}
