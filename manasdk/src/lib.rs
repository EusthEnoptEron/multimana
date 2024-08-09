#![allow(non_camel_case_types)]

mod enums;
mod functions;

use std::ffi::c_void;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, size_of};
use std::os::raw::c_char;
use bitfield::bitfield;
use flagset::FlagSet;
use widestring::WideChar;

pub use enums::*;
pub use functions::*;
use manasdk_macros::extend;

include!(concat!(env!("OUT_DIR"), "/generated_code.rs"));


/// Pointer to an UObject that might be null
#[derive(Debug, Clone, Copy)]
pub struct UObjectPointer<T: AsRef<UObject>>(
    *mut T
);

struct FNamePool {
    _padding: [u8; 8],
    current_block: u32,
    current_byte_cursor: u32,
    blocks: [usize; 0x2000]
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TArray<T> {
    pub data: *const T,
    pub num_elements: u32,
    pub max_elements: u32,
}

#[repr(C)]
#[derive(Debug, Clone)]
/** The result of a sparse array allocation. */
pub struct FSparseArrayAllocationInfo
{
    pub index: i32,
    pub pointer: *const c_void,
}

#[repr(C)]
pub union TSparseArrayElementOrFreeListLink<T> {
    pub element_data: ManuallyDrop<T>,
    pub prev_next_free_index: (i32, i32),
}

impl<T> Clone for TSparseArrayElementOrFreeListLink<T> {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl<T> Debug for TSparseArrayElementOrFreeListLink<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown")
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct InlineAllocator<const NUM_INLINE_ELEMENTS: usize, T> {
    pub data: [T; NUM_INLINE_ELEMENTS],
    pub secondary_data: *const T,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FBitArray {
    pub data: InlineAllocator<4, i32>,
    pub num_bits: i32,
    pub max_bits: i32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TSparseArray<T> {
    pub data: TArray<TSparseArrayElementOrFreeListLink<T>>,
    pub allocation_flags: FBitArray,
    pub first_free_index: i32,
    pub num_free_indices: i32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TSet<T> {
    pub elements: TSparseArray<T>,
    pub hash: InlineAllocator<1, i32>,
    pub hash_size: i32,
}


#[repr(C)]
#[derive(Debug, Clone)]
pub struct TMap<T1, T2> {
    pub elements: TSet<TPair<T1, T2>>,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TPair<T1, T2> {
    pub key: T1,
    pub value: T2,
}


#[repr(C)]
#[derive(Debug, Clone)]
pub struct FUObjectItem {
    pub object: *mut UObject,
    pub flags: i32,
    pub cluster_root_index: i32,
    pub serial_number: i32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TUObjectArray {
    /// Master table to chunks of pointers
    pub objects: *const *const FUObjectItem,
    /// If requested, a contiguous memory where all objects are allocated
    pub pre_allocated_objects: *const FUObjectItem,
    /// Maximum number of elements
    pub max_elements: i32,
    /// Number of elements we currently have
    pub num_elements: i32,
    /// Maximum number of chunks
    pub max_chunks: i32,
    /// Number of chunks we currently have
    pub num_chunks: i32,
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
pub struct FFieldClass {
    pub name: FName,
    pub id: u64,
    pub cast_flags: u64,
    pub class_flags: FlagSet<EClassFlags>,
    pub pad_1c: [u8; 4],
    pub superclass: *mut FFieldClass,
}

#[repr(C)]
pub union ContainerType {
    pub field: *mut FField,
    pub object: *mut UObject,
}

#[repr(C)]
pub struct FFieldVariant {
    pub container: ContainerType,
    pub is_uobject: bool,
}

impl Clone for FFieldVariant {
    fn clone(&self) -> Self {
        Self {
            is_uobject: self.is_uobject,
            container: if self.is_uobject {
                ContainerType {
                    object: unsafe { self.container.object }
                }
            } else {
                ContainerType {
                    field: unsafe { self.container.field }
                }
            },
        }
    }
}

impl Debug for FFieldVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "is_uobject={:?}", self.is_uobject)
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FField {
    pub vtable: *mut std::ffi::c_void,
    pub class_private: *mut FFieldClass,
    pub owner: FFieldVariant,
    pub next: *mut FField,
    pub name: FName,
    pub obj_flags: i32,
}

#[repr(C)]
#[extend(FField)]
#[derive(Debug, Clone)]
pub struct FProperty {
    pub array_dim: i32,
    pub element_size: i32,
    pub property_flags: u64,
    pub pad_48: [u8; 4],
    pub offset: i32,
    pub pad_50: [u8; 40],
}

#[repr(C)]
#[extend(FProperty)]
#[derive(Debug, Clone)]
pub struct FByteProperty {
    pub enum_: UObjectPointer<UEnum>,
}

#[repr(C)]
#[extend(FProperty)]
#[derive(Debug, Clone)]
pub struct FBoolProperty {
    pub field_size: u8,
    pub byte_offset: u8,
    pub byte_mask: u8,
    pub field_mask: u8,
}

#[repr(C)]
#[extend(FProperty)]
#[derive(Debug, Clone)]
pub struct FObjectPropertyBase {
    pub property_class: UObjectPointer<UClass>,
}

#[repr(C)]
#[extend(FObjectPropertyBase)]
#[derive(Debug, Clone)]
pub struct FClassProperty {
    pub meta_class: UObjectPointer<UClass>,
}

#[repr(C)]
#[extend(FProperty)]
#[derive(Debug, Clone)]
pub struct FStructProperty {
    pub struct_: UObjectPointer<UStruct>,
}

#[repr(C)]
#[extend(FProperty)]
#[derive(Debug, Clone)]
pub struct FArrayProperty {
    pub inner_property: *mut FProperty,
}

#[repr(C)]
#[extend(FProperty)]
#[derive(Debug, Clone)]
pub struct FDelegateProperty {
    pub signature_function: UObjectPointer<UFunction>,
}

#[repr(C)]
#[extend(FProperty)]
#[derive(Debug, Clone)]
pub struct FMapProperty {
    pub key_property: *mut FProperty,
    pub value_property: *mut FProperty,
}

#[repr(C)]
#[extend(FProperty)]
#[derive(Debug, Clone)]
pub struct FSetProperty {
    pub element_property: *mut FProperty,
}

#[repr(C)]
#[extend(FProperty)]
#[derive(Debug, Clone)]
pub struct FEnumProperty {
    pub underlying_property: *mut FProperty,
    pub enum_: *mut UEnum,
}

#[repr(C)]
#[extend(FProperty)]
#[derive(Debug, Clone)]
pub struct FFieldPathProperty {
    pub field_class: *mut FFieldClass,
}

#[repr(C)]
#[extend(FProperty)]
#[derive(Debug, Clone)]
pub struct FOptionalProperty {
    pub value_property: *mut FProperty,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TFieldPath<T> {
    _phantom: PhantomData<T>,
    pub resolved_field: *const FField,
    pub resolved_owner: TWeakObjectPtr<UStruct>,
    pub path: TArray<FName>,
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