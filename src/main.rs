/* MAIN ENTRY WAY TO COMPOSITOR
- USSES SMITHAY 0.7 */

/* IMPORTS */
mod state;
use calloop::EventLoop;
use smithay::reexports::{wayland_server::Display};
use tracing_subscriber;
use calloop::LoopSignal;

/* MAIN FUNCTION */
fn main() -> Result<(), Box<dyn std::error::Error>>  {
    tracing_subscriber::fmt().compact().init(); // set up tracing (it gives you further information
//                                                 about whats happening ins the code, you can think of it
//                                                 like a an auto println! log machine)

    let mut event_loop: EventLoop<LoopSignal> = EventLoop::try_new().unwrap();   // make our loop
    let display: Display<state::NocturaStates> = Display::new()?;                // make our display
    let noctura_comp = state::NocturaStates::try_new(&display, event_loop.get_signal());  // attempt for
//                                                                                  a new instance of our compositor
    noctura_comp.start_win(event_loop.handle())?;                             // open the window through which we 
//                                                                                   can see noctura compositor
    event_loop.run(None, &mut event_loop.get_signal(), move |_| {
        // noctura compositor runs now
    })?;
    Ok(())
}
