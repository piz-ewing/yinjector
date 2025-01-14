use log::*;
mod core;

use std::sync::mpsc::channel;

pub fn pause() {
    let (tx, rx) = channel();

    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    info!("[+] Monitor Start, waiting for Ctrl-C...");
    rx.recv().expect("Could not receive from channel.");
}

fn main() {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("[*] Welcome to yinjector");

    let args: Vec<_> = std::env::args().collect();
    let cfg = if args.len() < 2 {
        None
    } else {
        Some(args[1].as_str())
    };

    core::builder().start(cfg).wait_until(|| pause());
}
