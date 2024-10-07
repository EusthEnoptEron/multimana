use std::ffi::c_void;
use crate::input_core::FKey;
use crate::slate_core::EInputEvent;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FInputKeyEventArgs {
    /// The viewport which the key event is from.
    pub viewport: *const c_void,
    /// The controller which the key event is from.
    pub controller_id: i32,
    /// The type of event which occurred.
    pub key: FKey,
    /// The type of event which occurred.
    pub event: EInputEvent,
    /// For analog keys, the depression percent.
    pub amount_depressed: f32,
    /// input came from a touch surface.This may be a faked mouse button from touch.
    pub b_is_touch_event: bool
}