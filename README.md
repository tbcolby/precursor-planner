# precursor-planner

Encrypted day planner for the [Precursor](https://precursor.dev) hardware platform.

## Features

- **Day View** — see events for the selected date, navigate with arrow keys
- **Month View** — calendar grid with event dots, quick date picking
- **Task List** — to-do items with check/uncheck, priority levels
- **Event Management** — add, edit, delete events with time and priority
- **PDDB Storage** — all data encrypted at rest

## Controls

| Key | Action |
|-----|--------|
| ←/→ | Previous/next day |
| ↑/↓ | Move cursor in lists |
| A | Add event/task |
| E | Edit selected event |
| D | Delete selected |
| T | Switch to task list |
| M | Month calendar view |
| [/] | Previous/next month (in month view) |
| Enter | Select/confirm/toggle |
| Menu (∴) | Back/quit |

## Build

```bash
cargo build -p planner --target riscv32imac-unknown-xous-elf
```

---

## Development

This app was developed using the methodology described in [xous-dev-toolkit](https://github.com/tbcolby/xous-dev-toolkit) — an LLM-assisted approach to Precursor app development on macOS ARM64.

---

## Author

Made by Tyler Colby — [Colby's Data Movers, LLC](https://colbysdatamovers.com)

Contact: [tyler@colbysdatamovers.com](mailto:tyler@colbysdatamovers.com) | [GitHub Issues](https://github.com/tbcolby/precursor-planner/issues)

---

## License

Licensed under the Apache License, Version 2.0.

See [LICENSE](LICENSE) for the full text.
