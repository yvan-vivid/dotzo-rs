use chrono::Local;
use colored::Colorize;
use fern::Dispatch;
use log::{Level, LevelFilter, SetLoggerError};
use std::io::stderr;

pub fn setup_logging() -> Result<(), SetLoggerError> {
    Dispatch::new()
        .format(|out, message, record| {
            let color = match record.level() {
                Level::Error => "red",
                Level::Warn => "yellow",
                Level::Info => "green",
                Level::Debug => "blue",
                Level::Trace => "magenta",
            };
            out.finish(format_args!(
                "{} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level().to_string().color(color),
                message
            ))
        })
        .level(LevelFilter::Info)
        .chain(stderr())
        .apply()
}
