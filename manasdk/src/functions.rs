use std::ops::{Deref, DerefMut};
use crate::{APlayerController, UClass, UFunction, UObject, UStruct};
//
// trait UObjectLike {
//     fn cast(&self) -> &UObject {
//         unsafe {
//             (self as *const UObject).as_ref().unwrap()
//         }
//     }
//
//     fn cast_mut(&mut self) -> &mut UObject {
//         unsafe {
//             (self as *mut UObject).as_mut().unwrap()
//         }
//     }
//
//     fn get_name(&self) {
//         todo!();
//         // self.cast().
//     }
// }
// impl UObjectLike for UObject {
//
// }

impl UClass {
    fn get_function_mut(&self, class_name: &str, func_name: &str, controller: &APlayerController) -> Option<&mut UFunction> {
        let clss = Some(self);
        
        while let Some(current) = clss {

        }


        None
    }

}