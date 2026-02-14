//! Data models for the Day Planner.
//!
//! Events have a date, optional time, and description.
//! Tasks have a description and done/not-done status.
//! Dates are stored as (year, month, day) tuples — no floating point needed.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use serde::{Deserialize, Serialize};

/// A calendar date (year, month, day).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl Date {
    pub fn new(year: u16, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    pub fn display(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    pub fn short_display(&self) -> String {
        format!("{:02}/{:02}", self.month, self.day)
    }

    /// Days in the given month (handles leap years).
    pub fn days_in_month(year: u16, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    /// Advance by one day.
    pub fn next_day(&self) -> Date {
        let dim = Date::days_in_month(self.year, self.month);
        if self.day < dim {
            Date::new(self.year, self.month, self.day + 1)
        } else if self.month < 12 {
            Date::new(self.year, self.month + 1, 1)
        } else {
            Date::new(self.year + 1, 1, 1)
        }
    }

    /// Go back one day.
    pub fn prev_day(&self) -> Date {
        if self.day > 1 {
            Date::new(self.year, self.month, self.day - 1)
        } else if self.month > 1 {
            let prev_month = self.month - 1;
            let dim = Date::days_in_month(self.year, prev_month);
            Date::new(self.year, prev_month, dim)
        } else {
            Date::new(self.year - 1, 12, 31)
        }
    }

    /// Day of week (0=Sunday, 6=Saturday) — Zeller's formula.
    pub fn weekday(&self) -> u8 {
        let mut y = self.year as i32;
        let mut m = self.month as i32;
        if m < 3 {
            m += 12;
            y -= 1;
        }
        let q = self.day as i32;
        let k = y % 100;
        let j = y / 100;
        let h = (q + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 - 2 * j) % 7;
        ((h + 7) % 7) as u8 // 0=Sat in Zeller, convert: (h+6)%7 for 0=Mon... actually:
        // Zeller: 0=Sat, 1=Sun, 2=Mon... Let's convert to 0=Sun:
        // (h + 6) % 7 gives 0=Sun
    }

    /// Day of week with 0=Sunday, using Tomohiko Sakamoto's algorithm.
    pub fn day_of_week(&self) -> u8 {
        let t = [0i32, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let mut y = self.year as i32;
        if self.month < 3 {
            y -= 1;
        }
        let dow = (y + y / 4 - y / 100 + y / 400 + t[(self.month - 1) as usize] + self.day as i32) % 7;
        dow as u8 // 0=Sunday
    }

    pub fn weekday_name(&self) -> &'static str {
        match self.day_of_week() {
            0 => "Sun",
            1 => "Mon",
            2 => "Tue",
            3 => "Wed",
            4 => "Thu",
            5 => "Fri",
            6 => "Sat",
            _ => "???",
        }
    }

    pub fn month_name(month: u8) -> &'static str {
        match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "???",
        }
    }
}

/// A time of day (hour, minute) in 24h format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
}

impl Time {
    pub fn new(hour: u8, minute: u8) -> Self {
        Self {
            hour: hour.min(23),
            minute: minute.min(59),
        }
    }

    pub fn display(&self) -> String {
        let (h12, ampm) = if self.hour == 0 {
            (12, "AM")
        } else if self.hour < 12 {
            (self.hour, "AM")
        } else if self.hour == 12 {
            (12, "PM")
        } else {
            (self.hour - 12, "PM")
        };
        format!("{}:{:02}{}", h12, self.minute, ampm)
    }
}

/// Priority level for events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
}

impl Priority {
    pub fn label(&self) -> &'static str {
        match self {
            Priority::Low => "Low",
            Priority::Normal => "Normal",
            Priority::High => "High",
        }
    }

    pub fn marker(&self) -> &'static str {
        match self {
            Priority::Low => " ",
            Priority::Normal => "*",
            Priority::High => "!",
        }
    }

    pub fn cycle(&self) -> Priority {
        match self {
            Priority::Low => Priority::Normal,
            Priority::Normal => Priority::High,
            Priority::High => Priority::Low,
        }
    }
}

/// A scheduled event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: u32,
    pub date: Date,
    pub time: Option<Time>,
    pub title: String,
    pub priority: Priority,
}

impl Event {
    pub fn new(id: u32, date: Date, title: String) -> Self {
        Self {
            id,
            date,
            time: None,
            title,
            priority: Priority::Normal,
        }
    }

    pub fn time_display(&self) -> String {
        match &self.time {
            Some(t) => t.display(),
            None => String::from("All day"),
        }
    }
}

/// A task/to-do item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub done: bool,
    pub priority: Priority,
}

impl Task {
    pub fn new(id: u32, title: String) -> Self {
        Self {
            id,
            title,
            done: false,
            priority: Priority::Normal,
        }
    }
}

/// Sort events by time (all-day first, then by hour:minute).
pub fn sort_events(events: &mut Vec<Event>) {
    events.sort_by(|a, b| {
        let time_a = a.time.map(|t| (t.hour as u16) * 60 + t.minute as u16).unwrap_or(0);
        let time_b = b.time.map(|t| (t.hour as u16) * 60 + t.minute as u16).unwrap_or(0);
        let has_time_a = a.time.is_some() as u8;
        let has_time_b = b.time.is_some() as u8;
        has_time_a.cmp(&has_time_b).then(time_a.cmp(&time_b))
    });
}

/// Sort tasks: incomplete first, then by priority (high first).
pub fn sort_tasks(tasks: &mut Vec<Task>) {
    tasks.sort_by(|a, b| {
        a.done.cmp(&b.done).then_with(|| {
            let pa = match a.priority {
                Priority::High => 0,
                Priority::Normal => 1,
                Priority::Low => 2,
            };
            let pb = match b.priority {
                Priority::High => 0,
                Priority::Normal => 1,
                Priority::Low => 2,
            };
            pa.cmp(&pb)
        })
    });
}
