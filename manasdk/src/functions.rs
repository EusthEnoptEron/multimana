use std::cell::UnsafeCell;
use flagset::FlagSet;
use crate::{EClassCastFlags, UClass, UFunction, UObject, UStruct};
use crate::EClassCastFlags::PlayerController;
use crate::ETargetFlags::ETargetFlags_MAX;

impl UObject {
    fn get_name(&self) -> &str {
        todo!()
    }
}

impl UObject {
    fn has_type_flag(&self, flags: impl Into<FlagSet<EClassCastFlags>>) -> bool {
        unsafe {
            self.class.as_ref().map(|it| it.cast_flags.contains(flags)).unwrap_or_default()
        }
    }
}

impl UClass {
    fn get_function(&self, class_name: &str, func_name: &str) -> Option<&UFunction> {
        let mut clss: Option<&UStruct> = Some(self);

        unsafe {
            while let Some(current) = clss {
                if current.get_name() != class_name {
                    clss = current.super_.as_ref();
                    continue;
                }

                let mut child = current.children.as_ref();
                while let Some(child_) = child {
                    if child_.has_type_flag(EClassCastFlags::Function) && child_.get_name() == func_name {
                        return Some(std::mem::transmute(child_));
                    }
                    
                    child = child_.next.as_ref();
                }
            }
        }


        None
    }
}