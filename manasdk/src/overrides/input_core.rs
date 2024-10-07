use std::ffi::c_void;
use std::fmt::{Display, Formatter};
use crate::{is_bit_set, FName, FText, TSharedPtr};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FKey {
    pub key_name: FName,
    pub key_details: TSharedPtr<FKeyDetails>,
}

impl FKey {
    pub fn is_modifier_key(&self) -> bool {
        self.key_details.as_ref().map(|it| it.is_modifier_key()).unwrap_or_default()
    }
    pub fn is_gamepad_key(&self) -> bool {
        self.key_details.as_ref().map(|it| it.is_gamepad_key()).unwrap_or_default()
    }
    pub fn is_touch(&self) -> bool {
        self.key_details.as_ref().map(|it| it.is_touch()).unwrap_or_default()
    }
    pub fn is_mouse_button(&self) -> bool {
        self.key_details.as_ref().map(|it| it.is_mouse_button()).unwrap_or_default()
    }
    pub fn is_bindable_in_blueprints(&self) -> bool {
        self.key_details.as_ref().map(|it| it.is_bindable_in_blueprints()).unwrap_or_default()
    }
    pub fn should_update_axis_without_samples(&self) -> bool {
        self.key_details.as_ref().map(|it| it.should_update_axis_without_samples()).unwrap_or_default()
    }
    pub fn is_bindable_to_actions(&self) -> bool {
        self.key_details.as_ref().map(|it| it.is_bindable_to_actions()).unwrap_or_default()
    }
    pub fn is_deprecated(&self) -> bool {
        self.key_details.as_ref().map(|it| it.is_deprecated()).unwrap_or_default()
    }
}

impl Display for FKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = self.key_name.to_string().unwrap_or_default();
        // let str = &self.key_details.as_ref()
        //     .map(|it| {
        //         if it.long_display_name.is_set {
        //             it.long_display_name.value.to_string()
        //         } else if it.short_display_name.is_set {
        //             it.short_display_name.value.to_string()
        //         } else {
        //             it.key.key_name.to_string().unwrap_or_default()
        //         }
        //     }).unwrap_or_else(|| {
        //     self.key_name.to_string().unwrap_or_default()
        // });

        write!(f, "{str}")
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FKeyDetails {
    pub key: FKey,
    pub paired_axis: EPairedAxis,
    pub paired_axis_key: FKey,

    pub menu_category: FName,

    pub keys: u8,

    pub axis_type: EInputAxisType,

    pub long_display_name: TAttribute<FText>,
    pub short_display_name: TAttribute<FText>,
}

impl FKeyDetails {
    pub fn is_modifier_key(&self) -> bool {
        is_bit_set(self.keys, 0)
    }
    pub fn is_gamepad_key(&self) -> bool {
        is_bit_set(self.keys, 1)
    }
    pub fn is_touch(&self) -> bool {
        is_bit_set(self.keys, 2)
    }
    pub fn is_mouse_button(&self) -> bool {
        is_bit_set(self.keys, 3)
    }
    pub fn is_bindable_in_blueprints(&self) -> bool {
        is_bit_set(self.keys, 4)
    }
    pub fn should_update_axis_without_samples(&self) -> bool {
        is_bit_set(self.keys, 5)
    }
    pub fn is_bindable_to_actions(&self) -> bool {
        is_bit_set(self.keys, 6)
    }
    pub fn is_deprecated(&self) -> bool {
        is_bit_set(self.keys, 7)
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TAttribute<T>
where
    T: Sized,
{
    pub value: T,
    pub is_set: bool,
    pub getter: *const c_void,
}


#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum EPairedAxis
{
    Unpaired = 0,            // This key is unpaired
    X = 1,                    // This key represents the X axis of its PairedAxisKey
    Y = 2,                    // This key represents the Y axis of its PairedAxisKey
    Z = 3,                // This key represents the Z axis of its PairedAxisKey - Currently unused
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum EInputAxisType
{
    None = 0,
    Button = 1,            // Whilst the physical input is an analog axis the FKey uses it to emulate a digital button.
    Axis1D = 2,
    Axis2D = 3,
    Axis3D = 4,
}
