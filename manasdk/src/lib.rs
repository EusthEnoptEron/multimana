#![allow(non_camel_case_types)]

use std::cell::{Cell, LazyCell, OnceCell};
use std::ffi::c_void;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Add, Deref};
use std::sync::{LazyLock, OnceLock};

use bitfield::bitfield;
use flagset::FlagSet;
use tracing::info;
use widestring::WideChar;

pub use collections::*;
pub use enums::*;
pub use fields::*;
pub use functions::*;
use manasdk_macros::extend;

use crate::Offsets::OFFSET_GWORLD;

mod enums;
mod functions;
mod collections;
mod fields;
mod strings;

include!(concat!(env!("OUT_DIR"), "/generated_code.rs"));

static BASE_ADDRESS: LazyLock<usize> = LazyLock::new(|| {
    unsafe {
        let handle = windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(std::ptr::null()) as usize;
        info!("Module handle: {} ({:x})", handle, handle);

        handle
    }
});

fn resolve_offset<T>(offset: usize) -> *mut T {
    (*BASE_ADDRESS + offset) as *mut T
}


/// Pointer to an UObject that might be null
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UObjectPointer<T: AsRef<UObject>>(
    *mut T
);

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
pub union FStringData
{
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
pub struct FString {
    pub data: TArray<WideChar>,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TSubclassOf<UClass> {
    pub class_ptr: *const UClass,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FText {
    pub text_data: *const c_void,
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
pub struct UObject {
    pub v_table: *const usize,
    pub flags: FlagSet<EObjectFlags>,
    pub index: i32,
    pub class: UObjectPointer<UClass>,
    pub name: FName,
    pub outer: UObjectPointer<UObject>,
}

impl AsRef<UObject> for UObject {
    fn as_ref(&self) -> &UObject {
        self
    }
}

#[repr(C)]
#[extend(UObject)]
#[derive(Debug, Clone)]
pub struct UField {
    pub next: UObjectPointer<UField>,
}

#[repr(C)]
#[extend(UField)]
#[derive(Debug, Clone)]
pub struct UStruct {
    pub _padding_200: [u8; 0x10],
    pub super_: UObjectPointer<UStruct>,
    pub children: UObjectPointer<UField>,
    pub child_properties: *const FField,
    pub size: i32,
    pub min_alignment: i32,
    pub _padding_201: [u8; 0x50],
}

#[repr(C)]
#[extend(UStruct)]
#[derive(Debug, Clone)]
pub struct UClass {
    pub _pad_1: [u8; 0x20],
    pub cast_flags: FlagSet<EClassCastFlags>,
    pub _pad_2: [u8; 0x40],
    pub default_object: UObjectPointer<UObject>,
    pub _pad_3: [u8; 0x110],
}


pub type FNativeFuncPtr = fn(context: *const c_void, stack: *const c_void, result: *mut c_void);

#[repr(C)]
#[extend(UStruct)]
#[derive(Debug, Clone)]
pub struct UFunction {
    pub function_flags: FlagSet<EFunctionFlags>,
    pub _padding_300: [u8; 0x20usize],
    pub exec_function: FNativeFuncPtr
}


impl UWorld {
    pub fn get_world() -> Option<&'static UWorld> {
        if Offsets::OFFSET_GWORLD != 0 {
            unsafe {
                resolve_offset::<*const UWorld>(OFFSET_GWORLD).as_ref()?.as_ref()
            }
        } else {
            UEngine::get_engine()?.game_viewport.as_ref()?.world.as_ref()
        }
    }
}

impl UEngine {
    pub fn class() -> &'static UClass {
        UClass::find("Engine").unwrap()
    }

    pub fn get_engine() -> Option<&'static Self> {
        thread_local! {
            static CACHE: LazyCell<Option<&'static UEngine>> = LazyCell::new(|| {
                let uengine = UEngine::class();

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


mod Params {
    use crate::{APlayerController, UObject, UObjectPointer};

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct GameplayStatics_GetPlayerController {
        pub world_context_obj: *const UObject,
        pub player_index: i32,
        pub _padding: [u8;4],
        pub return_value: UObjectPointer<APlayerController>
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_size() {
            assert_eq!(size_of::<GameplayStatics_GetPlayerController>(), 0x000018);
            assert_eq!(align_of::<GameplayStatics_GetPlayerController>(), 0x00008);
        }
    }
}

impl UGameplayStatics {
    pub fn get_player_controller(world_context_obj: &UObject, player_index: i32) -> UObjectPointer<APlayerController> {
        let class = UClass::find("GameplayStatics")
            .expect("Unable to find GameplayStatics");

        let func = class
            .find_function_mut("GameplayStatics", "GetPlayerController")
            .expect("Unable to find GameplayStatics::GetPlayerController");

        let mut parms = Params::GameplayStatics_GetPlayerController {
            world_context_obj,
            player_index,
            _padding: Default::default(),
            return_value: Default::default(),
        };

        let flags = func.function_flags;
        func.function_flags |= EFunctionFlags::Native;
        class.default_object.as_ref().expect("No default object").process_event(func, &mut parms);
        func.function_flags = flags;

        parms.return_value
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


    #[test]
    fn test_u_class() {
        assert_eq!(size_of::<UClass>(), 560usize);
    }

    #[test]
    fn test_u_function() {
        assert_eq!(size_of::<UFunction>(), 224usize);
    }

    #[test]
    fn test_inheritance() {}
}