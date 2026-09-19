use std::{cell::RefCell, process::id, sync::Arc, time::Duration};

use calloop::{EventLoop};
use smithay::{backend::{input::{AbsolutePositionEvent, Axis, AxisSource, ButtonState, Event, InputEvent, KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent}, renderer::{self, damage::OutputDamageTracker, element::{AsRenderElements, surface::WaylandSurfaceRenderElement}, utils::on_commit_buffer_handler}, winit::{self, WinitEvent, WinitGraphicsBackend, WinitInput}}, delegate_compositor, delegate_data_control, delegate_data_device, delegate_fractional_scale, delegate_keyboard_shortcuts_inhibit, delegate_output, delegate_primary_selection, delegate_seat, delegate_shm, delegate_single_pixel_buffer, delegate_viewporter, delegate_xdg_activation, delegate_xdg_foreign, delegate_xdg_shell, desktop::{PopupManager, Space, Window, WindowSurfaceType, space}, input::{Seat, SeatHandler, SeatState, keyboard::FilterResult, pointer::{AxisFrame, ButtonEvent, CursorImageStatus, Focus, MotionEvent}}, output::{Mode, Output, PhysicalProperties}, reexports::{ash::khr::display, wayland_protocols::xdg::shell::server::xdg_toplevel, wayland_server::{Client, Display, Resource, backend::Backend, protocol::{wl_buffer, wl_surface::WlSurface}}, winit::keyboard, x11rb::protocol::randr::Output as otherOutput}, utils::{Logical, Point, Rectangle, SERIAL_COUNTER, Scale, Transform::Flipped180}, wayland::{buffer::BufferHandler, compositor::{self, CompositorClientState, CompositorHandler, CompositorState, get_parent, is_sync_subsurface}, fractional_scale::{FractionalScaleHandler, FractionalScaleManagerState}, keyboard_shortcuts_inhibit::{KeyboardShortcutsInhibitHandler, KeyboardShortcutsInhibitState}, output::{OutputHandler, OutputManagerState}, seat::WaylandFocus, selection::{SelectionHandler, data_device::{self, ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDnDGrab, ServerDndGrabHandler}, primary_selection::{PrimarySelectionHandler, PrimarySelectionState}, wlr_data_control::{DataControlHandler, DataControlState}}, shell::xdg::{XdgShellHandler, XdgShellState}, shm::{ShmHandler, ShmState}, single_pixel_buffer::SinglePixelBufferState, socket::ListeningSocketSource, viewporter::ViewporterState, xdg_activation::{XdgActivationHandler, XdgActivationState}, xdg_foreign::{XdgForeignHandler, XdgForeignState}}, xwayland::xwm::WmWindowProperty::WindowType};
use smithay::backend::renderer::gles::GlesRenderer;
use tracing::{warn, info};

use crate::{state::{NocturaCursor, NocturaStates, cursor_impls::PointerRenderElement}, utils::{move_window::MovingSurface, resize_window::{Edge, ResizingSurfaceStates, resizingSurface}, unconstrain_popups}};
use crate::state::NocturaClients;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State;



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

    fn cursor_image(&mut self, _seat: &Seat<Self>, image: smithay::input::pointer::CursorImageStatus) {
        self.cs = image
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        let client = focused.and_then(|surface| {
            self.dh.get_client(surface.id()).ok()   // basically, if there was a surface, we need the client
                                                    // this code gets the client who owns the surface
        });
        data_device::set_data_device_focus(&self.dh, seat, client);
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

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
        if !is_sync_subsurface(surface) {
            let mut root = surface.clone();
            while let Some(parent) = get_parent(&root) {
                root = parent;
            }
            if let Some(window) = self
                .space
                .elements()
                .find(|w| w.toplevel().unwrap().wl_surface() == &root)
            {
                window.on_commit();
            }
        };
        self.popups.commit(surface);
        handleResizedCommit(&mut self.space, surface);
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

impl PrimarySelectionHandler for NocturaStates {
    fn primary_selection_state(&self) -> &PrimarySelectionState {
        &self.primary_selection
    }
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
        surface.send_configure(); // sending configure is like telling the client to have x,y dimentions, be minimized/maximized.....
        let win = Window::new_wayland_window(surface);
        self.space.map_element(win, (10, 10), false);
    }

    fn new_popup(&mut self, surface: smithay::wayland::shell::xdg::PopupSurface, positioner: smithay::wayland::shell::xdg::PositionerState) {
        unconstrain_popups::unconstrain_popups(&self, &surface);
        surface.send_configure().expect(
            "An error has occured while sending configuations to a popup...."); // same thing with the toplevel, just with a popup
        let _ = self.popups.track_popup(smithay::desktop::PopupKind::Xdg(surface));
    }

    fn grab(&mut self, surface: smithay::wayland::shell::xdg::PopupSurface, seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat, serial: smithay::utils::Serial) {
        // ??
    }
    
    fn reposition_request(&mut self, surface: smithay::wayland::shell::xdg::PopupSurface, positioner: smithay::wayland::shell::xdg::PositionerState, token: u32) {
        surface.with_pending_state(|state| {
            let geometry = positioner.get_geometry();
            state.geometry = geometry;
            state.positioner = positioner;
        });
        unconstrain_popups::unconstrain_popups(&self, &surface);
        surface.send_repositioned(token);
    }

    fn move_request(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface, seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat, serial: smithay::utils::Serial) {
        let wlSurface = surface.wl_surface().clone();
        let pointer = self.seat.get_pointer();
        if pointer.is_none() {
            return;
        }
        let pointer = pointer.unwrap(); // safe to unwrap now 
        if !pointer.has_grab(serial) {
            return;
        }
        let sd = match pointer.grab_start_data() {
            Some(data) => data,
            None => return
        };
        
        let (fc, _) = match sd.focus.as_ref() {
            Some(data) => data,
            None => return
        };
        if !fc.id().same_client_as(&wlSurface.id()) {
            return;
        }
        let window = self.space.elements()
                                        .find(|w| w.toplevel().unwrap().wl_surface() == &wlSurface).unwrap().clone();
        let init_win_loc = self.space.element_location(&window).unwrap();

        let mving_grab = MovingSurface {
            sd,
            win: window,
            init_win: init_win_loc
        };
        pointer.set_grab(self, mving_grab, serial, Focus::Clear);
    }
    
    fn resize_request(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface, seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat, serial: smithay::utils::Serial, edges: smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,){
        let wlSurface = surface.wl_surface().clone();
        let pointer = self.seat.get_pointer();
        if pointer.is_none() {
            return;
        }
        let pointer = pointer.unwrap(); // safe to unwrap now 
        if !pointer.has_grab(serial) {
            return;
        }
        let sd = match pointer.grab_start_data() {
            Some(data) => data,
            None => return
        };
        
        let (fc, _) = match sd.focus.as_ref() {
            Some(data) => data,
            None => return
        };
        if !fc.id().same_client_as(&wlSurface.id()) {
            return
        }
        let window = self.space.elements()
                                        .find(|w| w.toplevel().unwrap().wl_surface() == &wlSurface).unwrap().clone();
        let init_win_loc = self.space.element_location(&window).unwrap();
        let init_win_size = window.geometry().size;
        surface.with_pending_state(|state| {
            state.states.set(State::Resizing);
        });
        surface.send_configure();
        let rszing_grab = resizingSurface::start_resizing(
            sd,
            window,
            edges.into(),
            Rectangle::new(init_win_loc, init_win_size)
        );

        pointer.set_grab(self, rszing_grab, serial, Focus::Clear);


    }

/* LATER     fn fullscreen_request(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface, wlout: Option<smithay::reexports::wayland_server::protocol::wl_output::WlOutput>) {
      let wl_s = surface.wl_surface();


        let option_geo = wlout.as_ref().and_then(|o| Output::from_resource(&o)).or_else(|| {
            let w = self.space.elements().find(|win| {
                win.wl_surface().map(|s| &*s == wl_s).unwrap_or(false)
            });
            w.and_then(|w| self.space.outputs_for_element(w).first().cloned())
        }).as_ref().and_then(|o| self.space.output_geometry(o));

        if let Some(geo) = option_geo {
            let output = wlout.as_ref().and_then(|o| Output::from_resource(o))
                .unwrap_or_else(|| self.space.outputs().next().unwrap().clone());
            if let Ok(client) = self.dh.get_client(wl_s.id()){
                let mut wl_output = None;
                output.client_outputs(&client).for_each(|output| {
                    wl_output = Some(output)
                });
                let window = self.space.elements().find(|w| {
                    w.wl_surface().map(|s| {
                        &*s == wl_s
                    }).unwrap_or(false)
                });
                if window.is_none() {
                    return
                }
                let window = window.unwrap(); // unwrap is safe here

                surface.with_pending_state(|state| {
                    state.states.set(xdg_toplevel::State::Fullscreen);
                    state.size = Some(geo.size);
                    state.fullscreen_output = wl_output;
                });
            }
        }
    } */

    fn maximize_request(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        let win = self.space.elements().find(|window| {
            window.wl_surface().map(|s| &*s == surface.wl_surface()
            ).unwrap_or(false)})
            .cloned();
        if win.is_none() {
            surface.send_configure(); // send configure, done or not
            return
        }
        let win = win.unwrap(); // unwrap is safe here
        let bind = self.space.outputs_for_element(&win);
        let output = bind.first().or_else(|| self.space.outputs().next()).expect("Error, no outputs found");
        let geo = self.space.output_geometry(output);
        if geo.is_none() {
            surface.send_configure(); // send configure, done or not
            return
        }
        let geo = geo.unwrap(); // unwrap is safe here
        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Maximized);
            state.size = Some(geo.size)
        });
        self.space.map_element(win, geo.loc, true);
        surface.send_configure(); // send configure, done or not
    }

    fn unmaximize_request(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        surface.with_pending_state(|state| {
            state.states.unset(xdg_toplevel::State::Maximized);
            state.size = None
        });
        // TODO: store where moved to, then when unmaximized, move there
        surface.send_configure(); // send configure, done or not
    }
}

impl FractionalScaleHandler for NocturaStates {
    // TODO
}

impl XdgActivationHandler for NocturaStates {
    fn activation_state(&mut self) -> &mut XdgActivationState {
        &mut self.xas
    }
    fn request_activation(
        &mut self,
        _token: smithay::wayland::xdg_activation::XdgActivationToken,
        token_data: smithay::wayland::xdg_activation::XdgActivationTokenData,
        surface: WlSurface,
    )
    {
        if token_data.timestamp.elapsed().as_millis() < 10000 {
            let target_window = self.space.elements().find(|win| {
                win.wl_surface().map(|surf| {
                    *surf == surface
                }).unwrap_or(false)
            }).cloned();
            if let Some(window) = target_window {
                self.space.raise_element(&window, true);
            }
        }
    }

    fn token_created(&mut self, token: smithay::wayland::xdg_activation::XdgActivationToken, data: smithay::wayland::xdg_activation::XdgActivationTokenData) -> bool {
        if data.serial.is_none() {
            return false
        }
        let (serial, seat) = data.serial.unwrap(); // unwrap is safe here
        let keyboard = self.seat.get_keyboard();
        if keyboard.is_none() {
            return false
        }
        let keyboard = keyboard.unwrap(); // unwrap is safe here
        Seat::from_resource(&seat) == Some(self.seat.clone()) &&
        keyboard.last_enter().map(|last| {serial.is_no_older_than(&last)}).unwrap_or(false)
    }
}

impl XdgForeignHandler for NocturaStates {
    fn xdg_foreign_state(&mut self) -> &mut XdgForeignState {
        &mut self.xdg_fs
    }
}

impl DataControlHandler for NocturaStates {
    fn data_control_state(&self) -> &smithay::wayland::selection::wlr_data_control::DataControlState {
        &self.data_cs
    }
}

impl KeyboardShortcutsInhibitHandler for NocturaStates {
    fn keyboard_shortcuts_inhibit_state(&mut self) -> &mut KeyboardShortcutsInhibitState {
        &mut self.shortcut_inhibitor
    }
    fn new_inhibitor(&mut self, inhibitor: smithay::wayland::keyboard_shortcuts_inhibit::KeyboardShortcutsInhibitor) {
        inhibitor.activate();
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
delegate_fractional_scale!(NocturaStates);
delegate_primary_selection!(NocturaStates);
delegate_viewporter!(NocturaStates);
delegate_xdg_activation!(NocturaStates);
delegate_xdg_foreign!(NocturaStates);
delegate_data_control!(NocturaStates);
delegate_single_pixel_buffer!(NocturaStates);
delegate_keyboard_shortcuts_inhibit!(NocturaStates);


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
    pub fn try_new(display: &Display<Self>, el: &EventLoop<Self>) -> Self { // making a new instance of our compositor
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
        info!("SOCKET NAME: {:?}", socket_name);
        let xdg_state = XdgShellState::new::<Self>(&dh);
        let space = Space::default();
        let popups = PopupManager::default();
        let cs = CursorImageStatus::default_named();
        let fraction_scale = FractionalScaleManagerState::new::<Self>(&dh);
        let primary_selection = PrimarySelectionState::new::<Self>(&dh);
        let vps = ViewporterState::new::<Self>(&dh);
        let xas = XdgActivationState::new::<Self>(&dh);
        let xdg_fs= XdgForeignState::new::<Self>(&dh);
        let data_cs = DataControlState::new::<Self, _>(&dh, Some(&primary_selection), |_client| {true});
        let single_pixle_buff = SinglePixelBufferState::new::<Self>(&dh);
        let shortcut_inhibitor = KeyboardShortcutsInhibitState::new::<Self>(&dh);



        el.handle().clone()
            .insert_source(listen_source, move |client_stream, _, state| {
                // Inside the callback, you should insert the client into the display.
                //
                // You may also associate some data with the client when inserting the client.
                let result  = state
                    .dh
                    .insert_client(client_stream, Arc::new(NocturaClients::default()));
                if let Err(error) = result {
                    warn!("Error with wayland client: {}", error);
                }
            })
            .expect("Failed to init the wayland event source.");




        Self::prep_seat(&mut seat);

        Self { // return the struct but first, populating it
            time,
            dh,
            ls: el.get_signal(),
            socket_name,
            ss,
            seat,
            output_manager,
            popups,
            comp_state,
            shm_state,
            dds,
            xdg_state,
            space,
            cs,
            pointerPos: (0.0, 0.0).into(),
            fraction_scale,
            primary_selection,
            vps,
            xas,
            xdg_fs,
            data_cs,
            single_pixle_buff,
            shortcut_inhibitor
        }
    }

    pub fn prep_seat(seat: &mut Seat<Self>) {
        seat.add_keyboard(Default::default(), 200, 25).expect("Error initializing keyboard");
        seat.add_pointer();
    }

    pub fn handle_input(&mut self, e: InputEvent<WinitInput>) {
        match e {
            InputEvent::Keyboard { event, .. } => {
                if let Some(keyboard) = self.seat.get_keyboard() {
                    keyboard.input::<(), _>(
                        self, event.key_code(), event.state(), SERIAL_COUNTER.next_serial(), event.time_msec(),
                        |_, _, _| FilterResult::Forward
                    );
                }
                
            }
            InputEvent::PointerMotionAbsolute { event } => {
                let output = self.space.outputs().next().unwrap();
                let output_geo = self.space.output_geometry(output).unwrap();
                self.pointerPos = event.position_transformed(output_geo.size);
                let pointer = self.seat.get_pointer().unwrap();
                let pointer_position = event.position_transformed(output_geo.size) + output_geo.loc.to_f64();
                let motionEvent = &MotionEvent {
                    location: pointer_position,
                    serial: SERIAL_COUNTER.next_serial(),
                    time: event.time() as u32,
                };
                let focus = self.space.element_under(pointer_position).and_then(
                    |(win, loc)| {
                        win.surface_under(pointer_position - loc.to_f64(), WindowSurfaceType::ALL).map(
                            |(surface, point)| {
                                (surface, (point + loc).to_f64())
                            }
                        )
                    }
                );
                pointer.motion(self, focus, motionEvent);
                pointer.frame(self);
            }
            InputEvent::PointerButton { event } => {
                let btn_state = event.state();
                let pointer = self.seat.get_pointer();
                let kb = self.seat.get_keyboard();
                if pointer.is_none() || kb.is_none() {
                    return;
                }
                let pointer = pointer.unwrap(); // unwrap is safe now
                let kb = kb.unwrap(); // unwrap is safe now
                let serial = SERIAL_COUNTER.next_serial();
                if btn_state == ButtonState::Pressed && !pointer.is_grabbed() {
                    if let Some(window) = self.space.element_under(pointer.current_location()).map(|(w, _)| {
                        w.clone()
                    }) {
                        self.space.raise_element(&window, true);
                        kb.set_focus(self, Some(window.toplevel().unwrap().wl_surface().clone()), serial);
                        self.space.elements().for_each(|window| {
                            window.toplevel().unwrap().send_pending_configure();
                        });
                    } else {
                        for window in self.space.elements() {
                            window.set_activated(false);
                            window.toplevel().unwrap().send_pending_configure();
                        }
                        kb.set_focus(self, None, serial);
                    }
                }

                pointer.button(self, &ButtonEvent {
                    serial: serial,
                    time: event.time_msec(),
                    state: event.state(),
                    button: event.button_code(),
                });
                pointer.frame(self);
            }
            InputEvent::PointerAxis { event } => {
                /*
                    using ratio " 15/120 " because:
                        mouse: each noch (the clicking noise you hear when you scroll on a mouse) = 120units
                        smtiahy: each noch = 15units
                    think of it like the mouse using "meters" and smithay using "killometers"
                    because 1km = 1000m, to turn m -> km, we multiply m by 1/1000
                    but in this case,
                    because 15smitahy units = 120 v120 mouse units, to turn v120->sm wemultiply by 15/120
                */
                let h_amnt = event.amount(Axis::Horizontal).unwrap_or_else(||{
                    event.amount_v120(Axis::Horizontal).unwrap_or(0.0) * (15.0 / 120.0)
                });
                let v_amnt = event.amount(Axis::Vertical).unwrap_or_else(||{
                    event.amount_v120(Axis::Vertical).unwrap_or(0.0) * (15.0 / 120.0)
                });
                let h_amnt_120 = event.amount_v120(Axis::Horizontal);
                let v_amnt_120 = event.amount_v120(Axis::Vertical);

                let mut frame = AxisFrame::new(event.time_msec()).source(event.source());
                if h_amnt != 0.0 {
                    frame = frame.relative_direction(Axis::Horizontal, event.relative_direction(Axis::Horizontal));
                    frame = frame.value(Axis::Horizontal, h_amnt);
                    if let Some(discrete) = h_amnt_120 {
                        frame = frame.v120(Axis::Horizontal, discrete as i32);
                    }
                }
                if v_amnt != 0.0 {
                    frame = frame.relative_direction(Axis::Vertical, event.relative_direction(Axis::Vertical));
                    frame = frame.value(Axis::Vertical, v_amnt);
                    if let Some(discrete) = v_amnt_120 {
                        frame = frame.v120(Axis::Vertical, discrete as i32);
                    }
                }
                if event.source() == AxisSource::Finger {
                    if event.amount(Axis::Horizontal) == Some(0.0) {
                        frame = frame.stop(Axis::Horizontal);
                    }
                    if event.amount(Axis::Vertical) == Some(0.0) {
                        frame = frame.stop(Axis::Vertical);
                    }
                }
                let pointer = self.seat.get_pointer();
                if pointer.is_none() {
                    return;
                }
                let pointer = pointer.unwrap(); // unwrap is safe now
                pointer.axis(self, frame);
                pointer.frame(self);
            }
            _ => {}
        }
    }

    pub fn start_win(&mut self, el: &mut EventLoop<NocturaStates>) -> Result<(), Box<dyn std::error::Error>> {
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

        let _ = output.create_global::<NocturaStates>(&self.dh);
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
        let mut pointer = NocturaCursor::try_new(backend.renderer()).unwrap();
        el.handle().insert_source(winit, move |event, _, state| {
            match event {
                WinitEvent::CloseRequested => {
                    state.ls.stop();
                }
                WinitEvent::Resized { size, scale_factor } => {
                    output.change_current_state(Some(Mode {size, refresh: 60_000}),
                        None,
                        None,
                        None,
                    );
                }
                WinitEvent::Redraw => {
                    let render_res = backend.bind().and_then(|(renderer, mut framebuffer)| {
                        pointer.current_delay(state.time.elapsed());
                        pointer.cs = state.cs.clone();
                            let scale = Scale::from(output.current_scale().fractional_scale());
                        let cp = state.pointerPos;
                        let cps = cp.to_physical(scale).to_i32_round();
                        let mut elements = Vec::<PointerRenderElement<GlesRenderer>>::new();
                        elements.extend(pointer.render_elements::<PointerRenderElement<GlesRenderer>>
                                (renderer, cps, scale, 1.0));
                        space::render_output::<
                            _,
                            PointerRenderElement<GlesRenderer>,
                            _,
                            _
                        >(
                            &output, renderer, &mut framebuffer,
                            1.0, 0, [&state.space],
                            &elements,
                            &mut damage_tracker,
                            [0.7, 0.1, 1.0, 1.0]
                        ).map_err(|err| match err {
                            renderer::damage::Error::Rendering(err) => smithay::backend::SwapBuffersError::from(err),
                            _ => unreachable!(),
                        })
                    });

                    match render_res {
                        Ok(ror) => {
                            if let Some(damage) = ror.damage {
                                backend.submit(Some(damage));
                            }
                        }
                        Err(error) => {
                            warn!("Error while rendering {}", error)
                        }
                    }
    

                    for win in state.space.elements() {
                        win.send_frame(&output, state.time.elapsed(), Some(Duration::ZERO), |_, _| { 
                            Some(output.clone())   // clone as &output already borrowed it
                        });
                    }

                    state.space.refresh();
                    state.dh.flush_clients();
                    backend.window().request_redraw();
                }
                WinitEvent::Input(e) => {
                    state.handle_input(e);
                }
                _ => {}
            }
        }).unwrap();
        Ok(())
    }
}

fn handleResizedCommit(space: &mut Space<Window>, surface: &WlSurface) -> Option<()> {
    let win = space.elements().find(|w| {
        w.toplevel().unwrap().wl_surface() == surface
    }).cloned()?;
    let mut location = space.element_location(&win)?;
    let geo = win.geometry();

    let newLocation: Point<Option<i32>, Logical> = 
        compositor::with_states(surface, |surfData| {
            surfData.data_map.insert_if_missing(RefCell::<ResizingSurfaceStates>::default);
            let state = surfData.data_map.get::<RefCell<ResizingSurfaceStates>>().unwrap();
            state.borrow_mut().wrap().and_then(|(edges, rect)| {
                edges.intersects(Edge::TOP | Edge::LEFT).then(|| {
                    let x = if edges.intersects(Edge::LEFT) {
                        Some(rect.loc.x + (rect.size.w - geo.size.w))
                    } else {
                        None
                    };
                    let y = if edges.intersects(Edge::TOP) {
                        Some(rect.loc.y + (rect.size.h - geo.size.h))
                    } else {
                        None
                    };

                    (x, y).into()
                })
            }).unwrap_or_default()
        });
    if let Some(x) = newLocation.x {
        location.x = x;
    }
    if let Some(y) = newLocation.y {
        location.y = y;
    }
    if newLocation.x.is_some() || newLocation.y.is_some() {
        space.map_element(win, location, false);
    }
    Some(())
}