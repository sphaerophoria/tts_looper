use std::ffi::{CStr, CString, NulError};

pub struct FliteWav {
    wav: *mut flite_sys::cst_wave,
}

impl FliteWav {
    fn new(wav: *mut flite_sys::cst_wave) -> FliteWav {
        FliteWav { wav }
    }

    pub fn sample_rate(&self) -> i32 {
        unsafe { (*self.wav).sample_rate }
    }

    pub fn num_channels(&self) -> i32 {
        unsafe { (*self.wav).num_channels }
    }
}

impl Drop for FliteWav {
    fn drop(&mut self) {
        unsafe {
            flite_sys::delete_wave(self.wav);
        }
    }
}

impl std::ops::Deref for FliteWav {
    type Target = [i16];

    fn deref(&self) -> &Self::Target {
        unsafe {
            let len = (*self.wav).num_samples * (*self.wav).num_channels;
            std::slice::from_raw_parts((*self.wav).samples, len as usize)
        }
    }
}

unsafe impl Send for FliteWav {}
unsafe impl Sync for FliteWav {}

static FLATE_INIT: std::sync::Once = std::sync::Once::new();

fn flite_init() {
    unsafe {
        flite_sys::flite_init();
        flite_sys::flite_set_lang_list();
        flite_sys::flite_set_voice_list(std::ptr::null());
    }
}

pub fn list_voices() -> Vec<&'static str> {
    FLATE_INIT.call_once(flite_init);

    unsafe {
        let mut it = flite_sys::flite_voice_list as *const flite_sys::cst_val;
        let mut ret = Vec::new();

        while it != std::ptr::null() {
            let voice = flite_sys::val_voice(flite_sys::val_car(it));
            let name =CStr::from_ptr((*voice).name);
            ret.push(name.to_str().expect("Invalid voice name"));
            it = flite_sys::val_cdr(it);
        }

        ret
    }
}

pub fn text_to_wave<S: Into<Vec<u8>>>(text: S, sample_rate: i32, voice: String) -> Result<FliteWav, NulError> {
    FLATE_INIT.call_once(flite_init);

    let wav = unsafe {
        let voice = CString::new(voice)?;
        let voice = flite_sys::flite_voice_select(voice.as_ptr());
        let text = CString::new(text)?;
        let wav = flite_sys::flite_text_to_wave(text.as_ptr(), voice);
        flite_sys::cst_wave_resample(wav, sample_rate);
        wav
    };

    Ok(FliteWav::new(wav))
}
