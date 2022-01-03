use channel::{Receiver, Request};
use deepspeech::{errors::DeepspeechError, Model as DsModel};
use flite::FliteWav;
use gui::GuiHandle;
use log::error;
use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
use std::convert::TryInto;
use std::ffi::NulError;
use std::path::Path;
use thiserror::Error as ThisError;

pub mod channel;
pub mod gui;

pub struct TtsLooper {
    stt_model: DsModel,
    audio: AudioPlayback,
    sample_rate: i32,
    text: String,
    voice: String,
    enable_audio: bool,
    buf: Option<FliteWav>,
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
    #[error("Data not available")]
    NoData,
}

impl TtsLooper {
    pub fn new() -> Result<TtsLooper, Error> {
        let path =
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("res/deepspeech-0.9.3-models.tflite");
        let stt_model = DsModel::load_from_files(&path)?;
        let sample_rate = stt_model.get_sample_rate();
        let audio = AudioPlayback::new()?;
        let voice = flite::list_voices()[0];
        Ok(TtsLooper {
            stt_model,
            audio,
            sample_rate,
            text: String::new(),
            voice: voice.to_string(),
            buf: None,
            enable_audio: false,
        })
    }

    pub fn speech_to_text(&mut self) -> Result<&str, Error> {
        if let Some(buf) = &self.buf {
            self.text = self.stt_model.speech_to_text(&*buf)?;
            return Ok(&self.text);
        }

        Err(Error::NoData)
    }

    pub fn text_to_speech(&mut self) -> Result<(), Error> {
        self.buf = Some(
            flite::text_to_wave(self.text.clone(), self.sample_rate, self.voice.clone())
                .map_err(Error::TtsError)?,
        );
        Ok(())
    }

    pub fn enable_audio(&mut self, enable: bool) {
        self.enable_audio = enable;
    }

    pub fn play_audio(&mut self) -> Result<(), Error> {
        if self.enable_audio {
            if let Some(buf) = &self.buf {
                self.audio.play_wav(buf)?;
            }
        }
        Ok(())
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn set_voice(&mut self, voice: String) {
        self.voice = voice;
    }

    pub fn get_voice(&self) -> &str {
        &self.voice
    }

    pub fn get_wav(&self) -> &Option<FliteWav> {
        &self.buf
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

fn exec_request(
    looper: &mut TtsLooper,
    gui: &GuiHandle,
    req: Request,
    req_rx: &Receiver,
) -> Result<bool, Error> {
    match req {
        Request::SetText { text } => {
            looper.set_text(text);
        }
        Request::LogStart { num_iters } => {
            gui.push_loop_start(looper.get_text(), looper.get_voice(), num_iters);
        }
        Request::SetVoice { voice } => {
            looper.set_voice(voice);
        }
        Request::EnableAudio { enable } => {
            looper.enable_audio(enable);
        }
        Request::RunTts => {
            looper.text_to_speech()?;
        }
        Request::PlayAudio => {
            looper.play_audio()?;
        }
        Request::RunStt => {
            let res = looper.speech_to_text()?;
            gui.push_output(res);
        }
        Request::Cancel => {
            if req_rx.execute_cancel() {
                gui.push_cancel();
            }
        }
        Request::Shutdown => {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn run() -> Result<(), Error> {
    let (req_tx, req_rx) = channel::channel();
    let gui = gui::run(req_tx, &flite::list_voices());
    let mut looper = TtsLooper::new().expect("Failed to construct tts looper");

    loop {
        let req = req_rx.recv();
        match exec_request(&mut looper, &gui, req, &req_rx) {
            Ok(true) => break,
            Ok(false) => (),
            Err(e) => {
                error!("{}", e);
                gui.push_error(&e.to_string());
            }
        }
    }

    Ok(())
}
