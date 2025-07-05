use log::info;
use std::cell::RefCell;

use super::controller::Controller;
use super::monitor::Monitor;

#[derive(Default)]
pub struct Injector {
    monitor: RefCell<Option<Monitor>>,
}

impl Injector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&self, path: Option<&str>) -> &Self {
        let m = Monitor::new();
        m.subscribe(Box::new(Controller::build(path)));

        info!("start <<<");

        m.start();

        self.monitor.borrow_mut().replace(m);
        self
    }

    pub fn stop(&self) {
        if let Some(m) = self.monitor.take() {
            m.stop();

            info!("stop <<<");
        }
    }

    pub fn wait_until<T: FnOnce()>(&self, f: T) {
        f()
    }
}

impl Drop for Injector {
    fn drop(&mut self) {
        self.stop();
    }
}
