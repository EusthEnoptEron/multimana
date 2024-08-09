use flagset::FlagSet;
use crate::{EClassCastFlags, UClass, UFunction, UObject, UObjectPointer, UStruct};


impl<T: AsRef<UObject>> UObjectPointer<T> {
    pub fn as_ref(&self) -> Option<&T> {
        return unsafe { self.0.as_ref() };
    }

    pub fn as_mut(&mut self) -> Option<&mut T> {
        return unsafe { self.0.as_mut() };
    }
}


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

        while let Some(current) = clss {
            if current.get_name() != class_name {
                clss = current.super_.as_ref();
                continue;
            }

            let mut child = current.children.as_ref();
            while let Some(child_) = child {
                if child_.has_type_flag(EClassCastFlags::Function) && child_.get_name() == func_name {
                    return unsafe { Some(std::mem::transmute(child_)) };
                }

                child = child_.next.as_ref();
            }
        }


        None
    }
}