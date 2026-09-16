/* MAIN ENTRY WAY TO COMPOSITOR
- USSES SMITHAY 0.7 */

/* IMPORTS */
mod state;
mod utils;

use calloop::{EventLoop, Interest, generic::Generic};
use smithay::reexports::{wayland_server::Display};
use tracing_subscriber;

/* MAIN FUNCTION */
fn main() -> Result<(), Box<dyn std::error::Error>>  {
    tracing_subscriber::fmt().compact().init(); // set up tracing (it gives you further information
//                                                 about whats happening ins the code, you can think of it
//                                                 like a an auto println! log machine)

    let mut event_loop: EventLoop<state::NocturaStates> = EventLoop::try_new().unwrap();   // make our loop
    let display: Display<state::NocturaStates> = Display::new()?;                // make our display
    let mut noctura_comp = state::NocturaStates::try_new(&display, &event_loop);  // attempt for
//                                                                                  a new instance of our compositor


    event_loop.handle().clone()
        .insert_source(
            Generic::new(display, Interest::READ, calloop::Mode::Level),
            |_, display, state| {
                // Safety: we don't drop the display
                unsafe {
                    display.get_mut().dispatch_clients(state).unwrap();
                }
                Ok(calloop::PostAction::Continue)
            },
        )
        .expect("Error while trying to initializing wayland server source");

    let winit_res = noctura_comp.start_win(&mut event_loop); //         open the window through which we 
//                                                                                                      can see noctura compositor

    if winit_res.is_err() {
        let err_type = winit_res.unwrap_err();
        return Err(err_type)
    }
    event_loop.run(None, &mut noctura_comp, move |_| {
        // noctura compositor runs now
    })?;
    Ok(())
}
