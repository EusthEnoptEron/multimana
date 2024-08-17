use either::Either;
use flagset::FlagSet;
use manasdk::{EClassCastFlags, FProperty, FRotator, FStructProperty, FVector, FVector2D, UObject, UProperty, UScriptStruct, UStruct, UStructProperty};
use std::ffi::c_void;

pub fn to_string(prop: &UProperty, result: *const c_void) -> String {
    to_string_internal(Either::Left(prop), result)
}


pub fn to_string_fproperty(prop: &FProperty, result: *const c_void) -> String {
    to_string_internal(Either::Right(prop), result)
}

fn to_string_internal(prop: Either<&UProperty, &FProperty>, result: *const c_void) -> String {
    let flags: FlagSet<EClassCastFlags> = match prop {
        Either::Left(prop) => prop.class.as_ref().unwrap().cast_flags,
        Either::Right(prop) => unsafe { prop.class_private.as_ref() }.unwrap().cast_flags
    };

    unsafe {
        if result.is_null() {
            return "NULL".to_string();
        }

        for flag in flags {
            return match flag {
                EClassCastFlags::Int8Property => {
                    (*(result as *const i8)).to_string()
                }
                EClassCastFlags::ByteProperty => {
                    (*(result as *const u8)).to_string()
                }
                EClassCastFlags::IntProperty => {
                    (*(result as *const i32)).to_string()
                }
                EClassCastFlags::FloatProperty => {
                    (*(result as *const f32)).to_string()
                }
                EClassCastFlags::UInt64Property => {
                    (*(result as *const u64)).to_string()
                }
                EClassCastFlags::UInt32Property => {
                    (*(result as *const u32)).to_string()
                }
                EClassCastFlags::NameProperty => {
                    (*(result as *const u32)).to_string()
                }
                EClassCastFlags::StrProperty => {
                    // Handle StrProperty casting and conversion logic
                    "StrProperty".to_string()
                }
                EClassCastFlags::ObjectProperty => {
                    let pointer = *(result as *const *const UObject);
                    if let Some(object) = pointer.as_ref() {
                        object.name().to_string()
                    } else {
                        "NULL".to_string()
                    }
                }
                EClassCastFlags::BoolProperty => {
                    (*(result as *const bool)).to_string()
                }
                EClassCastFlags::UInt16Property => {
                    (*(result as *const u16)).to_string()
                }
                EClassCastFlags::StructProperty => {
                    let struct_: Option<&UStruct> = unsafe {
                        match prop {
                            Either::Left(prop) => {
                                prop.cast::<UStructProperty>().as_ref().and_then(|it| {
                                    let struct_: *const UScriptStruct = std::mem::transmute(it._padding_300);
                                    struct_.as_ref().and_then(|it| it.cast::<UStruct>())
                                })
                            }
                            Either::Right(prop) => {
                                let prop: &FStructProperty = std::mem::transmute(prop);
                                prop.struct_.as_ref()
                            }
                        }
                    };

                    let struct_name = struct_.map(|it| it.name());

                    if let Some(struct_name) = struct_name {
                        if struct_name == "Vector" {
                            if let Some(v) = (result as *const FVector).as_ref() {
                                format!("[{}, {}, {}]", v.x, v.y, v.z)
                            } else {
                                "Vector?!".to_string()
                            }
                        } else if struct_name == "Vector2D" {
                            if let Some(v) = (result as *const FVector2D).as_ref() {
                                format!("[{}, {}]", v.x, v.y)
                            } else {
                                "Vector2D?!".to_string()
                            }
                        } else if struct_name == "Rotator" {
                            if let Some(r) = (result as *const FRotator).as_ref() {
                                format!("[{}, {}, {}]", r.pitch, r.yaw, r.roll)
                            } else {
                                "Rotator?!".to_string()
                            }
                        } else {
                            format!("{} (Struct)", struct_name)
                        }
                    } else {
                        "Struct?".to_string()
                    }
                }
                EClassCastFlags::ArrayProperty => {
                    // Handle ArrayProperty casting and conversion logic
                    "ArrayProperty".to_string()
                }
                EClassCastFlags::Int64Property => {
                    (*(result as *const i64)).to_string()
                }
                EClassCastFlags::TextProperty => {
                    // Handle TextProperty casting and conversion logic
                    "TextProperty".to_string()
                }
                EClassCastFlags::Int16Property => {
                    (*(result as *const i16)).to_string()
                }
                EClassCastFlags::DoubleProperty => {
                    (*(result as *const f64)).to_string()
                }
                EClassCastFlags::MapProperty => {
                    // Handle MapProperty casting and conversion logic
                    "MapProperty".to_string()
                }
                EClassCastFlags::SetProperty => {
                    // Handle SetProperty casting and conversion logic
                    "SetProperty".to_string()
                }
                EClassCastFlags::EnumProperty => {
                    // Handle EnumProperty casting and conversion logic
                    "Enum".to_string()
                }
                EClassCastFlags::FMulticastInlineDelegateProperty => {
                    // Handle FMulticastInlineDelegateProperty casting and conversion logic
                    "FMulticastInlineDelegateProperty".to_string()
                }
                EClassCastFlags::FMulticastSparseDelegateProperty => {
                    // Handle FMulticastSparseDelegateProperty casting and conversion logic
                    "FMulticastSparseDelegateProperty".to_string()
                }
                EClassCastFlags::FFieldPathProperty => {
                    // Handle FFieldPathProperty casting and conversion logic
                    "FFieldPathProperty".to_string()
                }
                EClassCastFlags::FLargeWorldCoordinatesRealProperty => {
                    // Handle FLargeWorldCoordinatesRealProperty casting and conversion logic
                    "FLargeWorldCoordinatesRealProperty".to_string()
                }
                EClassCastFlags::FOptionalProperty => {
                    // Handle FOptionalProperty casting and conversion logic
                    "FOptional".to_string()
                }
                EClassCastFlags::FVValueProperty => {
                    // Handle FVValueProperty casting and conversion logic
                    "FVValueProperty".to_string()
                }
                EClassCastFlags::FVRestValueProperty => {
                    // Handle FVRestValueProperty casting and conversion logic
                    "FVRestValueProperty".to_string()
                }
                EClassCastFlags::Enum => {
                    // Handle Enum casting and conversion logic
                    "Enum".to_string()
                }
                _ => {
                    continue;
                }
            }
        }
        
        "???".to_string()
    }
}