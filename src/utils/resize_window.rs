use std::{cell::RefCell, default};

use bitflags::bitflags;
use smithay::{desktop::{Space, Window}, input::pointer::{GrabStartData, PointerGrab}, reexports::{wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge, wayland_server::protocol::wl_surface::WlSurface}, utils::{Logical, Rectangle, Size, Point}, wayland::{compositor, seat::WaylandFocus, shell::xdg::SurfaceCachedState}};
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State;
use crate::state::NocturaStates;
bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Edge: u32 {
        const TOP = 0b0001;
        const BOTTOM = 0b0010;
        const LEFT = 0b0100;
        const RIGHT = 0b1000;
        
        /*
            BITS:
                0        0        0        0
                ^        ^        ^        ^
              RIGHT?    LEFT?   BOTTOM?   TOP?

            basically for each bit, if the bit is '1' then that edge -> yes
            if the bit is '0' then that edge -> no

            So for example:

                TOP = 0001

            The last bit is 1, meaning TOP? -> yes.

            And:

                BOTTOM = 0010

            The second-to-last bit is 1, meaning BOTTOM? -> yes.

            This works the same way for LEFT and RIGHT.

            /*
                here,
                    if bit 1 (the first bit from left) AND last bit (from left) was '1'
                    that means, we are referring to "TOP-RIGHT".
                    the "|" operator is used for binary OR.
                    like:-

                    Top:       0001
                    Right:     1000
                    ----------------
                    Top-Right: 1001

                    because 1 OR 0 = 1
                            0 OR 0 = 0

                Multiple edges can be active at the same time.

                For example, TOP + RIGHT:

                    TOP:        0001
                    RIGHT:      1000
                                ----
                    TOP_RIGHT:  1001

                So TOP_RIGHT (1001) means:
                    TOP   → active
                    RIGHT → active

                This lets us represent corner resizing using the same Edge type.
            */

        */

        const TOP_LEFT = Self::TOP.bits() | Self::LEFT.bits();
        const TOP_RIGHT = Self::TOP.bits() | Self::RIGHT.bits();
        const BOTTOM_RIGHT = Self::BOTTOM.bits() | Self::RIGHT.bits();
        const BOTTOM_LEFT = Self::BOTTOM.bits() | Self::LEFT.bits();
    }
}

impl From<ResizeEdge> for Edge {             // frompub struct  is used to convert types
    #[inline]
    fn from(value: ResizeEdge) -> Self {
        Self::from_bits(value as u32).unwrap()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum ResizingSurfaceStates {                // different states of the window related with being resized
    #[default]
    Idle,                                 // nothings happpening
    Resizing {                            // resizing
        edge: Edge,
        init_win: Rectangle<i32, Logical>
    },
    Resized {                             // Resized, now we want a commit from the client
        edge: Edge,
        init_win: Rectangle<i32, Logical>
    }
}

impl ResizingSurfaceStates {
    pub fn wrap(&mut self) -> Option<(Edge, Rectangle<i32, Logical>)> {
        match *self {
            Self::Idle => None,
            Self::Resizing { edge, init_win } => Some((edge, init_win)),
            Self::Resized { edge, init_win } => {
                *self = Self::Idle;
                Some((edge, init_win))
            },
        }
    }
}

pub struct resizingSurface {
    pub sd: GrabStartData<NocturaStates>,
    pub win: Window,
    pub edge: Edge,
    pub init_win: Rectangle<i32, Logical>,
    pub updated_window_size: Size<i32, Logical>,
}

impl resizingSurface {
    pub fn start_resizing(sd: GrabStartData<NocturaStates>, win: Window, edge: Edge, init_win:  Rectangle<i32, Logical>) -> Self {
        compositor::with_states(win.toplevel().unwrap().wl_surface(), |surfData| {
            surfData.data_map.insert_if_missing(RefCell::<ResizingSurfaceStates>::default);
            let state = surfData.data_map.get::<RefCell<ResizingSurfaceStates>>().unwrap();
            *state.borrow_mut() = ResizingSurfaceStates::Resizing { edge, init_win };
        });
        Self {
            sd,
            win,
            edge,
            init_win,
            updated_window_size: init_win.size,
        }
    }
}

impl PointerGrab<NocturaStates> for resizingSurface {
    fn axis(&mut self, data: &mut NocturaStates, handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>, details: smithay::input::pointer::AxisFrame) {
        handle.axis(data, details);
    }
    fn button(&mut self, data: &mut NocturaStates, handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>, event: &smithay::input::pointer::ButtonEvent) {
        handle.button(data, event);
        if !(handle.current_pressed().contains(&0x110)) {
            handle.unset_grab(self, data, event.serial, event.time, true);
            self.win.toplevel().unwrap().with_pending_state(|state| {
                state.states.unset(State::Resizing);
                state.size = Some(self.updated_window_size);
            });
            self.win.toplevel().unwrap().send_pending_configure();
            compositor::with_states(self.win.toplevel().unwrap().wl_surface(), |surfData| {
                surfData.data_map.insert_if_missing(RefCell::<ResizingSurfaceStates>::default);
                let state = surfData.data_map.get::<RefCell<ResizingSurfaceStates>>().unwrap();
                *state.borrow_mut() = ResizingSurfaceStates::Resized { edge: self.edge, init_win: self.init_win };
            });
        }
    }
    fn frame(&mut self, data: &mut NocturaStates, handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>) {
        handle.frame(data);
    }
    fn gesture_hold_begin(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        event: &smithay::input::pointer::GestureHoldBeginEvent,
    )
    {
        handle.gesture_hold_begin(data, event);
    }
    fn gesture_hold_end(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        event: &smithay::input::pointer::GestureHoldEndEvent,
    )
    {
        handle.gesture_hold_end(data, event);
    }
    fn gesture_pinch_begin(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        event: &smithay::input::pointer::GesturePinchBeginEvent,
    )
    {
        handle.gesture_pinch_begin(data, event);
    }
    fn gesture_pinch_end(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        event: &smithay::input::pointer::GesturePinchEndEvent,
    )
    {
        handle.gesture_pinch_end(data, event);
    }
    fn gesture_pinch_update(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        event: &smithay::input::pointer::GesturePinchUpdateEvent,
    )
    {
        handle.gesture_pinch_update(data, event);
    }
    fn gesture_swipe_begin(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        event: &smithay::input::pointer::GestureSwipeBeginEvent,
    )
    {
        handle.gesture_swipe_begin(data, event);
    }
    fn gesture_swipe_end(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        event: &smithay::input::pointer::GestureSwipeEndEvent,
    )
    {
        handle.gesture_swipe_end(data, event);
    }
    fn gesture_swipe_update(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        event: &smithay::input::pointer::GestureSwipeUpdateEvent,
    )
    {
        handle.gesture_swipe_update(data, event);
    }
    fn motion(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        focus: Option<(<NocturaStates as smithay::input::SeatHandler>::PointerFocus, smithay::utils::Point<f64, Logical>)>,
        event: &smithay::input::pointer::MotionEvent,
    )
    {
        handle.motion(data, None, event);
        let mut delta = event.location - self.sd.location;

        let mut updated_w = self.init_win.size.w;
        let mut updated_h = self.init_win.size.h;

        if self.edge.intersects(Edge::RIGHT) {
            updated_w = self.init_win.size.w + delta.x as i32;
        }

        if self.edge.intersects(Edge::LEFT) {
            updated_w = self.init_win.size.w - delta.x as i32;
        }

        if self.edge.intersects(Edge::BOTTOM) {
            updated_h = self.init_win.size.h + delta.y as i32;
        }

        if self.edge.intersects(Edge::TOP) {
            updated_h = self.init_win.size.h - delta.y as i32;
        }
        let (min_size, max_size) = compositor::with_states(self.win.toplevel().unwrap().wl_surface(), |surfData | {
            let mut lockedData = surfData.cached_state.get::<SurfaceCachedState>();
            let data = lockedData.current();
            (data.min_size, data.max_size)
        });

        let min_w = min_size.w.max(1);
        let min_h = min_size.h.max(1);

        let max_w = if max_size.w == 0 {
            2147483647      // max i32 value (+)
        } else {
            max_size.w
        };

        let max_h = if max_size.h == 0 {
            2147483647
        } else {
            max_size.h
        };
        let updated_size = Size::from((
            updated_w.max(min_w).min(max_w),
            updated_h.max(min_h).min(max_h),
        ));

        self.updated_window_size = updated_size;
        
        self.win.toplevel().unwrap().with_pending_state(|state| {
            state.states.set(State::Resizing);
            state.size = Some(self.updated_window_size);
        });
        self.win.toplevel().unwrap().send_pending_configure();

    }
    fn relative_motion(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        focus: Option<(<NocturaStates as smithay::input::SeatHandler>::PointerFocus, smithay::utils::Point<f64, Logical>)>,
        event: &smithay::input::pointer::RelativeMotionEvent,
    )
    {
        handle.relative_motion(data, focus, event);
    }
    fn start_data(&self) -> &GrabStartData<NocturaStates> {
        &self.sd
    }
    fn unset(&mut self, data: &mut NocturaStates) {}
}

