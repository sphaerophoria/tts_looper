use log::error;
use std::convert::TryInto;
use std::ffi::c_void;
use std::sync::Arc;
use thiserror::Error;

use crate::channel::{Request, Sender};

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

pub struct GuiHandle {
    handle: Arc<ImpHandle>,
}

impl GuiHandle {
    pub fn push_output(&self, text: &str) {
        unsafe {
            imp::PushOutput(**self.handle, to_gui_string(text));
        }
    }

    pub fn push_loop_start(&self, text: &str, voice: &str, num_iters: i32) {
        unsafe {
            imp::PushLoopStart(
                **self.handle,
                to_gui_string(text),
                to_gui_string(voice),
                num_iters,
            );
        }
    }

    pub fn push_error(&self, error: &str) {
        unsafe {
            imp::PushError(**self.handle, to_gui_string(error));
        }
    }

    pub fn push_cancel(&self) {
        unsafe {
            imp::PushCancel(**self.handle);
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
    tx: Sender,
}

pub fn run(tx: Sender, voices: &[&str]) -> GuiHandle {
    let gui_voices = voices.iter().map(|s| to_gui_string(*s)).collect::<Vec<_>>();

    let handle = unsafe {
        imp::MakeGui(
            imp::GuiCallbacks {
                start_tts_loop: Some(start_tts_loop),
                set_voice: Some(set_voice),
                enable_audio: Some(enable_audio),
                cancel: Some(cancel),
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
        gui_data.tx.send(Request::Shutdown);
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
            imp::PushError(**data.handle, to_gui_string(&err));
            return;
        }
    };

    data.tx.send(Request::Cancel);
    data.tx.send(Request::SetText {
        text: text.to_string(),
    });
    data.tx.send(Request::LogStart { num_iters });
    for _ in 0..num_iters {
        data.tx.send(Request::RunTts);
        data.tx.send(Request::PlayAudio);
        data.tx.send(Request::RunStt);
    }
}

unsafe extern "C" fn cancel(data: *const c_void) {
    let data = data_to_inner(data);
    data.tx.send(Request::Cancel);
}

unsafe extern "C" fn set_voice(voice: imp::String, data: *const c_void) {
    let data = data_to_inner(data);
    let voice = match parse_gui_string(&voice) {
        Ok(s) => s,
        Err(e) => {
            let err = format!("Invalid gui string: {}", e);
            error!("{}", err);
            imp::PushError(**data.handle, to_gui_string(&err));
            return;
        }
    };
    data.tx.send(Request::SetVoice {
        voice: voice.to_string(),
    });
}

unsafe extern "C" fn enable_audio(enable: bool, data: *const c_void) {
    let data = data_to_inner(data);
    data.tx.send(Request::EnableAudio { enable });
}
