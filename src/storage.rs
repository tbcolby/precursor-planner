//! PDDB storage for Day Planner.
//!
//! Dictionary: planner.data
//! Keys:
//!   events   — JSON array of all Event structs
//!   tasks    — JSON array of all Task structs
//!   next_id  — next unique ID counter

extern crate alloc;
use alloc::vec::Vec;

use crate::planner::{Event, Task};

const DICT: &str = "planner.data";
const KEY_EVENTS: &str = "events";
const KEY_TASKS: &str = "tasks";
const KEY_NEXT_ID: &str = "next_id";

pub struct Storage {
    pddb: pddb::Pddb,
}

impl Storage {
    pub fn new() -> Result<Self, ()> {
        let pddb = pddb::Pddb::new();
        pddb.is_mounted_blocking();
        Ok(Self { pddb })
    }

    fn read_key(&mut self, key: &str) -> Option<Vec<u8>> {
        let mut handle = self
            .pddb
            .get(DICT, key, None, false, false, None, None::<fn()>)
            .ok()?;
        let mut buf = Vec::new();
        use std::io::Read;
        handle.read_to_end(&mut buf).ok()?;
        if buf.is_empty() {
            None
        } else {
            Some(buf)
        }
    }

    fn write_key(&mut self, key: &str, data: &[u8]) {
        if let Ok(mut handle) = self.pddb.get(
            DICT, key, None, true, true, Some(data.len()), None::<fn()>,
        ) {
            use std::io::{Seek, Write};
            handle.seek(std::io::SeekFrom::Start(0)).ok();
            handle.write_all(data).ok();
            handle.set_len(data.len() as u64).ok();
        }
        self.pddb.sync().ok();
    }

    pub fn load_events(&mut self) -> Vec<Event> {
        self.read_key(KEY_EVENTS)
            .and_then(|buf| serde_json::from_slice(&buf).ok())
            .unwrap_or_default()
    }

    pub fn save_events(&mut self, events: &[Event]) {
        let data = serde_json::to_vec(events).unwrap_or_default();
        self.write_key(KEY_EVENTS, &data);
    }

    pub fn load_tasks(&mut self) -> Vec<Task> {
        self.read_key(KEY_TASKS)
            .and_then(|buf| serde_json::from_slice(&buf).ok())
            .unwrap_or_default()
    }

    pub fn save_tasks(&mut self, tasks: &[Task]) {
        let data = serde_json::to_vec(tasks).unwrap_or_default();
        self.write_key(KEY_TASKS, &data);
    }

    pub fn load_next_id(&mut self) -> u32 {
        self.read_key(KEY_NEXT_ID)
            .and_then(|buf| {
                let s = core::str::from_utf8(&buf).ok()?;
                s.trim().parse::<u32>().ok()
            })
            .unwrap_or(1)
    }

    pub fn save_next_id(&mut self, id: u32) {
        let data = alloc::format!("{}", id);
        self.write_key(KEY_NEXT_ID, data.as_bytes());
    }
}
