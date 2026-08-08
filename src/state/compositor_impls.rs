use std::time::Duration;

use calloop::{EventLoop, LoopHandle, LoopSignal};
use smithay::{backend::{renderer::{damage::OutputDamageTracker, element::surface::WaylandSurfaceRenderElement}, session::Event, winit::{self, WinitEvent, WinitGraphicsBackend}}, delegate_compositor, delegate_data_device, delegate_output, delegate_seat, delegate_shm, delegate_xdg_shell, desktop::{Space, Window, space}, input::{Seat, SeatHandler, SeatState}, output::{Mode, Output, PhysicalProperties}, reexports::{ash::khr::display, wayland_server::{Client, Display, backend::Backend, protocol::{wl_buffer, wl_surface::WlSurface}}}, utils::{Rectangle, Transform::Flipped180}, wayland::{buffer::BufferHandler, compositor::{CompositorClientState, CompositorHandler, CompositorState}, output::{OutputHandler, OutputManagerState}, selection::{SelectionHandler, data_device::{ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDnDGrab, ServerDndGrabHandler}}, shell::xdg::{XdgShellHandler, XdgShellState}, shm::{ShmHandler, ShmState}, socket::ListeningSocketSource}};
use smithay::backend::renderer::gles::GlesRenderer;

use crate::{state::NocturaStates};
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
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_state
    }
    
    fn new_toplevel(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        let win = Window::new_wayland_window(surface);
        self.space.map_element(win, (0, 0), false);
    }

    fn new_popup(&mut self, surface: smithay::wayland::shell::xdg::PopupSurface, positioner: smithay::wayland::shell::xdg::PositionerState) {
        
    }

    fn grab(&mut self, surface: smithay::wayland::shell::xdg::PopupSurface, seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat, serial: smithay::utils::Serial) {
        
    }
    fn reposition_request(&mut self, surface: smithay::wayland::shell::xdg::PopupSurface, positioner: smithay::wayland::shell::xdg::PositionerState, token: u32) {
        
    }
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
delegate_xdg_shell!(NocturaStates);


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
    pub fn try_new(display: &Display<Self>, ls: LoopSignal) -> Self { // making a new instance of our compositor
        let time = std::time::Instant::now();
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
        let space = Space::default();



        Self::prep_seat(&mut seat);

        Self { // return the struct but first, populating it
            time,
            dh,
            ls,
            listen_source,
            socket_name,
            ss,
            seat,
            output_manager,
            comp_state,
            shm_state,
            dds,
            xdg_state,
            space
        }
    }

    pub fn prep_seat(seat: &mut Seat<Self>) {
        seat.add_keyboard(Default::default(), 200, 25).unwrap();
        seat.add_pointer();
    }

    pub fn start_win<'a>(mut self, el: LoopHandle<'a, LoopSignal>) -> Result<(), Box<dyn std::error::Error>> {
        // this function is used to open the window through which you can see Noctura compositor
        let (mut backend, winit) = winit::init::<GlesRenderer>()?;
        let mut out_mode = Mode {
            size: backend.window_size(),
            refresh: 60000,
        };
        let output = Output::new(
            "noctura".into(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: smithay::output::Subpixel::Unknown,
                // NOTE: add real monitor name + model in the non-nested version
                make: "NocturaDisplay".into(),
                model: "Nested".into(),
            }
        );

        let _ = output.create_global::<Self>(&self.dh);
        output.change_current_state(
            Some(out_mode),
            Some(Flipped180),      // NOTE: I currently dont know why i have to flip 
//                                                  the display, but for openGL you must do it, im not sure about other renderers
            None,
            Some((0, 0).into())
        );
        output.set_preferred(out_mode);
        self.space.map_output(&output, (0, 0));
        let mut damage_tracker = OutputDamageTracker::from_output(&output);
        el.insert_source(winit, move |event, _, state| {
            match event {
                WinitEvent::CloseRequested => {
                    self.ls.stop();
                }
                WinitEvent::Resized { size, scale_factor } => {
                    out_mode.size = size;   // NOTE: find out why this dosent work
                }
                WinitEvent::Redraw => {
                    let size = backend.window_size();
                    let damage_space = Rectangle::from_size(size);
                    let (renderer, mut framebuffer) = backend.bind().unwrap();
                    
                    let _ = space::render_output::<
                        _,
                        WaylandSurfaceRenderElement<GlesRenderer>,
                        _,
                        _
                    >(
                        &output, renderer, &mut framebuffer,
                        1.0, 0, [&self.space],
                        &[],
                        &mut damage_tracker,
                        [0.7, 0.1, 1.0, 1.0]
                    ).unwrap();

                    drop(framebuffer); // Because framebuffer borrow backend, I cant mutably borrow it later when 
//                                        I have to submit the damaged frame. Hence, if i drop it, it will no longer exist
//                                        which is fine since i already used it but more importantly, it will allow me to
//                                        mutably borrow backend again to submit the damaged frame.
                    backend.submit(Some(&[damage_space]));

                    for win in self.space.elements() {
                        win.send_frame(&output, self.time.elapsed(), Some(Duration::ZERO), |_, _| { 
                            Some(output.clone())   // clone as &output already borrowed it
                        });
                    }

                    self.space.refresh();
                    self.dh.flush_clients();
                    backend.window().request_redraw();
                }
                _ => {}
            }
        }).unwrap();
        Ok(())
    }
}

