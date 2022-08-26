use log::{Metadata, Level, Record};

pub struct FilterLogger;

impl log::Log for FilterLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
       true
    }
  
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("Rust says: {} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {

    }
}