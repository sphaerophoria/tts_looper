use cpal::traits::*;
use cpal::{SampleFormat, SampleRate, Stream};
use thiserror::Error as ThisError;

use std::sync::mpsc;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("No input device available")]
    NoInputDevice,
    #[error("Error retrieving supported device configurations")]
    SupportedStreamConfigsError,
    #[error("Could not find config for requested sample rate")]
    UnsupportedSampleRate,
    #[error(transparent)]
    DeviceError(#[from] cpal::DevicesError),
    #[error(transparent)]
    BuildStreamError(#[from] cpal::BuildStreamError),
}

pub(crate) struct AudioManager {
    host: cpal::Host,
}

impl AudioManager {
    pub(crate) fn new() -> AudioManager {
        let host = cpal::default_host();
        AudioManager { host }
    }

    pub(crate) fn input_stream<F: Fn(&[i16]) + Send + 'static>(
        &self,
        sample_rate: u32,
        input_callback: F,
    ) -> Result<Stream, Error> {
        let input_dev = self
            .host
            .default_input_device()
            .ok_or(Error::NoInputDevice)?;

        let supported_configs = input_dev
            .supported_input_configs()
            .map_err(|_| Error::SupportedStreamConfigsError)?;

        let supported_config = supported_configs
            .into_iter()
            .find(|item| {
                item.max_sample_rate().0 > sample_rate
                    && item.min_sample_rate().0 < sample_rate
                    && item.sample_format() == SampleFormat::I16
                    && item.channels() == 1
            })
            .ok_or(Error::UnsupportedSampleRate)?
            .with_sample_rate(SampleRate(sample_rate));

        let stream = input_dev.build_input_stream(
            &supported_config.config(),
            move |samples, _| input_callback(samples),
            |_err| (),
        )?;

        Ok(stream)
    }

    pub(crate) fn output_stream<F: FnMut(&mut [i16]) + Send + 'static>(
        &self,
        sample_rate: u32,
        mut output_callback: F,
    ) -> Result<Stream, Error> {
        let output_dev = self
            .host
            .default_output_device()
            .ok_or(Error::NoInputDevice)?;

        let supported_configs = output_dev
            .supported_output_configs()
            .map_err(|_| Error::SupportedStreamConfigsError)?;

        let supported_config = supported_configs
            .into_iter()
            .find(|item| {
                item.max_sample_rate().0 > sample_rate
                    && item.min_sample_rate().0 < sample_rate
                    && item.sample_format() == SampleFormat::I16
                    && item.channels() == 1
            })
            .ok_or(Error::UnsupportedSampleRate)?
            .with_sample_rate(SampleRate(sample_rate));

        let stream = output_dev.build_output_stream(
            &supported_config.config(),
            move |samples, _| output_callback(samples),
            |_err| (),
        )?;

        Ok(stream)
    }

    pub(crate) fn play_buf_blocking(&self, buf: &[i16], sample_rate: u32) -> Result<(), Error> {
        let input_buf = buf.to_owned();
        let mut buf_pos = 0usize;

        let (tx, rx) = mpsc::channel();

        let _output_stream = self.output_stream(sample_rate, move |output_buf| {
            if buf_pos >= input_buf.len() {
                let _ = tx.send(());
                return;
            }

            let expected_size = output_buf.len();
            let mut end_pos = buf_pos + expected_size;
            if end_pos > input_buf.len() {
                end_pos = input_buf.len();
            }

            output_buf[..end_pos - buf_pos].copy_from_slice(&input_buf[buf_pos..end_pos]);
            buf_pos = end_pos;
        })?;

        let _ = rx.recv();

        Ok(())
    }
}
