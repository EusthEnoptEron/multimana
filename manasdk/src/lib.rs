#![allow(non_camel_case_types)]

use std::cell::LazyCell;
use std::ffi::c_void;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::LazyLock;

use bitfield::bitfield;
use flagset::FlagSet;
use tracing::info;
use widestring::{decode_utf16_lossy, WideChar};

pub use collections::*;
pub use enums::*;
pub use fields::*;
pub use functions::*;
use crate::core_u_object::{FSoftObjectPath, UField, UFunction, UProperty};
use crate::engine::{UWorld, UEngine};
use crate::offsets::OFFSET_GWORLD;

pub use crate::core_u_object::{UClass, UObject};

mod collections;
mod enums;
mod fields;
mod functions;
mod strings;
mod overrides;

include!(concat!(env!("OUT_DIR"), "/generated_code/lib.rs"));

pub trait HasClassObject {
    fn static_class() -> &'static UClass;
}

static BASE_ADDRESS: LazyLock<usize> = LazyLock::new(|| unsafe {
    let handle =
        windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(std::ptr::null()) as usize;
    info!("Module handle: {} ({:x})", handle, handle);

    handle
});

fn resolve_offset<T>(offset: usize) -> *mut T {
    (*BASE_ADDRESS + offset) as *mut T
}

/// Pointer to an UObject that might be null
#[repr(C)]
#[derive(Debug, Clone, Copy, Eq)]
pub struct UObjectPointer<T: AsRef<UObject>>(*mut T);

unsafe impl<T: AsRef<UObject>> Send for UObjectPointer<T> {}
unsafe impl<T: AsRef<UObject>> Sync for UObjectPointer<T> {}

impl<T: AsRef<UObject>> PartialEq for UObjectPointer<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 as usize == other.0 as usize
    }
}

pub trait AsObjectPointer<T: AsRef<UObject>> {
    fn as_pointer(&self) -> UObjectPointer<T>;
}

impl<T: AsRef<UObject>> AsObjectPointer<T> for &T {
    fn as_pointer(&self) -> UObjectPointer<T> {
        UObjectPointer(*self as *const T as *mut T)
    }
}

impl<T> From<&T> for UObjectPointer<T>
where
    T: AsRef<UObject>,
{
    fn from(value: &T) -> Self {
        UObjectPointer(value as &T as *const T as *mut T)
    }
}

impl<T> From<&mut T> for UObjectPointer<T>
where
    T: AsRef<UObject>,
{
    fn from(value: &mut T) -> Self {
        UObjectPointer(value as &mut T as *mut T)
    }
}

impl<T: AsRef<UObject>> Default for UObjectPointer<T> {
    fn default() -> Self {
        Self(std::ptr::null_mut())
    }
}


#[repr(C)]
#[derive(Debug, Clone)]
pub struct FNumberedData {}

bitfield! {
    pub struct FNameEntryHeader(u16);
    impl Debug;
    pub b_is_wide, _: 0, 0;
    _reserved, _ : 5, 1; // 5 bits reserved (padding)
    pub len, _: 15, 6;
}

#[repr(C)]
pub union FStringData {
    pub ansi_name: [u8; 0x400],
    pub wide_name: [WideChar; 0x400],
}

#[repr(C)]
pub struct FNameEntry {
    pub header: FNameEntryHeader,
    pub name: FStringData,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FName {
    pub comparison_index: i32,
    pub number: i32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FScriptName {
    pub comparison_index: i32,
    pub display_index: i32,
    pub number: i32,
}

impl From<FScriptName> for FName {
    fn from(value: FScriptName) -> Self {
        FName {
            comparison_index: value.comparison_index,
            number: value.number,
        }
    }
}

impl Display for FName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string().unwrap_or_default())
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FString {
    pub data: TArray<WideChar>,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TSubclassOf<UClass>(*const c_void, PhantomData<UClass>);

impl<UClass, T> From<&T> for TSubclassOf<UClass> {
    fn from(value: &T) -> Self {
        TSubclassOf(value as *const T as *const c_void, PhantomData::default())
    }
}


#[repr(C)]
#[derive(Debug, Clone)]
pub struct FTextData {
    _padding: [u8; 0x28],
    pub text_source: FString,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FText {
    pub text_data: *const FTextData,
    _padding: [u8; 16],
}


#[repr(C)]
#[derive(Debug, Clone)]
pub struct FWeakObjectPtr {
    pub object_index: i32,
    pub object_serial_number: i32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TWeakObjectPtr<UEType> {
    _phantom: PhantomData<UEType>,
    pub f_weak_object_ptr: FWeakObjectPtr,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TLazyObjectPtr<T> {
    pub weak_ptr: FWeakObjectPtr,
    pub tag_at_last_text: i32,
    pub object_id: T,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TScriptInterface<T> {
    pub object_pointer: *const T,
    pub interface_pointer: *const c_void,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FMulticastSparseDelegateProperty_ {
    _padding: [u8; 1],
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FMulticastInlineDelegateProperty_ {
    _padding: [u8; 16],
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FMulticastInlineDelegate {
    _padding: [u8; 16],
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FDelegateProperty_ {
    pub object: FWeakObjectPtr,
    pub function_name: FName,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TSoftObjectPtr<T> {
    _phantom: PhantomData<T>,
    pub pointer: TLazyObjectPtr<FSoftObjectPath>,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TSoftClassPtr<T> {
    _phantom: PhantomData<T>,
    pub pointer: TLazyObjectPtr<FSoftObjectPath>,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TSharedPtr<T> {
    pub obj: *const T,
    pub _shared_reference_count: *const FReferenceControllerBase,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FReferenceControllerBase {
    pub shared_reference_count: i32,
    pub weak_reference_count: i32,
}

impl<T> TSharedPtr<T> {
    pub fn as_ref(&self) -> Option<&T> {
        unsafe {
            self.obj.as_ref()
        }
    }
}

pub type FNativeFuncPtr = fn(context: UObjectPointer<UObject>, stack: &FFrame, result: *mut c_void);

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FFrame<'a> {
    pub pad_0x0000: [u8; 0x10], // 0x0000
    pub node: &'a UFunction,    // 0x0010
    pub object: &'a UObject,    // 0x0018
    pub code: *mut u8,           // 0x0020
    pub locals: &'a c_void,     // 0x0028
    pub most_recent_property: *mut UProperty,
    pub most_recent_property_address: *mut c_void,
    pub primary_data: [u32; 8], // Execution flow stack for compiled Kismet code
    pub secondary_data: *mut c_void,
    pub array_num: i32,
    pub array_max: i32,
    pub previous_frame: *mut FFrame<'a>, // Previous frame on the stack
    pub out_parms: *mut c_void,          // Contains information on any out parameters
    pub property_chain_for_compiled_in: *mut UField, // Property chain for compiled-in functions
    pub current_native_function: *mut UFunction, // Currently executed native function
    pub b_array_context_failed: bool,    // Indicates if array context failed
}

impl<'a> FFrame<'a> {
    pub unsafe fn get_params<T>(&self) -> Option<&mut T> {
        unsafe {
            let t_p: *mut T = std::mem::transmute(self.locals);
            t_p.as_mut()
        }
    }
}


#[repr(C)]
#[extend(FField)]
#[derive(Debug, Clone)]
pub struct FProperty {
    pub array_dim: i32,
    pub element_size: i32,
    pub property_flags: FlagSet<EPropertyFlags>,
    pub pad_48: [u8; 4],
    pub offset: i32,
    pub pad_50: [u8; 40],
}

impl UWorld {
    pub fn get_world() -> Option<&'static UWorld> {
        if offsets::OFFSET_GWORLD != 0 {
            unsafe {
                resolve_offset::<*const UWorld>(OFFSET_GWORLD)
                    .as_ref()?
                    .as_ref()
            }
        } else {
            UEngine::get_engine()?
                .game_viewport
                .as_ref()?
                .world
                .as_ref()
        }
    }
}

impl UEngine {
    pub fn get_engine() -> Option<&'static Self> {
        thread_local! {
            static CACHE: LazyCell<Option<&'static UEngine>> = LazyCell::new(|| {
                let uengine = UEngine::static_class();

                unsafe {
                    UObject::find_object_of_type(EClassCastFlags::None, |it| {
                        !it.is_default_obj() && it.is_a(uengine)
                    })
                }
            });
        }

        CACHE.with(|it| it.deref().clone())
    }
}


pub fn is_bit_set(value: u8, bit_offset: u8) -> bool {
    let mask = 0b00000001u8 << bit_offset;
    (value & mask) != 0
}

pub fn set_bit(value: u8, bit_offset: u8, is_set: bool) {
    let mask = 0b00000001u8 << bit_offset;
    if is_set {
        value | mask;
    } else {
        value & !mask;
    }
}

#[cfg(test)]
mod collection_tests {
    use std::mem::size_of;
    use super::*;

    #[test]
    fn test_f_string() {}

    #[test]
    fn test_collections() {
        assert_eq!(size_of::<TArray<i32>>(), 0x10, "TArray has a wrong size!");
        assert_eq!(size_of::<TSet<i32>>(), 0x50, "TSet has a wrong size!");
        assert_eq!(size_of::<TMap<i32, i32>>(), 0x50, "TMap has a wrong size!");
        assert_eq!(size_of::<FText>(), 24, "FText has a wrong size!");
    }
}
