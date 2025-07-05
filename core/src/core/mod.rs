mod config;
mod controller;
mod executor;
mod monitor;
mod util;

pub fn builder() -> executor::Injector {
    executor::Injector::new()
}
