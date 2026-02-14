//! UI rendering for Day Planner.
//!
//! All rendering uses the GAM 1-bit monochrome display.
//! Screen: 336×536 with header (30px) and footer (46px).

extern crate alloc;
use alloc::format;
use alloc::string::String;

use gam::*;
use graphics_server::api::GlyphStyle;
use graphics_server::{DrawStyle, PixelColor, Point, Rectangle, TextBounds};

use crate::app::*;
use crate::planner::*;

const SCREEN_W: i16 = 336;
const HEADER_H: i16 = 30;
const FOOTER_H: i16 = 46;
const LINE_H: i16 = 22;

fn draw_header(gam: &Gam, canvas: Canvas, text: &str) {
    let header_rect = Rectangle::new(
        Point::new(0, 0),
        Point::new(SCREEN_W - 1, HEADER_H - 1),
    );
    gam.draw_rectangle(canvas, header_rect.style(
        DrawStyle::new(PixelColor::Dark, PixelColor::Dark, 0),
    )).ok();

    let tb = TextBounds::BoundingBox(Rectangle::new(
        Point::new(4, 2),
        Point::new(SCREEN_W - 4, HEADER_H - 2),
    ));
    gam.draw_textview(
        canvas,
        tv::TextView::new(tb, text)
            .style(GlyphStyle::Bold)
            .draw_border(false)
            .invert(true),
    ).ok();
}

fn draw_footer(gam: &Gam, canvas: Canvas, text: &str) {
    let y = 536 - FOOTER_H;
    gam.draw_line(canvas, Point::new(0, y), Point::new(SCREEN_W - 1, y),
        DrawStyle::new(PixelColor::Dark, PixelColor::Dark, 1),
    ).ok();

    let tb = TextBounds::BoundingBox(Rectangle::new(
        Point::new(4, y + 4),
        Point::new(SCREEN_W - 4, 536 - 2),
    ));
    gam.draw_textview(
        canvas,
        tv::TextView::new(tb, text)
            .style(GlyphStyle::Small)
            .draw_border(false),
    ).ok();
}

fn draw_text(gam: &Gam, canvas: Canvas, x: i16, y: i16, text: &str, style: GlyphStyle) {
    let tb = TextBounds::BoundingBox(Rectangle::new(
        Point::new(x, y),
        Point::new(SCREEN_W - 4, y + LINE_H),
    ));
    gam.draw_textview(
        canvas,
        tv::TextView::new(tb, text)
            .style(style)
            .draw_border(false),
    ).ok();
}

fn draw_text_inverted(gam: &Gam, canvas: Canvas, x: i16, y: i16, w: i16, text: &str) {
    let bg = Rectangle::new(Point::new(x, y), Point::new(x + w, y + LINE_H));
    gam.draw_rectangle(canvas, bg.style(
        DrawStyle::new(PixelColor::Dark, PixelColor::Dark, 0),
    )).ok();

    let tb = TextBounds::BoundingBox(Rectangle::new(
        Point::new(x + 2, y),
        Point::new(x + w - 2, y + LINE_H),
    ));
    gam.draw_textview(
        canvas,
        tv::TextView::new(tb, text)
            .style(GlyphStyle::Regular)
            .draw_border(false)
            .invert(true),
    ).ok();
}

pub fn draw(app: &PlannerApp, gam: &Gam, canvas: Canvas) {
    gam.draw_rectangle(
        canvas,
        Rectangle::new(Point::new(0, 0), Point::new(SCREEN_W - 1, 535))
            .style(DrawStyle::new(PixelColor::Light, PixelColor::Light, 0)),
    ).ok();

    match app.state {
        AppState::DayView => draw_day_view(app, gam, canvas),
        AppState::TaskList => draw_task_list(app, gam, canvas),
        AppState::AddEvent | AppState::EditEvent => draw_event_form(app, gam, canvas),
        AppState::AddTask => draw_add_task(app, gam, canvas),
        AppState::ConfirmDel => draw_confirm(app, gam, canvas),
        AppState::MonthView => draw_month_view(app, gam, canvas),
    }

    gam.redraw().ok();
}

fn draw_day_view(app: &PlannerApp, gam: &Gam, canvas: Canvas) {
    let header = format!(
        "{} {} {} {}",
        app.current_date.weekday_name(),
        app.current_date.display(),
        " ",
        if app.pending_task_count() > 0 {
            format!("[{} tasks]", app.pending_task_count())
        } else {
            String::new()
        }
    );
    draw_header(gam, canvas, &header);

    let events = app.events_for_date();
    let mut y = HEADER_H + 4;

    if events.is_empty() {
        draw_text(gam, canvas, 8, y, "No events scheduled", GlyphStyle::Regular);
        y += LINE_H + 4;
        draw_text(gam, canvas, 8, y, "Press A to add an event", GlyphStyle::Small);
    } else {
        for (i, ev) in events.iter().enumerate() {
            let prefix = format!(
                "{} {} {}",
                ev.priority.marker(),
                ev.time_display(),
                ev.title
            );
            if i == app.day_cursor {
                draw_text_inverted(gam, canvas, 4, y, SCREEN_W - 8, &prefix);
            } else {
                draw_text(gam, canvas, 8, y, &prefix, GlyphStyle::Regular);
            }
            y += LINE_H + 2;

            if y > 536 - FOOTER_H - LINE_H {
                break;
            }
        }
    }

    draw_footer(
        gam,
        canvas,
        "<>/> Day  A)dd  E)dit  D)el  T)asks  M)onth  Menu=Quit",
    );
}

fn draw_task_list(app: &PlannerApp, gam: &Gam, canvas: Canvas) {
    let done_count = app.tasks.iter().filter(|t| t.done).count();
    let header = format!(
        "Tasks ({}/{})",
        app.tasks.len() - done_count,
        app.tasks.len()
    );
    draw_header(gam, canvas, &header);

    let mut y = HEADER_H + 4;

    if app.tasks.is_empty() {
        draw_text(gam, canvas, 8, y, "No tasks yet", GlyphStyle::Regular);
        y += LINE_H + 4;
        draw_text(gam, canvas, 8, y, "Press A to add a task", GlyphStyle::Small);
    } else {
        for (i, task) in app.tasks.iter().enumerate() {
            let check = if task.done { "[x]" } else { "[ ]" };
            let line = format!("{} {} {}", check, task.priority.marker(), task.title);
            if i == app.task_cursor {
                draw_text_inverted(gam, canvas, 4, y, SCREEN_W - 8, &line);
            } else {
                draw_text(gam, canvas, 8, y, &line, GlyphStyle::Regular);
            }
            y += LINE_H + 2;
            if y > 536 - FOOTER_H - LINE_H {
                break;
            }
        }
    }

    draw_footer(
        gam,
        canvas,
        "Enter=Toggle  A)dd  P)riority  D)el  <=Back",
    );
}

fn draw_event_form(app: &PlannerApp, gam: &Gam, canvas: Canvas) {
    let title = if app.state == AppState::AddEvent {
        "Add Event"
    } else {
        "Edit Event"
    };
    let header = format!("{} — {}", title, app.current_date.display());
    draw_header(gam, canvas, &header);

    let mut y = HEADER_H + 8;

    // Title field
    let label = "Title:";
    let is_sel = app.form_field == EventField::Title;
    draw_text(gam, canvas, 8, y, label, GlyphStyle::Small);
    y += 16;
    let display_title = if app.form_title.is_empty() && is_sel {
        String::from("_")
    } else if is_sel {
        format!("{}_", app.form_title)
    } else {
        app.form_title.clone()
    };
    if is_sel {
        draw_text_inverted(gam, canvas, 8, y, SCREEN_W - 16, &display_title);
    } else {
        draw_text(gam, canvas, 12, y, &display_title, GlyphStyle::Regular);
    }
    y += LINE_H + 8;

    // Time toggle + hour
    let time_label = if app.form_has_time {
        format!("Time: {}:{:02}  (Space=all-day)", app.form_hour, app.form_minute)
    } else {
        String::from("Time: All day  (Space=set time)")
    };
    let hour_sel = app.form_field == EventField::Hour;
    draw_text(gam, canvas, 8, y, "Hour:", GlyphStyle::Small);
    y += 16;
    if hour_sel {
        draw_text_inverted(gam, canvas, 8, y, SCREEN_W - 16, &time_label);
    } else {
        draw_text(gam, canvas, 12, y, &time_label, GlyphStyle::Regular);
    }
    y += LINE_H + 8;

    // Minute
    if app.form_has_time {
        let min_sel = app.form_field == EventField::Minute;
        let min_label = format!("Minute: {:02}  (</>  +/- 5)", app.form_minute);
        draw_text(gam, canvas, 8, y, "Minute:", GlyphStyle::Small);
        y += 16;
        if min_sel {
            draw_text_inverted(gam, canvas, 8, y, SCREEN_W - 16, &min_label);
        } else {
            draw_text(gam, canvas, 12, y, &min_label, GlyphStyle::Regular);
        }
        y += LINE_H + 8;
    }

    // Priority
    let pri_sel = app.form_field == EventField::Priority;
    let pri_label = format!("Priority: {}  (</>  cycle)", app.form_priority.label());
    draw_text(gam, canvas, 8, y, "Priority:", GlyphStyle::Small);
    y += 16;
    if pri_sel {
        draw_text_inverted(gam, canvas, 8, y, SCREEN_W - 16, &pri_label);
    } else {
        draw_text(gam, canvas, 12, y, &pri_label, GlyphStyle::Regular);
    }

    draw_footer(
        gam,
        canvas,
        "Up/Down=Field  Enter=Save  Menu=Cancel",
    );
}

fn draw_add_task(app: &PlannerApp, gam: &Gam, canvas: Canvas) {
    draw_header(gam, canvas, "Add Task");

    let y = HEADER_H + 20;
    draw_text(gam, canvas, 8, y, "Task description:", GlyphStyle::Small);

    let display = if app.task_input.is_empty() {
        String::from("_")
    } else {
        format!("{}_", app.task_input)
    };
    draw_text_inverted(gam, canvas, 8, y + 20, SCREEN_W - 16, &display);

    draw_footer(gam, canvas, "Enter=Save  Menu=Cancel");
}

fn draw_confirm(app: &PlannerApp, gam: &Gam, canvas: Canvas) {
    draw_header(gam, canvas, "Confirm Delete");

    let y = HEADER_H + 40;
    let msg = match app.delete_target {
        Some(DeleteTarget::Event(id)) => {
            let name = app
                .events
                .iter()
                .find(|e| e.id == id)
                .map(|e| e.title.as_str())
                .unwrap_or("?");
            format!("Delete event '{}'?", name)
        }
        Some(DeleteTarget::Task(id)) => {
            let name = app
                .tasks
                .iter()
                .find(|t| t.id == id)
                .map(|t| t.title.as_str())
                .unwrap_or("?");
            format!("Delete task '{}'?", name)
        }
        None => String::from("Nothing selected"),
    };
    draw_text(gam, canvas, 8, y, &msg, GlyphStyle::Regular);

    let y2 = y + LINE_H + 10;
    draw_text(gam, canvas, 8, y2, "Y = Yes, any other = Cancel", GlyphStyle::Small);

    draw_footer(gam, canvas, "Y)es  Any=Cancel");
}

fn draw_month_view(app: &PlannerApp, gam: &Gam, canvas: Canvas) {
    let header = format!(
        "{} {} — [/] Month",
        Date::month_name(app.month_view_month),
        app.month_view_year
    );
    draw_header(gam, canvas, &header);

    let mut y = HEADER_H + 4;
    let col_w = SCREEN_W / 7;

    // Weekday headers
    let days = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
    for (i, d) in days.iter().enumerate() {
        let x = (i as i16) * col_w + 4;
        draw_text(gam, canvas, x, y, d, GlyphStyle::Small);
    }
    y += 18;

    // First day of month
    let first = Date::new(app.month_view_year, app.month_view_month, 1);
    let start_col = first.day_of_week() as i16;
    let dim = Date::days_in_month(app.month_view_year, app.month_view_month);

    let cell_h: i16 = 28;
    let mut col = start_col;
    let mut row_y = y;

    for day in 1..=dim {
        let x = col * col_w;
        let label = format!("{}", day);

        let is_cursor = day == app.month_cursor_day;
        let is_today = Date::new(app.month_view_year, app.month_view_month, day) == app.current_date;
        let has_events = app.event_count_for(Date::new(
            app.month_view_year,
            app.month_view_month,
            day,
        )) > 0;

        if is_cursor {
            draw_text_inverted(gam, canvas, x + 2, row_y, col_w - 4, &label);
        } else if is_today {
            // Draw a box around today
            let r = Rectangle::new(
                Point::new(x + 1, row_y),
                Point::new(x + col_w - 2, row_y + cell_h - 4),
            );
            gam.draw_rectangle(canvas, r.style(
                DrawStyle::new(PixelColor::Light, PixelColor::Dark, 1),
            )).ok();
            draw_text(gam, canvas, x + 4, row_y + 2, &label, GlyphStyle::Regular);
        } else {
            draw_text(gam, canvas, x + 4, row_y + 2, &label, GlyphStyle::Regular);
        }

        // Event dot
        if has_events {
            let dot_x = x + col_w / 2;
            let dot_y = row_y + cell_h - 6;
            let dot = Rectangle::new(
                Point::new(dot_x - 1, dot_y - 1),
                Point::new(dot_x + 1, dot_y + 1),
            );
            gam.draw_rectangle(canvas, dot.style(
                DrawStyle::new(PixelColor::Dark, PixelColor::Dark, 0),
            )).ok();
        }

        col += 1;
        if col >= 7 {
            col = 0;
            row_y += cell_h;
        }
    }

    draw_footer(
        gam,
        canvas,
        "Arrows=Navigate  [/]=Month  Enter=Select",
    );
}
