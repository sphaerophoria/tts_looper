use log::{error, warn};
use std::convert::TryInto;
use std::ffi::c_void;
use std::sync::mpsc::Sender;
use std::sync::Arc;

mod imp {
    #![allow(non_snake_case)]
    #![allow(non_upper_case_globals)]
    #![allow(unused)]
    #![allow(non_camel_case_types)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub enum GuiRequest {
    Start {
        text: String,
        num_iters: i32,
        play_audio: bool,
    },
    Shutdown,
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
        let text_len = text.len().try_into().expect("usize does not fit in u64");
        unsafe {
            imp::PushOutput(**self.handle, text.as_ptr(), text_len);
        }
    }

    pub fn reset_output(&self) {
        unsafe {
            imp::ResetOutput(**self.handle);
        }
    }
}

pub fn run(tx: Sender<GuiRequest>) -> GuiHandle {
    let handle = unsafe {
        imp::MakeGui(imp::GuiCallbacks {
            start_tts_loop: Some(start_tts_loop),
        })
    };

    let handle = Arc::new(ImpHandle { handle });
    let thread_handle = Arc::clone(&handle);

    std::thread::spawn(move || {
        let handle = thread_handle;
        unsafe { imp::Exec(**handle, &tx as *const Sender<GuiRequest> as *const c_void) };
        if let Err(e) = tx.send(GuiRequest::Shutdown) {
            warn!("Failed to send shutdown notification: {:?}", e);
        }
    });

    GuiHandle { handle }
}

unsafe fn data_to_inner(data: *const c_void) -> &'static Sender<GuiRequest> {
    let data = data as *const Sender<GuiRequest>;
    &*data
}

unsafe extern "C" fn start_tts_loop(
    text: *const u8,
    len: u64,
    num_iters: i32,
    play: bool,
    data: *const c_void,
) {
    let tx = data_to_inner(data);
    let len = match len.try_into() {
        Ok(len) => len,
        Err(e) => {
            error!("Could not convert {} to usize: {}", len, e);
            return;
        }
    };

    let slice = std::slice::from_raw_parts(text, len);
    let s = match std::str::from_utf8(slice) {
        Ok(s) => s,
        Err(e) => {
            error!("Invalid utf8 input: {}", e);
            return;
        }
    };

    let res = tx.send(GuiRequest::Start {
        text: s.to_string(),
        num_iters,
        play_audio: play,
    });

    if let Err(e) = res {
        error!("Cannot start tts loop: {:?}", e);
        return;
    }
}
