use std::ffi::CString;

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

static FLATE_INIT: std::sync::Once = std::sync::Once::new();

pub fn text_to_wave<S: Into<Vec<u8>>>(text: S, sample_rate: i32) -> FliteWav {
    FLATE_INIT.call_once(|| unsafe {
        flite_sys::flite_init();
        flite_sys::flite_set_lang_list();
        flite_sys::flite_set_voice_list(std::ptr::null());
    });

    let wav = unsafe {
        let voice = flite_sys::flite_voice_select(std::ptr::null());
        let text = CString::new(text).unwrap();
        let wav = flite_sys::flite_text_to_wave(text.as_ptr(), voice);
        flite_sys::cst_wave_resample(wav, sample_rate);
        wav
    };

    FliteWav::new(wav)
}
