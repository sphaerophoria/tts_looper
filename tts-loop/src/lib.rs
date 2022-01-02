use deepspeech::{errors::DeepspeechError, Model as DsModel};
use flite::FliteWav;
use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
use std::convert::TryInto;
use std::ffi::NulError;
use std::path::Path;
use thiserror::Error as ThisError;

pub mod channel;

pub struct TtsLooper {
    stt_model: DsModel,
    sample_rate: i32,
    audio: AudioPlayback,
}

#[derive(ThisError, Debug)]
pub enum Error {
    #[error(transparent)]
    DeepspeechError(#[from] DeepspeechError),
    #[error(transparent)]
    Audio(#[from] AudioError),
    #[error("Cannot play audio, audio channel invalid")]
    PlayAudio,
    #[error("Action canceled by user")]
    Canceled,
    #[error("Cannot convert text with a null character")]
    TtsError(NulError),
}

impl TtsLooper {
    pub fn new() -> Result<TtsLooper, Error> {
        let path =
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("res/deepspeech-0.9.3-models.tflite");
        let stt_model = DsModel::load_from_files(&path)?;
        let sample_rate = stt_model.get_sample_rate();
        let audio = AudioPlayback::new()?;
        Ok(TtsLooper {
            stt_model,
            sample_rate,
            audio,
        })
    }

    pub fn speech_to_text<B: AsRef<[i16]>>(&mut self, buf: B) -> Result<String, Error> {
        let buf = buf.as_ref();
        Ok(self.stt_model.speech_to_text(buf)?)
    }

    pub fn text_to_speech(&self, s: String, voice: String) -> Result<flite::FliteWav, Error> {
        flite::text_to_wave(s, self.sample_rate, voice).map_err(Error::TtsError)
    }

    pub fn text_to_text(
        &mut self,
        text: String,
        play_audio: bool,
        voice: String,
    ) -> Result<String, Error> {
        let buf = self.text_to_speech(text, voice)?;
        if play_audio {
            self.audio.play_wav(&buf)?;
        }
        self.speech_to_text(&*buf)
    }

    pub fn text_to_text_loop<F: Fn(&str), C: Fn() -> bool>(
        &mut self,
        mut text: String,
        play_audio: bool,
        num_iters: i32,
        voice: String,
        cancel_fn: C,
        status_fn: F,
    ) -> Result<(), Error> {
        for _ in 0..num_iters {
            if cancel_fn() {
                return Err(Error::Canceled);
            }

            text = self.text_to_text(text, play_audio, voice.clone())?;
            status_fn(&text);
        }
        Ok(())
    }
}

#[derive(ThisError, Debug)]
pub enum AudioError {
    #[error("Failed to open audio device: {0}")]
    OutputOpenError(String),
    #[error("Invalid num channels: {0}")]
    NumChannelsError(i32),
    #[error("Invalid sample rate: {0}")]
    SampleRateError(i32),
}

pub struct AudioPlayback {
    _stream: OutputStream,
    sink: Sink,
}

impl AudioPlayback {
    pub fn new() -> Result<AudioPlayback, AudioError> {
        let (_stream, stream_handle) =
            OutputStream::try_default().map_err(|e| AudioError::OutputOpenError(e.to_string()))?;
        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| AudioError::OutputOpenError(e.to_string()))?;

        sink.play();

        Ok(AudioPlayback { _stream, sink })
    }

    pub fn play_wav(&mut self, wav: &FliteWav) -> Result<(), AudioError> {
        let num_channels = wav.num_channels();
        let num_channels = num_channels
            .try_into()
            .map_err(|_| AudioError::NumChannelsError(num_channels))?;

        let sample_rate = wav.sample_rate();
        let sample_rate = sample_rate
            .try_into()
            .map_err(|_| AudioError::SampleRateError(sample_rate))?;

        let samples = SamplesBuffer::new(num_channels, sample_rate, (*wav).to_vec());
        self.sink.append(samples);
        self.sink.sleep_until_end();

        Ok(())
    }
}
