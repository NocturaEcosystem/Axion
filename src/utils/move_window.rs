use smithay::{desktop::Window, input::{pointer::GrabStartData, pointer::PointerGrab}, utils::{self, Logical}};
use crate::state::NocturaStates;
pub struct MovingSurface {
    pub sd: GrabStartData<NocturaStates>,
    pub win: Window,
    pub init_win: utils::Point<i32, Logical>
}

impl PointerGrab<NocturaStates> for MovingSurface {
    fn axis(&mut self, data: &mut NocturaStates, handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>, details: smithay::input::pointer::AxisFrame) {
        handle.axis(data, details);
    }


    fn button(&mut self, data: &mut NocturaStates, handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>, event: &smithay::input::pointer::ButtonEvent) {

        handle.button(data, event);
        if !(handle.current_pressed().contains(&0x110)) {
            handle.unset_grab(self, data, event.serial, event.time, true)
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
    ){
        handle.gesture_hold_begin(data, event);
    }


    fn gesture_hold_end(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        event: &smithay::input::pointer::GestureHoldEndEvent,
    ){
        handle.gesture_hold_end(data, event);
    }


    fn gesture_pinch_begin(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        event: &smithay::input::pointer::GesturePinchBeginEvent,
    ){
        handle.gesture_pinch_begin(data, event);
    }


    fn gesture_pinch_end(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        event: &smithay::input::pointer::GesturePinchEndEvent,
    ){
        handle.gesture_pinch_end(data, event);
    }


    fn gesture_pinch_update(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        event: &smithay::input::pointer::GesturePinchUpdateEvent,
    ){
        handle.gesture_pinch_update(data, event);
    }


    fn gesture_swipe_begin(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        event: &smithay::input::pointer::GestureSwipeBeginEvent,
    ){
        handle.gesture_swipe_begin(data, event);
    }


    fn gesture_swipe_end(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        event: &smithay::input::pointer::GestureSwipeEndEvent,
    ){
        handle.gesture_swipe_end(data, event);
    }


    fn gesture_swipe_update(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        event: &smithay::input::pointer::GestureSwipeUpdateEvent,
    ){
        handle.gesture_swipe_update(data, event);
    }


    fn motion(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        focus: Option<(<NocturaStates as smithay::input::SeatHandler>::PointerFocus, utils::Point<f64, Logical>)>,
        event: &smithay::input::pointer::MotionEvent,
    ){
        handle.motion(data, None, event);
        let updated = event.location - self.sd.location;
        let new_loc = self.init_win.to_f64() + updated;
        data.space.map_element(self.win.clone(), new_loc.to_i32_round(), true);
    }


    fn relative_motion(
        &mut self,
        data: &mut NocturaStates,
        handle: &mut smithay::input::pointer::PointerInnerHandle<'_, NocturaStates>,
        focus: Option<(<NocturaStates as smithay::input::SeatHandler>::PointerFocus, utils::Point<f64, Logical>)>,
        event: &smithay::input::pointer::RelativeMotionEvent,
    ){
        handle.relative_motion(data, focus, event);
    }


    fn start_data(&self) -> &smithay::input::pointer::GrabStartData<NocturaStates> {
        &self.sd
    }


    fn unset(&mut self, data: &mut NocturaStates) {}
}