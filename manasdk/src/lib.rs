#![allow(non_camel_case_types)]

use std::cell::LazyCell;
use std::ffi::c_void;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::size_of;
use std::sync::LazyLock;
use bitfield::bitfield;
use flagset::FlagSet;
use tracing::info;
use widestring::WideChar;

pub use collections::*;
pub use enums::*;
pub use fields::*;
pub use functions::*;
use manasdk_macros::extend;

mod enums;
mod functions;
mod collections;
mod fields;
mod strings;

include!(concat!(env!("OUT_DIR"), "/generated_code.rs"));

static BASE_ADDRESS: LazyLock<usize> = LazyLock::new(|| {
    unsafe {
        let handle  =windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(std::ptr::null()) as usize;
        info!("Module handle: {} ({:x})", handle, handle);
        
        handle
    }
});


/// Pointer to an UObject that might be null
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
    pub wide_name: [WideChar; 0x400]
}

#[repr(C)]
pub struct FNameEntry {
    pub header: FNameEntryHeader,
    pub name: FStringData
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
    pub v_table: *const c_void,
    pub flags: FlagSet<EObjectFlags>,
    pub index: i32,
    pub class: UObjectPointer<UClass>,
    pub name: FName,
    pub outer: UObjectPointer<UObject>
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
    pub default_object: *const UObject,
    pub _pad_3: [u8; 0x110],
}

#[repr(C)]
#[extend(UStruct)]
#[derive(Debug, Clone)]
pub struct UFunction {
    pub _padding_300: [u8; 48usize],
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
    fn test_inheritance() {

    }
}