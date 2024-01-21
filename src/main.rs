use log::*;
use std::sync::mpsc::channel;

mod core;
use crate::core::Injector;

fn injector() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        Injector::build().watch()?;
    } else {
        Injector::build()
            .config_path(Some(args[1].clone()))
            .watch()?;
    }
    Ok(())
}

fn pause() {
    let (tx, rx) = channel();

    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    info!("[+] waiting for Ctrl-C...");
    rx.recv().expect("Could not receive from channel.");
}

fn main() {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("[*] Welcome to injector");
    match injector() {
        Ok(_) => (),
        Err(c) => {
            error!("[!] {}", c);
            pause()
        }
    }
}
