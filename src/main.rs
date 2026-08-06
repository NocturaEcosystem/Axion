/* MAIN ENTRY WAY TO COMPOSITOR
- USSES SMITHAY 0.6 */

/* IMPORTS */
mod state;
use calloop::EventLoop;
use tracing_subscriber;
use calloop::LoopSignal;

/* MAIN FUNCTION */
fn main() {
    tracing_subscriber::fmt().compact().init(); // set up tracing (it gives you further information
//                                                 about whats happening ins the code, you can think of it
//                                                 like a an auto println! log machine)

    let event_loop: EventLoop<LoopSignal> = EventLoop::try_new().unwrap();   // make our loop

}
