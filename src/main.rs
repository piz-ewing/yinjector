use std::sync::mpsc::channel;

use injector::Injector;
use log::*;

mod injector;

fn injector() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        let _ = Injector::new().build(None)?.watch()?;
    } else {
        let _ = Injector::new().build(Some(args[1].clone()))?.watch()?;
    }
    Ok(())
}

fn pause() {
    let (tx, rx) = channel();

    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    println!("Waiting for Ctrl-C...");
    rx.recv().expect("Could not receive from channel.");
    println!("Got it! Exiting...");
}

fn main() {
    pretty_env_logger_custom::formatted_builder_raw()
        .filter_level(LevelFilter::Info)
        .init();

    match injector() {
        Ok(_) => (),
        Err(c) => {
            error!("{}", c);
            pause()
        }
    }
}
