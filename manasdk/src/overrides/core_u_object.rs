use flagset::FlagSet;
use manasdk_macros::{extend, HasClassObject};
use crate::{EClassCastFlags, EFunctionFlags, EObjectFlags, EPropertyFlags, FField, FName, FNativeFuncPtr, UObjectPointer};

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
#[derive(Debug, Clone, HasClassObject)]
pub struct UField {
    pub next: UObjectPointer<UField>,
}

#[repr(C)]
#[extend(UField)]
#[derive(Debug, Clone, HasClassObject)]
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
#[derive(Debug, Clone, HasClassObject)]
pub struct UClass {
    pub _pad_1: [u8; 0x20],
    pub cast_flags: FlagSet<EClassCastFlags>,
    pub _pad_2: [u8; 0x40],
    pub default_object: UObjectPointer<UObject>,
    pub _pad_3: [u8; 0x110],
}


#[repr(C)]
#[extend(UStruct)]
#[derive(Debug, Clone, HasClassObject)]
pub struct UFunction {
    pub function_flags: FlagSet<EFunctionFlags>,
    pub rep_offset: i16,
    pub num_parms: u8,
    pub parms_size: u16,
    pub return_value_offset: u16,
    pub _padding_300: [u8; 27],
    pub exec_function: FNativeFuncPtr,
}

#[repr(C)]
#[extend(UField)]
#[derive(Debug, Clone, HasClassObject)]
pub struct UProperty {
    pub array_dim: i32,
    pub element_size: i32,
    pub property_flags: FlagSet<EPropertyFlags>,
    pub rep_index: u16,
    pub blueprint_replication_condition: u8,
    pub offset_internal: i32,

    pub _padding_200: [u8; 33usize],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u_class() {
        assert_eq!(size_of::<UClass>(), 560usize);
    }

    #[test]
    fn test_u_function() {
        assert_eq!(size_of::<UFunction>(), 224usize);
    }

    #[test]
    fn test_UProperty() {
        assert_eq!(size_of:: < UProperty > (), 112usize);
    }
}