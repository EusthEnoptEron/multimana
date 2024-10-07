use libmem::{Address, Vmt};
use manasdk::engine::{FInputKeyEventArgs, UGameViewportClient, UScriptViewportClient};
use manasdk::input_core::FKey;
use manasdk::{EClassCastFlags, HasClassObject, UObject};
use std::ffi::c_void;
use std::ops::Deref;
use std::sync::LazyLock;
use rusty_xinput::{xinput_get_state, XInputHandle, XInputState, XInputUsageError};
use tracing::warn;

const INPUT_KEY_IDX: usize = 0x48 / 8;
const INPUT_AXIS_IDX: usize = 0x50 / 8;

static INPUT_MANAGER: LazyLock<InputManager> = LazyLock::new(|| {
    let game_viewport_client = UGameViewportClient::static_class();
    let viewport: &UGameViewportClient = unsafe {
        UObject::find_object_of_type(EClassCastFlags::None, |it| {
            !it.is_default_obj() && it.is_a(game_viewport_client)
        })
    }.expect("No viewport found");

    let viewport_client_vtable: usize = usize::from_le_bytes(viewport._padding_100[0..8].try_into()
        .expect("Viewport VFTable not found"));
    let mut vmt = Vmt::new(viewport_client_vtable);

    unsafe {
        vmt.hook(INPUT_KEY_IDX, on_input_key as Address);
        vmt.hook(INPUT_AXIS_IDX, on_input_axis as Address);
    }

    InputManager {
        vmt: vmt,
        xinput: XInputHandle::load_default().expect("Unable to load xinput")
    }
});

pub struct InputManager {
    vmt: Vmt,
    xinput: XInputHandle
}

unsafe impl Send for InputManager {}
unsafe impl Sync for InputManager {}

unsafe fn on_input_key(this: &mut UScriptViewportClient, event_args: &FInputKeyEventArgs) -> bool {
    InputManager::instance().vmt.get_original::<fn(&mut UScriptViewportClient, &FInputKeyEventArgs) -> bool>(INPUT_KEY_IDX)
        (this, event_args)
}

unsafe fn on_input_axis(this: &mut UScriptViewportClient, in_viewport: *const c_void, controller_id: i32, key: FKey, delta: f32, delta_time: f32, num_samples: i32, b_gamepad: bool) -> bool {
    //info!("Input Axis: {}", key.key_name);

    InputManager::instance().vmt.get_original::<fn(&mut UScriptViewportClient, *const c_void, i32, FKey, f32, f32, i32, bool) -> bool>(INPUT_AXIS_IDX)
        (this, in_viewport, controller_id, key, delta, delta_time, num_samples, b_gamepad)
}

impl InputManager {
    pub fn instance() -> &'static Self {
        INPUT_MANAGER.deref()
    }
    
    pub fn get_controller_state(&self, idx: u8) -> Option<XInputState> {
        match self.xinput.get_state(idx as u32) {
            Ok(state) => { Some(state) }
            Err(XInputUsageError::DeviceNotConnected) => {
                None
            }
            Err(e) => {
                warn!("XInput complained: {e:?}");
                None
            }
        }
    }
}