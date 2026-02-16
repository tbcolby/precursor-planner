//! Precursor Day Planner
//!
//! Schedule events, track tasks, navigate by day or month.
//! Everything encrypted at rest via PDDB.

#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]


mod app;
mod planner;
mod storage;
mod ui;

use app::PlannerApp;
use planner::Date;
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;

const SERVER_NAME: &str = "_Day Planner_";
const APP_NAME: &str = "Day Planner";

#[derive(Debug, num_derive::FromPrimitive, num_derive::ToPrimitive)]
enum AppOp {
    Redraw = 0,
    Rawkeys = 1,
    FocusChange = 2,
    Quit = 255,
}

fn main() -> ! {
    log_server::init_wait().unwrap();
    log::set_max_level(log::LevelFilter::Info);
    log::info!("Day Planner starting, PID {}", xous::process::id());

    let xns = xous_names::XousNames::new().unwrap();
    let sid = xns
        .register_name(SERVER_NAME, None)
        .expect("can't register server");
    let gam = gam::Gam::new(&xns).expect("can't connect to GAM");
    let tt = ticktimer_server::Ticktimer::new().unwrap();

    let token = gam
        .register_ux(gam::UxRegistration {
            app_name: String::from(gam::APP_NAME_PLANNER),
            ux_type: gam::UxType::Framebuffer,
            predictor: None,
            listener: sid.to_array(),
            redraw_id: AppOp::Redraw.to_u32().unwrap(),
            gotinput_id: None,
            audioframe_id: None,
            rawkeys_id: Some(AppOp::Rawkeys.to_u32().unwrap()),
            focuschange_id: Some(AppOp::FocusChange.to_u32().unwrap()),
        })
        .expect("couldn't register UX")
        .unwrap();

    let content = gam
        .request_content_canvas(token)
        .expect("couldn't get canvas");
    let screensize = gam
        .get_canvas_bounds(content)
        .expect("couldn't get dimensions");
    log::info!("Canvas size: {:?}", screensize);

    // Get initial date from ticktimer (seconds since epoch)
    let epoch_ms = tt.elapsed_ms();
    let initial_date = epoch_to_date(epoch_ms);

    let mut app = PlannerApp::new(initial_date);
    app.init_storage();
    let mut allow_redraw = true;
    ui::draw(&app, &gam, content);

    loop {
        let msg = xous::receive_message(sid).unwrap();
        match FromPrimitive::from_usize(msg.body.id()) {
            Some(AppOp::Redraw) => {
                if allow_redraw {
                    app.needs_redraw = true;
                    ui::draw(&app, &gam, content);
                }
            }
            Some(AppOp::Rawkeys) => xous::msg_scalar_unpack!(msg, k1, k2, k3, k4, {
                let keys = [
                    core::char::from_u32(k1 as u32).unwrap_or('\u{0000}'),
                    core::char::from_u32(k2 as u32).unwrap_or('\u{0000}'),
                    core::char::from_u32(k3 as u32).unwrap_or('\u{0000}'),
                    core::char::from_u32(k4 as u32).unwrap_or('\u{0000}'),
                ];
                let mut should_quit = false;
                for &key in keys.iter() {
                    if key != '\u{0000}' {
                        if !app.handle_key(key) {
                            should_quit = true;
                            break;
                        }
                    }
                }
                if should_quit { break; }
                if app.needs_redraw && allow_redraw {
                    ui::draw(&app, &gam, content);
                    app.needs_redraw = false;
                }
            }),
            Some(AppOp::FocusChange) => xous::msg_scalar_unpack!(msg, state_code, _, _, _, {
                match gam::FocusState::convert_focus_change(state_code) {
                    gam::FocusState::Background => {
                        allow_redraw = false;
                        app.save_state();
                    }
                    gam::FocusState::Foreground => {
                        allow_redraw = true;
                        ui::draw(&app, &gam, content);
                    }
                }
            }),
            Some(AppOp::Quit) => break,
            _ => log::warn!("unknown opcode: {:?}", msg.body.id()),
        }
    }

    app.save_state();
    xns.unregister_server(sid).unwrap();
    xous::destroy_server(sid).unwrap();
    xous::terminate_process(0)
}

/// Convert milliseconds-since-boot to a rough date.
/// Xous ticktimer gives ms since boot, not epoch. For a real RTC
/// we'd use the RTC service. This provides a reasonable default
/// (2026-01-01) that the user can navigate from.
fn epoch_to_date(_ms: u64) -> Date {
    // Precursor doesn't have a persistent RTC that's easily accessible
    // from userspace in all configs. Default to a known date.
    // The user navigates to their actual date with arrow keys.
    Date::new(2026, 1, 1)
}
