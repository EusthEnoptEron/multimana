use std::cell::LazyCell;
use std::fmt::{Debug, Display};
use std::iter::once;
use flagset::FlagSet;
use serde::de::Expected;

use crate::{EClassCastFlags, Offsets, TUObjectArray, UClass, UField, UFunction, UObject, UObjectPointer, UStruct};

impl<T: AsRef<UObject>> UObjectPointer<T> {
    pub fn as_ref(&self) -> Option<&T> {
        return unsafe { self.0.as_ref() };
    }

    pub fn as_mut(&mut self) -> Option<&mut T> {
        return unsafe { self.0.as_mut() };
    }
}


impl UObject {
    const INSTANCE: LazyCell<&'static TUObjectArray> = LazyCell::new(|| {
        unsafe { (Offsets::OFFSET_GOBJECTS as *const TUObjectArray).as_ref().expect("Unable to find GObjects") }
    });

    pub fn all() -> &'static TUObjectArray {
        &Self::INSTANCE
    }

    pub fn find_object(predicate: impl Fn(&UObject) -> bool, required_type: impl Into<FlagSet<EClassCastFlags>> + Copy) -> Option<&'static UObject> {
        Self::all().iter()
            .find(|it| it.has_type_flag(required_type) && predicate(it))
    }

    pub fn name(&self) -> String {
        self.name.to_string().unwrap_or_default()
    }

    /// Returns the name of this object in the format 'Class Package.Outer.Object'
    pub fn full_name(&self) -> String {
        if let Some(class) = self.class.as_ref() {
            let mut hierarchy = once(self).chain(self.iter_outers()).map(|it| it.name()).collect::<Vec<_>>();
            hierarchy.reverse();

            format!("{} {}", class.name(), hierarchy.join("."))
        } else {
            "None".to_string()
        }
    }
}

impl UObject {
    pub fn has_type_flag(&self, flags: impl Into<FlagSet<EClassCastFlags>>) -> bool {
        self.class.as_ref().map(|it| it.cast_flags.contains(flags)).unwrap_or_default()
    }

    pub fn iter_outers(&self) -> impl Iterator<Item=&UObject> {
        StructTraverser {
            current: self.outer.as_ref(),
            get_next: &|el| {
                el.outer.as_ref()
            },
        }
    }
}

impl UStruct {
    /// Iterate though the parents of this struct.
    pub fn iter_parents(&self) -> impl Iterator<Item=&UStruct> {
        StructTraverser {
            current: self.super_.as_ref(),
            get_next: &|el| {
                el.super_.as_ref()
            },
        }
    }

    /// Iterate through the child fields of this struct.
    pub fn iter_children(&self) -> impl Iterator<Item=&UField> {
        StructTraverser {
            current: self.children.as_ref(),
            get_next: &|el| {
                el.next.as_ref()
            },
        }
    }
}

impl UClass {
    pub fn find_function(&self, class_name: &str, func_name: &str) -> Option<&UFunction> {
        once::<&UStruct>(self).chain(self.iter_parents())
            .filter(|parent| parent.name() == class_name)
            .flat_map(|parent| parent.iter_children())
            .filter(|child| child.has_type_flag(EClassCastFlags::Function))
            .find(|child| child.name() == func_name)
            .map(|child| unsafe { std::mem::transmute(child) })
    }
}

struct StructTraverser<'a, 'b, T, Delegate: Fn(&T) -> Option<&T>> {
    current: Option<&'b T>,
    get_next: &'a Delegate,
}

impl<'a, 'b, T, Delegate: Fn(&T) -> Option<&T>> Iterator for StructTraverser<'a, 'b, T, Delegate> {
    type Item = &'b T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        self.current = (self.get_next)(current);

        Some(current)
    }
}