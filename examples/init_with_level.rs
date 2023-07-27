use log::LevelFilter;
use ic_logger::IcLogger;

fn main() {
    IcLogger::new().with_level(LevelFilter::Warn).init().unwrap();

    log::warn!("This will be logged.");
    log::info!("This will NOT be logged.");
}
