use crate::{audio::AudioManager, gui::GuiHandle};

use deepspeech::{errors::DeepspeechError, Model as DsModel};
use hound::{WavSpec, WavWriter};
use log::{error, info, warn};
use thiserror::Error as ThisError;

use std::convert::TryFrom;
use std::{
    convert::TryInto,
    path::{Path, PathBuf},
    sync::{
        mpsc::{self, Receiver},
        Arc,
    },
};

mod audio;
mod gui;
mod logger;

pub use logger::init_logger;

const SAMPLE_RATE: u32 = 16000;

pub(crate) enum Request {
    TtsLoop { text: String, num_iters: i32 },
    SetVoice { voice: String },
    EnableAudio { enable: bool },
    Cancel,
    Shutdown,
    Save { path: PathBuf },
    StartRecording,
    EndRecording,
}

struct Settings {
    enable_audio: bool,
    voice: String,
}

enum LoopStatePhase {
    Tts,
    Playback,
    Stt,
    Finished,
}

struct LoopState {
    phase: LoopStatePhase,
    text: String,
    wav: Vec<i16>,
    last_frame_len: usize,
    remaining_iters: usize,
}

impl LoopState {
    fn new() -> LoopState {
        LoopState {
            phase: LoopStatePhase::Finished,
            text: String::new(),
            wav: Vec::new(),
            last_frame_len: 0,
            remaining_iters: 0,
        }
    }

    fn last_frame(&self) -> &[i16] {
        assert!(self.wav.len() >= self.last_frame_len);
        let frame_start = self.wav.len() - self.last_frame_len;
        &self.wav[frame_start..]
    }

    fn is_finished(&self) -> bool {
        match self.phase {
            LoopStatePhase::Tts | LoopStatePhase::Stt | LoopStatePhase::Playback => false,
            LoopStatePhase::Finished => true,
        }
    }

    fn set_finished(&mut self) {
        self.phase = LoopStatePhase::Finished;
    }
}

enum Recording {
    Ongoing {
        _stream: cpal::Stream,
        rx: mpsc::Receiver<Vec<i16>>,
    },
    Finished {
        buf: Vec<i16>,
    },
}

impl Recording {
    fn start_recording(&mut self, audio_manager: &AudioManager) -> Result<(), Error> {
        match self {
            Recording::Ongoing { .. } => (),
            Recording::Finished { .. } => {
                let (tx, rx) = mpsc::channel();

                let _stream = audio_manager.input_stream(SAMPLE_RATE, move |buf| {
                    let _ = tx.send(buf.to_owned());
                })?;

                *self = Recording::Ongoing { _stream, rx };
            }
        }

        Ok(())
    }

    fn stop_recording(&mut self) {
        let buf = match self {
            Recording::Finished { .. } => return,
            Recording::Ongoing { rx, .. } => rx
                .try_iter()
                .flat_map(|v| v.into_iter())
                .collect::<Vec<i16>>(),
        };

        *self = Recording::Finished { buf }
    }
}

#[derive(ThisError, Debug)]
pub enum Error {
    #[error(transparent)]
    DeepspeechError(#[from] DeepspeechError),
    #[error(transparent)]
    AudioManager(#[from] audio::Error),
    #[error("Action canceled by user")]
    Canceled,
    #[error("Failed to write wav to file: {0}")]
    WavWriteError(hound::Error),
    #[error("Data not available")]
    NoData,
    #[error("Recording in progress")]
    CurrentlyRecording,
    #[error("Gui request handle no longer active")]
    GuiRecvError,
    #[error("Invalid C string")]
    NulError(#[from] std::ffi::NulError),
    #[error("Cannot execute {attempted_action} while {blocking_action} is running")]
    Busy {
        attempted_action: String,
        blocking_action: String,
    },
}

enum AppState {
    Running,
    Shutdown,
}

pub struct TtsLooper {
    stt_model: DsModel,
    audio_manager: AudioManager,
    gui: Arc<GuiHandle>,
    gui_rx: Receiver<Request>,
    work: LoopState,
    recording: Recording,
    settings: Settings,
}

impl TtsLooper {
    pub fn new() -> Result<TtsLooper, Error> {
        let path =
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("res/deepspeech-0.9.3-models.tflite");
        let stt_model = DsModel::load_from_files(&path)?;

        let sample_rate = stt_model.get_sample_rate();
        assert!(u32::try_from(sample_rate).unwrap() == SAMPLE_RATE);

        let audio_manager = AudioManager::new();
        let voices = flite::list_voices();

        let (tx, rx) = mpsc::channel();

        let gui = gui::run(tx, &voices);

        let settings = Settings {
            voice: voices[0].to_string(),
            enable_audio: false,
        };

        Ok(TtsLooper {
            stt_model,
            audio_manager,
            gui: Arc::new(gui),
            gui_rx: rx,
            work: LoopState::new(),
            recording: Recording::Finished { buf: Vec::new() },
            settings,
        })
    }

    pub fn run(&mut self) {
        let mut loop_fn = || -> Result<(), Error> {
            loop {
                while !self.work.is_finished() {
                    if let Ok(req) = self.gui_rx.try_recv() {
                        if let AppState::Shutdown = self.handle_request(req)? {
                            return Ok(());
                        }
                        continue;
                    }

                    self.iterate_work()?;
                }

                // All work is complete, sleep until more work is queued
                let req = self.gui_rx.recv().map_err(|_| Error::GuiRecvError)?;
                if let AppState::Shutdown = self.handle_request(req)? {
                    return Ok(());
                }
            }
        };

        while let Err(e) = loop_fn() {
            error!("{}", e);
        }
    }

    fn handle_request(&mut self, req: Request) -> Result<AppState, Error> {
        match req {
            Request::Cancel => {
                if !self.work.is_finished() {
                    self.work.set_finished();
                    warn!("Canceled executing job");
                }
            }
            Request::SetVoice { voice } => {
                self.settings.voice = voice.clone();
                info!("Voice changed: {}", voice);
            }
            Request::EnableAudio { enable } => {
                self.settings.enable_audio = enable;
                if enable {
                    info!("Audio playback enabled")
                } else {
                    info!("Audio playback disabled")
                }
            }
            Request::StartRecording => {
                self.recording.start_recording(&self.audio_manager)?;
                info!("Recording started");
            }
            Request::EndRecording => {
                self.recording.stop_recording();
                info!("Recording stopped");
                let text = self.recording_to_text()?;
                info!("Recorded text: {}", text);
                self.gui.push_input_text(&text);
            }
            Request::Save { path } => {
                self.save_full_wav(&path)?;
            }
            Request::TtsLoop { text, num_iters } => {
                if !self.work.is_finished() {
                    return Err(Error::Busy {
                        attempted_action: "tts loop".to_string(),
                        blocking_action: "tts loop".to_string(),
                    });
                }

                info!(
                    "Starting work. Text: {}, Number of iterations: {}",
                    text, num_iters
                );

                self.work = LoopState {
                    phase: LoopStatePhase::Tts,
                    text,
                    wav: Vec::new(),
                    last_frame_len: 0,
                    remaining_iters: num_iters.try_into().unwrap(),
                };
            }
            Request::Shutdown => {
                return Ok(AppState::Shutdown);
            }
        }

        Ok(AppState::Running)
    }

    fn iterate_work(&mut self) -> Result<(), Error> {
        self.work.phase = match self.work.phase {
            LoopStatePhase::Playback => {
                if self.settings.enable_audio {
                    self.audio_manager
                        .play_buf_blocking(self.work.last_frame(), SAMPLE_RATE)?;
                }
                LoopStatePhase::Stt
            }
            LoopStatePhase::Stt => {
                self.work.text = self.stt_model.speech_to_text(self.work.last_frame())?;
                self.gui.push_output(&self.work.text);

                self.work.remaining_iters = self.work.remaining_iters.saturating_sub(1);
                if self.work.remaining_iters == 0 {
                    info!("Tts loop complete");
                    LoopStatePhase::Finished
                } else {
                    LoopStatePhase::Tts
                }
            }
            LoopStatePhase::Tts => {
                let wav = flite::text_to_wave(
                    self.work.text.clone(),
                    SAMPLE_RATE as i32,
                    self.settings.voice.clone(),
                )?;
                self.work.last_frame_len = wav.len();
                self.work.wav.extend(wav.iter());
                LoopStatePhase::Playback
            }
            LoopStatePhase::Finished => LoopStatePhase::Finished,
        };

        Ok(())
    }

    fn recording_to_text(&mut self) -> Result<String, Error> {
        let buf = match &self.recording {
            Recording::Finished { buf } => buf,
            Recording::Ongoing { .. } => {
                return Err(Error::CurrentlyRecording);
            }
        };

        Ok(self.stt_model.speech_to_text(buf)?)
    }

    fn save_full_wav<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let full_wav = match self.work.phase {
            LoopStatePhase::Finished => &self.work.wav,
            _ => {
                return Err(Error::Busy {
                    attempted_action: "save".to_string(),
                    blocking_action: "tts loop".to_string(),
                })
            }
        };

        let wav_spec = WavSpec {
            channels: 1,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = WavWriter::create(path, wav_spec).unwrap();
        for sample in full_wav {
            writer.write_sample(*sample).map_err(Error::WavWriteError)?;
        }

        Ok(())
    }
}
