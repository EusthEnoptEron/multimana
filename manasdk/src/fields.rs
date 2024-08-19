use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use flagset::FlagSet;
use manasdk_macros::extend;
use crate::{EClassCastFlags, EClassFlags, FName, TArray, TWeakObjectPtr, UClass, UFunction, UObject, UObjectPointer, UStruct};
use crate::core_u_object::UEnum;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FFieldClass {
    pub name: FName,
    pub id: u64,
    pub cast_flags: FlagSet<EClassCastFlags>,
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
    pub inner_property: *mut crate::FProperty,
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
    pub key_property: *mut crate::FProperty,
    pub value_property: *mut crate::FProperty,
}

#[repr(C)]
#[extend(FProperty)]
#[derive(Debug, Clone)]
pub struct FSetProperty {
    pub element_property: *mut crate::FProperty,
}

#[repr(C)]
#[extend(FProperty)]
#[derive(Debug, Clone)]
pub struct FEnumProperty {
    pub underlying_property: *mut crate::FProperty,
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
    pub value_property: *mut crate::FProperty,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TFieldPath<T> {
    _phantom: PhantomData<T>,
    pub resolved_field: *const FField,
    pub resolved_owner: TWeakObjectPtr<UStruct>,
    pub path: TArray<FName>,
}
