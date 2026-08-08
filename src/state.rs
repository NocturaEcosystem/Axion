/* THIS FILE WILL CONTAIN THE STATES WE WILL NEED, THEIR IMPLEMENTATIONS
   WILL BE IN ANOTHER FILE
*/

use std::ffi::OsString;

use calloop::{EventLoop, LoopHandle, LoopSignal};
use smithay::{desktop::{Space, Window}, input::{Seat, SeatState}, reexports::{ash::vk::Display, wayland_server::DisplayHandle}, wayland::{compositor::{CompositorClientState, CompositorState}, output::OutputManagerState, selection::data_device::DataDeviceState, shell::xdg::XdgShellState, shm::ShmState, socket::ListeningSocketSource}};
// activate our implementations

mod client_impls;
mod compositor_impls;
pub struct NocturaStates {    // this will contain data about our compositor as a whole
    pub time: std::time::Instant,
    pub dh: DisplayHandle,
    pub ls: LoopSignal,
    pub listen_source: ListeningSocketSource,
    pub socket_name: OsString,
    pub ss: SeatState<Self>,
    pub seat: Seat<Self>,
    pub output_manager: OutputManagerState,
    pub comp_state: CompositorState,
    pub shm_state: ShmState,
    pub dds: DataDeviceState,
    pub xdg_state: XdgShellState,
    pub space: Space<Window>,

}


pub struct NocturaClients {  // this will contain data about our clients/apps
    pub comp_state: CompositorClientState,
}

pub struct NocturaSystem {   // this will contan data like number of worksapce, apps, etc.

}