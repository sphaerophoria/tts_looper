use tts_loop::TtsLooper;
use std::ffi::c_void;
use std::cell::{RefCell, RefMut};
use std::convert::TryInto;
use std::sync::mpsc::{self, Sender};

mod imp {
    #![allow(non_snake_case)]
    #![allow(non_upper_case_globals)]
    #![allow(unused)]
    #![allow(non_camel_case_types)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

type TtsRequest = (String, i32, bool);

struct GuiInner {
    tx: Sender<TtsRequest>,
}

#[derive(Copy, Clone)]
struct ImpHandle {
    handle: *mut imp::Gui
}

impl std::ops::Deref for ImpHandle {
    type Target = *mut imp::Gui;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}


unsafe impl Send for ImpHandle {}
unsafe impl Sync for ImpHandle {}



pub struct Gui {
    handle: ImpHandle,
    _inner: Box<RefCell<GuiInner>>,
}

impl Gui {
    pub fn new(mut tts_looper: TtsLooper) -> Gui {
        let (tx, rx) = mpsc::channel();

        let inner = GuiInner {
            tx,
        };

        let mut inner = Box::new(RefCell::new(inner));


        let handle = unsafe {
            imp::MakeGui(imp::GuiCallbacks {
                data: inner.as_mut() as *mut RefCell<GuiInner> as *mut c_void,
                start_tts_loop: Some(start_tts_loop),
            })
        };

        let handle = ImpHandle { handle };

        std::thread::spawn(move || {
            while let Ok((mut text, num_iters, play)) = rx.recv() {
                unsafe { imp::ResetOutput(*handle); }
                for _ in 0..num_iters {
                    let buf = tts_looper.text_to_speech(text);
                    if play {
                        tts_looper.play_buf(&buf);
                    }
                    text = tts_looper.speech_to_text(&*buf).unwrap();
                    unsafe {
                        imp::PushOutput(*handle, text.as_ptr(), text.len().try_into().unwrap());
                    }
                }
            }
        });

        Gui {
            handle,
            _inner: inner,
        }
    }

    pub fn exec(&mut self) {
        unsafe { imp::Exec(*self.handle) };
    }
}

impl Drop for Gui {
    fn drop(&mut self) {
        unsafe { imp::DestroyGui(*self.handle) };
    }
}

unsafe fn data_to_inner(data: *const c_void) -> RefMut<'static, GuiInner> {
    let data = data as *const RefCell<GuiInner>;
    (*data).borrow_mut()
}

unsafe extern "C" fn start_tts_loop(text: *const u8, len: u64, num_iters: i32, play: bool, data: *const c_void) {
    let inner = data_to_inner(data);
    let slice = std::slice::from_raw_parts(text, len.try_into().unwrap());
    // FIXME: evil unwrap
    let s = std::str::from_utf8(slice).unwrap();
    inner.tx.send((s.to_string(), num_iters, play)).unwrap();
}
