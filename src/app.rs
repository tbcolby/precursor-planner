//! State machine for Day Planner.
//!
//! States:
//!   DayView     — shows events for the selected date
//!   TaskList    — shows the to-do list
//!   AddEvent    — multi-field form for new event
//!   EditEvent   — edit an existing event
//!   AddTask     — text entry for new task
//!   ConfirmDel  — confirm deletion of event or task
//!   MonthView   — calendar month grid for date picking

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::planner::*;
use crate::storage::Storage;

// Keyboard constants
const KEY_UP: char = '\u{F700}';
const KEY_DOWN: char = '\u{F701}';
const KEY_LEFT: char = '\u{F702}';
const KEY_RIGHT: char = '\u{F703}';
const KEY_ENTER: char = '\u{000D}';
const KEY_BACKSPACE: char = '\u{0008}';
const KEY_MENU: char = '\u{2234}'; // ∴

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    DayView,
    TaskList,
    AddEvent,
    EditEvent,
    AddTask,
    ConfirmDel,
    MonthView,
}

/// Which field is being edited in AddEvent/EditEvent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventField {
    Title,
    Hour,
    Minute,
    Priority,
}

/// What we're about to delete.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeleteTarget {
    Event(u32),
    Task(u32),
}

pub struct PlannerApp {
    pub state: AppState,
    pub needs_redraw: bool,

    // Date navigation
    pub current_date: Date,

    // Events & tasks
    pub events: Vec<Event>,
    pub tasks: Vec<Task>,
    pub next_id: u32,

    // Day view cursor
    pub day_cursor: usize,

    // Task list cursor
    pub task_cursor: usize,

    // Event form fields
    pub form_title: String,
    pub form_hour: u8,
    pub form_minute: u8,
    pub form_has_time: bool,
    pub form_priority: Priority,
    pub form_field: EventField,
    pub editing_event_id: Option<u32>,

    // Task form
    pub task_input: String,

    // Delete confirmation
    pub delete_target: Option<DeleteTarget>,

    // Month view
    pub month_view_year: u16,
    pub month_view_month: u8,
    pub month_cursor_day: u8,

    // Storage
    storage: Option<Storage>,
}

impl PlannerApp {
    pub fn new(initial_date: Date) -> Self {
        Self {
            state: AppState::DayView,
            needs_redraw: true,
            current_date: initial_date,
            events: Vec::new(),
            tasks: Vec::new(),
            next_id: 1,
            day_cursor: 0,
            task_cursor: 0,
            form_title: String::new(),
            form_hour: 9,
            form_minute: 0,
            form_has_time: true,
            form_priority: Priority::Normal,
            form_field: EventField::Title,
            editing_event_id: None,
            task_input: String::new(),
            delete_target: None,
            month_view_year: initial_date.year,
            month_view_month: initial_date.month,
            month_cursor_day: initial_date.day,
            storage: None,
        }
    }

    pub fn init_storage(&mut self) {
        if let Ok(mut st) = Storage::new() {
            self.events = st.load_events();
            self.tasks = st.load_tasks();
            self.next_id = st.load_next_id();
            self.storage = Some(st);
        }
    }

    pub fn save_state(&mut self) {
        if let Some(ref mut st) = self.storage {
            st.save_events(&self.events);
            st.save_tasks(&self.tasks);
            st.save_next_id(self.next_id);
        }
    }

    fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Events for the currently selected date, sorted by time.
    pub fn events_for_date(&self) -> Vec<&Event> {
        let mut day_events: Vec<&Event> = self
            .events
            .iter()
            .filter(|e| e.date == self.current_date)
            .collect();
        day_events.sort_by(|a, b| {
            let ta = a.time.map(|t| (t.hour as u16) * 60 + t.minute as u16).unwrap_or(0);
            let tb = b.time.map(|t| (t.hour as u16) * 60 + t.minute as u16).unwrap_or(0);
            let ha = a.time.is_some() as u8;
            let hb = b.time.is_some() as u8;
            ha.cmp(&hb).then(ta.cmp(&tb))
        });
        day_events
    }

    /// Count events for a given date (for month view dots).
    pub fn event_count_for(&self, date: Date) -> usize {
        self.events.iter().filter(|e| e.date == date).count()
    }

    /// Count incomplete tasks.
    pub fn pending_task_count(&self) -> usize {
        self.tasks.iter().filter(|t| !t.done).count()
    }

    /// handle_key returns true to keep running, false to quit.
    pub fn handle_key(&mut self, key: char) -> bool {
        self.needs_redraw = true;
        match self.state {
            AppState::DayView => self.handle_day_view(key),
            AppState::TaskList => self.handle_task_list(key),
            AppState::AddEvent => self.handle_add_event(key),
            AppState::EditEvent => self.handle_edit_event(key),
            AppState::AddTask => self.handle_add_task(key),
            AppState::ConfirmDel => self.handle_confirm_del(key),
            AppState::MonthView => self.handle_month_view(key),
        }
    }

    fn handle_day_view(&mut self, key: char) -> bool {
        let count = self.events_for_date().len();
        match key {
            KEY_MENU => return false,
            KEY_UP => {
                if count > 0 && self.day_cursor > 0 {
                    self.day_cursor -= 1;
                }
            }
            KEY_DOWN => {
                if count > 0 && self.day_cursor < count - 1 {
                    self.day_cursor += 1;
                }
            }
            KEY_LEFT => {
                self.current_date = self.current_date.prev_day();
                self.day_cursor = 0;
            }
            KEY_RIGHT => {
                self.current_date = self.current_date.next_day();
                self.day_cursor = 0;
            }
            'a' | 'A' => {
                // Add event
                self.form_title.clear();
                self.form_hour = 9;
                self.form_minute = 0;
                self.form_has_time = true;
                self.form_priority = Priority::Normal;
                self.form_field = EventField::Title;
                self.editing_event_id = None;
                self.state = AppState::AddEvent;
            }
            'e' | 'E' => {
                // Edit selected event
                let day_events = self.events_for_date();
                if let Some(ev) = day_events.get(self.day_cursor) {
                    self.form_title = ev.title.clone();
                    self.form_hour = ev.time.map(|t| t.hour).unwrap_or(9);
                    self.form_minute = ev.time.map(|t| t.minute).unwrap_or(0);
                    self.form_has_time = ev.time.is_some();
                    self.form_priority = ev.priority;
                    self.form_field = EventField::Title;
                    self.editing_event_id = Some(ev.id);
                    self.state = AppState::EditEvent;
                }
            }
            'd' | 'D' => {
                // Delete selected event
                let day_events = self.events_for_date();
                if let Some(ev) = day_events.get(self.day_cursor) {
                    self.delete_target = Some(DeleteTarget::Event(ev.id));
                    self.state = AppState::ConfirmDel;
                }
            }
            't' | 'T' => {
                self.task_cursor = 0;
                self.state = AppState::TaskList;
            }
            'm' | 'M' => {
                self.month_view_year = self.current_date.year;
                self.month_view_month = self.current_date.month;
                self.month_cursor_day = self.current_date.day;
                self.state = AppState::MonthView;
            }
            _ => {}
        }
        true
    }

    fn handle_task_list(&mut self, key: char) -> bool {
        let count = self.tasks.len();
        match key {
            KEY_MENU | KEY_LEFT => {
                self.state = AppState::DayView;
            }
            KEY_UP => {
                if count > 0 && self.task_cursor > 0 {
                    self.task_cursor -= 1;
                }
            }
            KEY_DOWN => {
                if count > 0 && self.task_cursor < count - 1 {
                    self.task_cursor += 1;
                }
            }
            KEY_ENTER => {
                // Toggle done
                if self.task_cursor < self.tasks.len() {
                    self.tasks[self.task_cursor].done = !self.tasks[self.task_cursor].done;
                    sort_tasks(&mut self.tasks);
                    self.save_state();
                }
            }
            'a' | 'A' => {
                self.task_input.clear();
                self.state = AppState::AddTask;
            }
            'p' | 'P' => {
                // Cycle priority of selected task
                if self.task_cursor < self.tasks.len() {
                    self.tasks[self.task_cursor].priority =
                        self.tasks[self.task_cursor].priority.cycle();
                    sort_tasks(&mut self.tasks);
                    self.save_state();
                }
            }
            'd' | 'D' => {
                if self.task_cursor < self.tasks.len() {
                    self.delete_target =
                        Some(DeleteTarget::Task(self.tasks[self.task_cursor].id));
                    self.state = AppState::ConfirmDel;
                }
            }
            _ => {}
        }
        true
    }

    fn handle_event_form(&mut self, key: char) -> bool {
        match self.form_field {
            EventField::Title => match key {
                KEY_MENU => {
                    self.state = AppState::DayView;
                    return true;
                }
                KEY_BACKSPACE => {
                    self.form_title.pop();
                }
                KEY_DOWN => {
                    self.form_field = EventField::Hour;
                }
                KEY_ENTER => {
                    // Submit handled by caller
                    return false; // signal submit
                }
                c if c >= ' ' && c <= '~' => {
                    if self.form_title.len() < 40 {
                        self.form_title.push(c);
                    }
                }
                _ => {}
            },
            EventField::Hour => match key {
                KEY_MENU => {
                    self.state = AppState::DayView;
                    return true;
                }
                KEY_UP => {
                    self.form_field = EventField::Title;
                }
                KEY_DOWN => {
                    self.form_field = EventField::Minute;
                }
                KEY_LEFT => {
                    if self.form_hour > 0 {
                        self.form_hour -= 1;
                    } else {
                        self.form_hour = 23;
                    }
                }
                KEY_RIGHT => {
                    if self.form_hour < 23 {
                        self.form_hour += 1;
                    } else {
                        self.form_hour = 0;
                    }
                }
                ' ' => {
                    self.form_has_time = !self.form_has_time;
                }
                KEY_ENTER => return false,
                _ => {}
            },
            EventField::Minute => match key {
                KEY_MENU => {
                    self.state = AppState::DayView;
                    return true;
                }
                KEY_UP => {
                    self.form_field = EventField::Hour;
                }
                KEY_DOWN => {
                    self.form_field = EventField::Priority;
                }
                KEY_LEFT => {
                    if self.form_minute >= 5 {
                        self.form_minute -= 5;
                    } else {
                        self.form_minute = 55;
                    }
                }
                KEY_RIGHT => {
                    if self.form_minute < 55 {
                        self.form_minute += 5;
                    } else {
                        self.form_minute = 0;
                    }
                }
                KEY_ENTER => return false,
                _ => {}
            },
            EventField::Priority => match key {
                KEY_MENU => {
                    self.state = AppState::DayView;
                    return true;
                }
                KEY_UP => {
                    self.form_field = EventField::Minute;
                }
                KEY_LEFT | KEY_RIGHT | ' ' => {
                    self.form_priority = self.form_priority.cycle();
                }
                KEY_ENTER => return false,
                _ => {}
            },
        }
        true
    }

    fn handle_add_event(&mut self, key: char) -> bool {
        let still_editing = self.handle_event_form(key);
        if !still_editing {
            // Submit
            if !self.form_title.is_empty() {
                let id = self.alloc_id();
                let mut event = Event::new(id, self.current_date, self.form_title.clone());
                if self.form_has_time {
                    event.time = Some(Time::new(self.form_hour, self.form_minute));
                }
                event.priority = self.form_priority;
                self.events.push(event);
                self.save_state();
            }
            self.state = AppState::DayView;
        }
        true
    }

    fn handle_edit_event(&mut self, key: char) -> bool {
        let still_editing = self.handle_event_form(key);
        if !still_editing {
            // Apply edits
            if let Some(eid) = self.editing_event_id {
                if let Some(ev) = self.events.iter_mut().find(|e| e.id == eid) {
                    if !self.form_title.is_empty() {
                        ev.title = self.form_title.clone();
                    }
                    ev.time = if self.form_has_time {
                        Some(Time::new(self.form_hour, self.form_minute))
                    } else {
                        None
                    };
                    ev.priority = self.form_priority;
                }
                self.save_state();
            }
            self.state = AppState::DayView;
        }
        true
    }

    fn handle_add_task(&mut self, key: char) -> bool {
        match key {
            KEY_MENU => {
                self.state = AppState::TaskList;
            }
            KEY_BACKSPACE => {
                self.task_input.pop();
            }
            KEY_ENTER => {
                if !self.task_input.is_empty() {
                    let id = self.alloc_id();
                    let task = Task::new(id, self.task_input.clone());
                    self.tasks.push(task);
                    sort_tasks(&mut self.tasks);
                    self.save_state();
                }
                self.state = AppState::TaskList;
            }
            c if c >= ' ' && c <= '~' => {
                if self.task_input.len() < 50 {
                    self.task_input.push(c);
                }
            }
            _ => {}
        }
        true
    }

    fn handle_confirm_del(&mut self, key: char) -> bool {
        match key {
            'y' | 'Y' | KEY_ENTER => {
                if let Some(target) = self.delete_target.take() {
                    match target {
                        DeleteTarget::Event(id) => {
                            self.events.retain(|e| e.id != id);
                            self.day_cursor = 0;
                            self.state = AppState::DayView;
                        }
                        DeleteTarget::Task(id) => {
                            self.tasks.retain(|t| t.id != id);
                            if self.task_cursor > 0
                                && self.task_cursor >= self.tasks.len()
                            {
                                self.task_cursor = self.tasks.len().saturating_sub(1);
                            }
                            self.state = AppState::TaskList;
                        }
                    }
                    self.save_state();
                }
            }
            _ => {
                // Any other key = cancel
                self.delete_target = None;
                match self.state {
                    _ => self.state = AppState::DayView,
                }
            }
        }
        true
    }

    fn handle_month_view(&mut self, key: char) -> bool {
        match key {
            KEY_MENU | KEY_ENTER => {
                // Select this date and go to day view
                let dim = Date::days_in_month(self.month_view_year, self.month_view_month);
                if self.month_cursor_day >= 1 && self.month_cursor_day <= dim {
                    self.current_date = Date::new(
                        self.month_view_year,
                        self.month_view_month,
                        self.month_cursor_day,
                    );
                    self.day_cursor = 0;
                }
                self.state = AppState::DayView;
            }
            KEY_LEFT => {
                if self.month_cursor_day > 1 {
                    self.month_cursor_day -= 1;
                }
            }
            KEY_RIGHT => {
                let dim = Date::days_in_month(self.month_view_year, self.month_view_month);
                if self.month_cursor_day < dim {
                    self.month_cursor_day += 1;
                }
            }
            KEY_UP => {
                if self.month_cursor_day > 7 {
                    self.month_cursor_day -= 7;
                } else {
                    self.month_cursor_day = 1;
                }
            }
            KEY_DOWN => {
                let dim = Date::days_in_month(self.month_view_year, self.month_view_month);
                if self.month_cursor_day + 7 <= dim {
                    self.month_cursor_day += 7;
                } else {
                    self.month_cursor_day = dim;
                }
            }
            '[' => {
                // Previous month
                if self.month_view_month > 1 {
                    self.month_view_month -= 1;
                } else {
                    self.month_view_month = 12;
                    self.month_view_year -= 1;
                }
                let dim = Date::days_in_month(self.month_view_year, self.month_view_month);
                if self.month_cursor_day > dim {
                    self.month_cursor_day = dim;
                }
            }
            ']' => {
                // Next month
                if self.month_view_month < 12 {
                    self.month_view_month += 1;
                } else {
                    self.month_view_month = 1;
                    self.month_view_year += 1;
                }
                let dim = Date::days_in_month(self.month_view_year, self.month_view_month);
                if self.month_cursor_day > dim {
                    self.month_cursor_day = dim;
                }
            }
            _ => {}
        }
        true
    }
}
