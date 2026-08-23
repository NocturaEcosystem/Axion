/* MAIN ENTRY WAY TO COMPOSITOR
- USSES SMITHAY 0.7 */

/* IMPORTS */
mod state;
use std::sync::Arc;
mod utils;

use calloop::{EventLoop, Interest, generic::Generic};
use smithay::reexports::{wayland_server::Display};
use tracing_subscriber;
use calloop::LoopSignal;

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
        .unwrap();

    {
        state::start_win(&mut noctura_comp, event_loop.handle().clone());                             // open the window through which we 
//                                                                                                      can see noctura compositor
    }
    event_loop.run(None, &mut noctura_comp, move |_| {
        // noctura compositor runs now
    })?;
    Ok(())
}
