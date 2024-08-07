#![allow(non_camel_case_types)]

mod enums;
mod functions;

use std::ffi::c_void;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, size_of};
use widestring::WideChar;

pub use enums::*;
pub use functions::*;

include!(concat!(env!("OUT_DIR"), "/generated_code.rs"));


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

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FNameEntryHeader {}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FNameEntry {}

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
    pub class_flags: EClassFlags,
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
#[derive(Debug, Clone)]
pub struct FProperty {
    pub base: FField,
    pub array_dim: i32,
    pub element_size: i32,
    pub property_flags: u64,
    pub pad_48: [u8; 4],
    pub offset: i32,
    pub pad_50: [u8; 40],
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FByteProperty {
    pub base: FProperty,
    pub enum_: *mut UEnum,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FBoolProperty {
    pub base: FProperty,
    pub field_size: u8,
    pub byte_offset: u8,
    pub byte_mask: u8,
    pub field_mask: u8,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FObjectPropertyBase {
    pub base: FProperty,
    pub property_class: *mut UClass,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FClassProperty {
    pub base: FObjectPropertyBase,
    pub meta_class: *mut UClass,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FStructProperty {
    pub base: FProperty,
    pub struct_: *mut UStruct,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FArrayProperty {
    pub base: FProperty,
    pub inner_property: *mut FProperty,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FDelegateProperty {
    pub base: FProperty,
    pub signature_function: *mut UFunction,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FMapProperty {
    pub base: FProperty,
    pub key_property: *mut FProperty,
    pub value_property: *mut FProperty,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FSetProperty {
    pub base: FProperty,
    pub element_property: *mut FProperty,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FEnumProperty {
    pub base: FProperty,
    pub underlying_property: *mut FProperty,
    pub enum_: *mut UEnum,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FFieldPathProperty {
    pub base: FProperty,
    pub field_class: *mut FFieldClass,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FOptionalProperty {
    pub base: FProperty,
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