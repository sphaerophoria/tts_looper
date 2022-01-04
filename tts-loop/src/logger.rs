use crate::{gui::GuiHandle, TtsLooper};

use env_logger::Logger;
use log::{LevelFilter, Log, Metadata, Record};
use once_cell::sync::OnceCell;

use std::sync::{Arc, Weak};

static LOGGER: OnceCell<TtsLogger> = OnceCell::new();

pub(crate) struct TtsLogger {
    gui: Weak<GuiHandle>,
    logger: Logger,
}

impl TtsLogger {
    pub(crate) fn new(gui: Weak<GuiHandle>) -> TtsLogger {
        let logger = env_logger::Builder::from_default_env()
            .filter(Some("tts_loop"), LevelFilter::Info)
            .build();

        TtsLogger { gui, logger }
    }
}

impl Log for TtsLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.logger.enabled(metadata)
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            let log = format!("{}", record.args());
            if let Some(gui) = self.gui.upgrade() {
                gui.log(&log, record.level());
            }
        }
        self.logger.log(record);
    }

    fn flush(&self) {
        self.logger.flush()
    }
}

pub fn init_logger(looper: &TtsLooper) {
    let looper = LOGGER.set(TtsLogger::new(Arc::downgrade(&looper.gui)));
    looper.map_err(|_| "").expect("Failed to construct logger");
    log::set_logger(LOGGER.get().unwrap()).unwrap();
    log::set_max_level(LevelFilter::Debug);
}
