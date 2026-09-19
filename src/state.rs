/* THIS FILE WILL CONTAIN THE STATES WE WILL NEED, THEIR IMPLEMENTATIONS
   WILL BE IN ANOTHER FILE
*/

use std::{collections::BTreeMap, ffi::OsString};

use calloop::{EventLoop, LoopHandle, LoopSignal};
use smithay::{backend::renderer::{Texture, element::texture::TextureBuffer}, desktop::{PopupManager, Space, Window}, input::{Seat, SeatState, pointer::CursorImageStatus}, reexports::{ash::vk::Display, wayland_server::DisplayHandle}, utils::{Logical, Point}, wayland::{compositor::{CompositorClientState, CompositorState}, fractional_scale::FractionalScaleManagerState, keyboard_shortcuts_inhibit::KeyboardShortcutsInhibitState, output::OutputManagerState, selection::{data_device::DataDeviceState, primary_selection::PrimarySelectionState, wlr_data_control::DataControlState}, shell::xdg::XdgShellState, shm::ShmState, single_pixel_buffer::SinglePixelBufferState, socket::ListeningSocketSource, viewporter::ViewporterState, xdg_activation::XdgActivationState, xdg_foreign::XdgForeignState}};
// activate our implementations

mod compositor_impls;
pub struct NocturaStates {    // this will contain data about our compositor as a whole
    pub time: std::time::Instant,
    pub dh: DisplayHandle,
    pub ls: LoopSignal,
    pub socket_name: OsString,
    pub ss: SeatState<Self>,
    pub seat: Seat<Self>,
    pub output_manager: OutputManagerState,
    pub popups: PopupManager,
    pub comp_state: CompositorState,
    pub shm_state: ShmState,
    pub dds: DataDeviceState,
    pub xdg_state: XdgShellState,
    pub space: Space<Window>,
    pub cs: CursorImageStatus,
    pub pointerPos: Point<f64, Logical>,
    pub fraction_scale: FractionalScaleManagerState,
    pub primary_selection: PrimarySelectionState,
    pub vps: ViewporterState,
    pub xas: XdgActivationState,
    pub xdg_fs: XdgForeignState,
    pub data_cs: DataControlState,
    pub single_pixle_buff: SinglePixelBufferState,
    pub shortcut_inhibitor: KeyboardShortcutsInhibitState

}

mod client_impls;
#[derive(Default)]
pub struct NocturaClients {  // this will contain data about our clients/apps
    pub comp_state: CompositorClientState,
}

mod cursor_impls;
pub struct NocturaCursor<T: Texture> {
    pub cs: CursorImageStatus,
    pub whole_stamp: u64,
    pub current_stamp: u64,
    pub map_of_image: BTreeMap<u64, TextureBuffer<T>>,
}



