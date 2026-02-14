# Day Planner — Build Notes

## Architecture
- 7-state machine: DayView, TaskList, AddEvent, EditEvent, AddTask, ConfirmDel, MonthView
- Events have date + optional time + priority
- Tasks have done/not-done toggle + priority
- Month view uses Sakamoto's day-of-week algorithm

## Key Patterns
**Date navigation** — left/right arrows move day-by-day, month view for jumping
**Multi-field form** — EventField enum tracks which field is active
**Event dots** — month view shows dots for days with events
**Priority cycling** — Low → Normal → High → Low via P key
**Sort orders** — Events by time, tasks by done-status then priority

## Build
```bash
cargo build -p planner --target riscv32imac-unknown-xous-elf
```
