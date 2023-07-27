use ic_logger::IcLogger;

fn main() {
    IcLogger::new().init().unwrap();

    log::warn!("This is an example message.");
}
