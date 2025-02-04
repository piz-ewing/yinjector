use log::info;
use std::cell::RefCell;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use ferrisetw::parser::Parser;
use ferrisetw::provider::Provider;
use ferrisetw::schema_locator::SchemaLocator;
use ferrisetw::trace::UserTrace;
use ferrisetw::trace::*;
use ferrisetw::EventRecord;

use super::util;

#[derive(Clone)]
pub enum Event {
    ProcessStart(u32, String),
    ProcessStop(u32, String),

    ImageLoad(u32, String),
    // ImageUnload(u32),
    GUIProcessStart(u32),
    // GUIProcessStop(u32),
}

pub trait Listener {
    fn trigger(&mut self, _: Event);
}

type ListenerBox = Box<dyn Listener + 'static + Send + Sync>;
type AListeners = Arc<Mutex<Vec<ListenerBox>>>;

#[derive(Default)]
pub struct Monitor {
    listeners: AListeners, // Arc Mutext Option/Vec
    user_trace: RefCell<Option<UserTrace>>,
}

impl Monitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self, r: ListenerBox) {
        self.listeners.lock().unwrap().push(r)
    }

    pub fn start(&self) {
        let listeners_clone = self.listeners.clone();
        let process_provider = Provider::by_guid("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716") // Microsoft-Windows-Kernel-Process
            .add_callback(move |record, schema_locator| {
                process_callback(record, schema_locator, &listeners_clone)
            })
            .build();

        let listeners_clone = self.listeners.clone();
        let win32k_provider = Provider::by_guid("8c416c79-d49b-4f01-a467-e56d3aa8234c") // Microsoft-Windows-Win32k
            .add_callback(move |record, schema_locator| {
                win32k_callback(record, schema_locator, &listeners_clone)
            })
            .build();

        // FIXME: Replay fake event, Events before etw takes effect may be lost.
        let listeners_clone = self.listeners.clone();
        util::enum_process(|process_id, file_name| {
            notify(
                &listeners_clone,
                Event::ProcessStart(process_id, file_name.to_lowercase()),
            );

            util::enum_module(process_id, |module_name| -> bool {
                let module_name = module_name.to_lowercase();

                if module_name == "user32.dll" {
                    notify(&listeners_clone, Event::GUIProcessStart(process_id));
                }
                notify(&listeners_clone, Event::ImageLoad(process_id, module_name));
                true
            });
        });

        let _ = stop_trace_by_name("YInjectorMonitor");

        *self.user_trace.borrow_mut() = Some(
            UserTrace::new()
                .named(String::from("YInjectorMonitor"))
                .enable(process_provider)
                .enable(win32k_provider)
                .start_and_process()
                .unwrap(),
        );
    }

    pub fn stop(&self) {
        if let Some(user_trace) = self.user_trace.borrow_mut().take() {
            let _ = user_trace.stop();
        }
    }
}

fn notify(listeners: &AListeners, event: Event) {
    for l in listeners.lock().unwrap().iter_mut() {
        l.trigger(event.clone());
    }
}

fn process_callback(record: &EventRecord, schema_locator: &SchemaLocator, listeners: &AListeners) {
    // https://github.com/repnz/etw-providers-docs/blob/master/Manifests-Win10-17134/Microsoft-Windows-Kernel-Process.xml
    // https://github.com/everdox/InfinityHook?tab=readme-ov-file
    match schema_locator.event_schema(record) {
        Ok(schema) => match record.event_id() {
            // ProcessStart
            1 => {
                let parser = Parser::create(record, &schema);
                let process_id: u32 = parser.try_parse("ProcessID").unwrap();
                // let parent_process_id: u32 = parser.try_parse("ParentProcessID").unwrap();
                let image_name: String = parser.try_parse("ImageName").unwrap();

                let path = Path::new(&image_name);
                let file_name = path.file_name().unwrap().to_str().unwrap().to_lowercase();

                notify(listeners, Event::ProcessStart(process_id, file_name));
            }
            2 => {
                let parser = Parser::create(record, &schema);
                let process_id: u32 = parser.try_parse("ProcessID").unwrap();
                // let exit_code: u32 = parser.try_parse("ExitCode").unwrap();
                let image_name: String = parser.try_parse("ImageName").unwrap();

                let path = Path::new(&image_name);
                let file_name = path.file_name().unwrap().to_str().unwrap().to_lowercase();

                notify(listeners, Event::ProcessStop(process_id, file_name));
            }
            // ImageLoad
            5 => {
                let parser = Parser::create(record, &schema);
                let process_id: u32 = parser.try_parse("ProcessID").unwrap();
                let image_name: String = parser.try_parse("ImageName").unwrap();
                // let image_base: u64 = parser.try_parse("ImageBase").unwrap();

                let path = Path::new(&image_name);
                let filename = path.file_name().unwrap().to_str().unwrap().to_lowercase();

                notify(listeners, Event::ImageLoad(process_id, filename));
            }
            // ImageUnload
            // 6 => {}
            _ => {}
        },
        Err(err) => info!("Error {:?}", err),
    };
}

fn win32k_callback(record: &EventRecord, schema_locator: &SchemaLocator, listeners: &AListeners) {
    // https://github.com/jdu2600/Windows10EtwEvents/blob/master/manifest/Microsoft-Windows-Win32k.tsv
    match schema_locator.event_schema(record) {
        Ok(_schema) => {
            let id = record.event_id();

            match id {
                // GUIProcessStart
                84 => {
                    notify(listeners, Event::GUIProcessStart(record.process_id()));
                }
                // GUIProcessStop
                // 85 => {
                //     notify(listeners, Event::GUIProcessStop(record.process_id()));
                // }
                _ => {
                    // info!("==> {}", id)
                }
            }
        }
        Err(err) => info!("Error {:?}", err),
    }
}
