use crate::{
    gui::rich_text::{Color, Format},
    Request,
};

use log::{error, Level};
use thiserror::Error;

use std::{
    convert::TryInto,
    ffi::c_void,
    sync::{mpsc::Sender, Arc},
};

mod rich_text;

mod imp {
    #![allow(non_snake_case)]
    #![allow(non_upper_case_globals)]
    #![allow(unused)]
    #![allow(non_camel_case_types)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

struct ImpHandle {
    handle: *mut imp::Gui,
}

impl std::ops::Deref for ImpHandle {
    type Target = *mut imp::Gui;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Drop for ImpHandle {
    fn drop(&mut self) {
        unsafe { imp::DestroyGui(self.handle) };
    }
}

unsafe impl Send for ImpHandle {}
unsafe impl Sync for ImpHandle {}

pub(crate) struct GuiHandle {
    handle: Arc<ImpHandle>,
}

impl GuiHandle {
    pub(crate) fn push_output(&self, text: &str) {
        unsafe {
            imp::PushOutput(**self.handle, to_gui_string(&text));
        }
    }

    pub(crate) fn push_input_text(&self, text: &str) {
        unsafe {
            imp::PushInputText(**self.handle, to_gui_string(text));
        }
    }

    pub(crate) fn log(&self, text: &str, level: Level) {
        let encoded = Format::bold(Format::text(text));

        let encoded = match level {
            Level::Trace => Format::color(Color::Green, encoded),
            Level::Debug => Format::color(Color::Green, encoded),
            Level::Info => Format::color(Color::Blue, encoded),
            Level::Warn => Format::color(Color::Orange, encoded),
            Level::Error => Format::color(Color::Red, encoded),
        };

        let text = encoded.into_string();

        unsafe {
            imp::PushRawOutput(**self.handle, to_gui_string(&text));
        }
    }
}

fn to_gui_string(s: &str) -> imp::String {
    let text_len = s.len().try_into().expect("usize does not fit in u64");
    imp::String {
        data: s.as_ptr(),
        len: text_len,
    }
}

#[derive(Error, Debug)]
enum GuiStringParseError {
    #[error("Invalid string length")]
    InvalidLen,
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
}

fn parse_gui_string(s: &imp::String) -> Result<&str, GuiStringParseError> {
    unsafe {
        let len = s
            .len
            .try_into()
            .map_err(|_| GuiStringParseError::InvalidLen)?;
        let slice = std::slice::from_raw_parts(s.data, len);
        std::str::from_utf8(slice).map_err(GuiStringParseError::Utf8Error)
    }
}

struct GuiData {
    handle: Arc<ImpHandle>,
    tx: Sender<Request>,
}

pub(crate) fn run(tx: Sender<Request>, voices: &[&str]) -> GuiHandle {
    let gui_voices = voices.iter().map(|s| to_gui_string(*s)).collect::<Vec<_>>();

    let handle = unsafe {
        imp::MakeGui(
            imp::GuiCallbacks {
                start_tts_loop: Some(start_tts_loop),
                set_voice: Some(set_voice),
                enable_audio: Some(enable_audio),
                cancel: Some(cancel),
                save: Some(save),
                start_recording: Some(start_recording),
                end_recording: Some(end_recording),
            },
            gui_voices.as_ptr(),
            gui_voices
                .len()
                .try_into()
                .expect("usize does not fit in u64"),
        )
    };

    let handle = Arc::new(ImpHandle { handle });
    let thread_handle = Arc::clone(&handle);

    std::thread::spawn(move || {
        let handle = thread_handle;
        let gui_data = GuiData { handle, tx };
        unsafe {
            imp::Exec(
                **gui_data.handle,
                &gui_data as *const GuiData as *const c_void,
            )
        };
        let _ = gui_data.tx.send(Request::Shutdown);
    });

    GuiHandle { handle }
}

unsafe fn data_to_inner(data: *const c_void) -> &'static GuiData {
    let data = data as *const GuiData;
    &*data
}

unsafe extern "C" fn start_tts_loop(text: imp::String, num_iters: i32, data: *const c_void) {
    let data = data_to_inner(data);

    let text = match parse_gui_string(&text) {
        Ok(s) => s,
        Err(e) => {
            let err = format!("Invalid gui string: {}", e);
            error!("{}", err);
            return;
        }
    };

    let _ = data.tx.send(Request::TtsLoop {
        text: text.to_string(),
        num_iters,
    });
}

unsafe extern "C" fn cancel(data: *const c_void) {
    let data = data_to_inner(data);
    let _ = data.tx.send(Request::Cancel);
}

unsafe extern "C" fn set_voice(voice: imp::String, data: *const c_void) {
    let data = data_to_inner(data);
    let voice = match parse_gui_string(&voice) {
        Ok(s) => s,
        Err(e) => {
            error!("Invalid gui string: {}", e);
            return;
        }
    };
    let _ = data.tx.send(Request::SetVoice {
        voice: voice.to_string(),
    });
}

unsafe extern "C" fn enable_audio(enable: bool, data: *const c_void) {
    let data = data_to_inner(data);
    let _ = data.tx.send(Request::EnableAudio { enable });
}

unsafe extern "C" fn save(path: imp::String, data: *const c_void) {
    let data = data_to_inner(data);

    let path = match parse_gui_string(&path) {
        Ok(s) => s,
        Err(e) => {
            error!("Invalid gui string: {}", e);
            return;
        }
    };

    let _ = data.tx.send(Request::Save { path: path.into() });
}

unsafe extern "C" fn start_recording(data: *const c_void) {
    let data = data_to_inner(data);
    let _ = data.tx.send(Request::StartRecording);
}

unsafe extern "C" fn end_recording(data: *const c_void) {
    let data = data_to_inner(data);
    let _ = data.tx.send(Request::EndRecording);
}
