use smithay::{delegate_compositor, delegate_data_device, delegate_output, delegate_seat, delegate_shm, input::{Seat, SeatHandler, SeatState}, reexports::{ash::khr::display, wayland_server::{Client, Display, protocol::{wl_buffer, wl_surface::WlSurface}}}, wayland::{buffer::BufferHandler, compositor::{CompositorClientState, CompositorHandler, CompositorState}, output::{OutputHandler, OutputManagerState}, selection::{SelectionHandler, data_device::{ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDnDGrab, ServerDndGrabHandler}}, shell::xdg::{XdgShellHandler, XdgShellState}, shm::{ShmHandler, ShmState}, socket::ListeningSocketSource}};

use crate::state::NocturaStates;
use crate::state::NocturaClients;


// Traits:

/* 
    In smithay we have traits, which can be given to structs. for example,
    because I am using the seat field in my sturct, i have to give it the seathandler trait
*/

impl SeatHandler for NocturaStates {
    type PointerFocus = WlSurface;
    type KeyboardFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.ss
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: smithay::input::pointer::CursorImageStatus) {}

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        //NOTE: ADD LATER
    }
}

impl OutputHandler for NocturaStates {}

impl CompositorHandler for NocturaStates {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.comp_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<NocturaClients>().unwrap().comp_state
    }

    fn commit(&mut self, _surface: &WlSurface) {
        // NOTE: will do later
    }
}

impl ShmHandler for NocturaStates {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}
impl BufferHandler for NocturaStates {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl SelectionHandler for NocturaStates {
    type SelectionUserData = ();
}

impl DataDeviceHandler for NocturaStates {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.dds
    }
}
impl ClientDndGrabHandler for NocturaStates {}
impl ServerDndGrabHandler for NocturaStates {}

impl XdgShellHandler for NocturaStates {
    // NOTE: add later
}


// Delegate

/* 
    In smitahy, we 'delegate' traits.
    delegate means to tell smithay "I applied the traits and use my functions for your protocol"
    so for example, if smithay needs to check your seat state to do an operation, it will check the
    trait 'SeatHandler', function 'seat_state' where you return the Seat State
*/

delegate_seat!(NocturaStates);
delegate_output!(NocturaStates);
delegate_compositor!(NocturaStates);
delegate_shm!(NocturaStates);
delegate_data_device!(NocturaStates);


// My own implementations:

/*
    Along with traits i added my own functions to my struct. I did this to make my code easier
    to debug
    
    For people who are used to clasical OOP,
    class mycalss {
        MYVAR = 10

        def myMethod(&self) {
            return self.MYVAR
        }
    }

    in rust is

    struct myclass {
        MYVAR: i32 // you have to define the type, NOT GIVE VALUE
    }

    impl myclass {            // add methods to my class, 'myclass'
        fn new() -> Self  {   // for any new instance of class that wants to run like so: myclass::new(),
//                               but technically, you could also just do myclass { MYVAR: 10 }
            return Self{
                10
            }
        }

        fn myMethod(&self) -> i32 {
            reuturn self.MYVAR
        }
    }
*/

impl NocturaStates {
    pub fn try_new(display: &Display<Self>) -> Self { // making a new instance of our compositor
        let dh: smithay::reexports::wayland_server::DisplayHandle = display.handle(); // create a 'key' to let your compositor talk to wayland server
        let mut ss = SeatState::new();
        let mut seat: Seat<Self> = ss.new_wl_seat(&dh, "Noctura_seat");
        let output_manager: OutputManagerState = OutputManagerState::new_with_xdg_output::<Self>(&dh);
        let comp_state = CompositorState::new::<Self>(&dh);
        let shm_state = ShmState::new::<Self>(&dh, vec![]);
        let dds = DataDeviceState::new::<Self>(&dh);
        let listen_source = ListeningSocketSource::new_auto().unwrap();
        let socket_name = listen_source.socket_name().to_os_string();
        let xdg_state = XdgShellState::new::<Self>(&dh);



        Self::prep_seat(&mut seat);

        Self { // return the struct but first, populating it
            dh,
            listen_source,
            socket_name,
            ss,
            seat,
            output_manager,
            comp_state,
            shm_state,
            dds,
            xdg_state
        }
    }

    pub fn prep_seat(seat: &mut Seat<Self>) {
        seat.add_keyboard(Default::default(), 200, 25).unwrap();
        seat.add_pointer();
    }
}

