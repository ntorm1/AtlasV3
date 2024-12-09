use crate::error::AtlasError;
use env_logger::Builder;
use log::{error, info, LevelFilter};
use std::env;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

//==========================================================================
pub struct AtlasUtil {}

static LOGGER_INITIALIZED: AtomicBool = AtomicBool::new(false);

//==========================================================================
impl AtlasUtil {
    //==========================================================================
    pub fn setup_logger() -> Result<(), AtlasError> {
        if LOGGER_INITIALIZED.load(Ordering::Relaxed) {
            return Ok(());
        }
        LOGGER_INITIALIZED.store(true, Ordering::Relaxed);
        let mut builder = Builder::new();
        builder.filter(None, LevelFilter::Info);
        builder.format(|buf, record| {
            use chrono::Local;
            let now = Local::now();
            let file = record.file().unwrap_or("<unknown>");
            let line = record.line().unwrap_or(0);
            writeln!(
                buf,
                "{} [{} {}:{}] {}: {}",
                now.format("[%H:%M:%S.%3f]"),
                record.level(),
                file,
                line,
                record.target(),
                record.args()
            )
        });
        builder.try_init()?;
        Ok(())
    }
}
