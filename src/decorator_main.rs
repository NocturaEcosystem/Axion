use std::io::{self, BufRead};

use smithay::wayland::shell::xdg;
use tracing::warn;
slint::include_modules!();

fn main() {
    unsafe {
        std::env::set_var("WAYLAND_DISPLAY", "wayland-2");
    }
    let decore = TestPreview::new().unwrap();
    decore.set_window_title("No title".into());
    let decore_handle = decore.as_weak();
    std::thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if let Ok(request) = line {
                let handle = decore_handle.clone();
                if let Some(new_title) = request.strip_prefix("SET_TITLE:") {
                    let title = new_title.to_string();
                    
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = handle.upgrade() {
                            ui.set_window_title(title.into());
                        }
                    }).unwrap();
                } else if let Some(new_pos) = request.strip_prefix("SET_POS:") {
                    // Dosent work, but still written for future developers/explorers
                    if let Some((x, y)) = new_pos.split_once(":") {
                        if let (Ok(x), Ok(y)) = (x.parse::<i32>(), y.parse::<i32>()) {
                            let xdg = slint::invoke_from_event_loop( move || {
                                if let Some(ui) = handle.upgrade() {
                                    ui.window().set_position(slint::PhysicalPosition::new(x, y));
                                }
                            });
                            println!("xdf: {:?}", xdg);
                        }
                    }
                } else if let Some(new_w) = request.strip_prefix("SET_WIDTH:") {
                    if let Ok(new_w) = new_w.parse::<f32>() {
                        let _ = slint::invoke_from_event_loop( move || {
                            if let Some(ui) = handle.upgrade() {
                                ui.set_w(new_w);
                            }
                        });
                    }
                } else {
                    warn!("IGNORE COMMAND {}", request)
                }

            }
        }
    });

    decore.run().unwrap();
}