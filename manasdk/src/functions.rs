use std::ffi::c_void;
use std::fmt::{Display, Formatter};
use std::iter::once;
use std::sync::LazyLock;

use crate::core_u_object::{UField, UFunction, UStruct};
use crate::{offsets, EClassCastFlags, EObjectFlags, FName, FProperty, HasClassObject, TUObjectArray, UClass, UObject, UObjectPointer, BASE_ADDRESS};
use dashmap::DashMap;
use flagset::FlagSet;

thread_local! {
    static CLASS_CACHE: DashMap<String, Option<&'static UClass>> = DashMap::new();
}

#[derive(Clone, Copy, Debug)]
pub enum PointerError {
    NullPointer,
    NotValid,
}

impl Display for PointerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for PointerError {}

impl<T: AsRef<UObject>> UObjectPointer<T> {
    pub fn is_same<T2>(&self, other: &UObjectPointer<T2>) -> bool
    where
        T2: AsRef<UObject>,
    {
        other.0 as usize == self.0 as usize
    }

    pub fn as_ref(&self) -> Option<&T> {
        unsafe { self.0.as_ref() }
    }

    pub fn try_as_ref(&self) -> Result<&T, PointerError> {
        self.as_ref().ok_or(PointerError::NullPointer)
    }

    pub fn as_mut(&mut self) -> Option<&mut T> {
        unsafe { self.0.as_mut() }
    }

    pub fn try_as_mut(&mut self) -> Result<&mut T, PointerError> {
        self.as_mut().ok_or(PointerError::NullPointer)
    }

    pub fn try_get<'a>(self) -> Result<&'a mut T, PointerError> {
        unsafe { self.0.as_mut() }.ok_or(PointerError::NullPointer)
    }

    pub fn name(&self) -> String {
        if let Some(obj) = self.as_ref() {
            obj.as_ref().name()
        } else {
            "NULL".into()
        }
    }
}

static UOBJECT: LazyLock<&'static TUObjectArray> = LazyLock::new(|| unsafe {
    ((*BASE_ADDRESS + offsets::OFFSET_GOBJECTS) as *const TUObjectArray)
        .as_ref()
        .expect("Unable to find GObjects")
});

impl UObject {
    pub fn all() -> &'static TUObjectArray {
        &UOBJECT
    }

    pub fn cast<T: HasClassObject>(&self) -> Option<&T> {
        let class = T::static_class();

        if self.is_a(class) {
            Some(unsafe { std::mem::transmute(self as *const Self) })
        } else {
            None
        }
    }

    pub fn cast_mut<T: HasClassObject>(&mut self) -> Option<&mut T> {
        let class = T::static_class();

        if self.is_a(class) {
            Some(unsafe { std::mem::transmute(self as *const Self) })
        } else {
            None
        }
    }
    
    pub fn find_object(
        required_type: impl Into<FlagSet<EClassCastFlags>> + Copy,
        predicate: impl Fn(&UObject) -> bool,
    ) -> Option<&'static UObject> {
        Self::all()
            .iter()
            .find(|it| it.has_type_flag(required_type) && predicate(it))
    }

    pub unsafe fn find_object_of_type<T>(
        required_type: impl Into<FlagSet<EClassCastFlags>> + Copy,
        predicate: impl Fn(&UObject) -> bool,
    ) -> Option<&'static T> {
        Self::find_object(required_type, predicate).map(|it| std::mem::transmute(it))
    }

    pub fn find_function(predicate: impl Fn(&UFunction) -> bool) -> Option<&'static UFunction> {
        unsafe {
            Self::find_object_of_type(EClassCastFlags::Function, |it| {
                predicate(std::mem::transmute(it))
            })
        }
    }

    pub fn name(&self) -> String {
        self.name.to_string().unwrap_or("None".to_string())
    }

    /// Returns the name of this object in the format 'Class Package.Outer.Object'
    pub fn full_name(&self) -> String {
        if let Some(class) = self.class.as_ref() {
            let mut hierarchy = once(self)
                .chain(self.iter_outers())
                .map(|it| it.name())
                .collect::<Vec<_>>();
            hierarchy.reverse();

            format!("{} {}", class.name(), hierarchy.join("."))
        } else {
            "None".to_string()
        }
    }
    
    pub fn class_hierarchy(&self) -> String {
        if let Some(class) = self.class.as_ref() {
            let mut hierarchy = once(class)
                .chain(class.iter_parents().filter_map(|it| it.cast::<UClass>()))
                .map(|it| it.name())
                .collect::<Vec<_>>();
            hierarchy.reverse();
            
            
            hierarchy.join(".")
        } else {
            "None".to_string()
        }
    }

    pub fn is_a(&self, class: &UClass) -> bool {
        self.class
            .as_ref()
            .map(|it| it.is_subclass_of(class))
            .unwrap_or_default()
    }
}

type ProcessEventFn = extern "C" fn(*const UObject, *const UFunction, *mut c_void);

impl UObject {
    pub fn has_type_flag(&self, flags: impl Into<FlagSet<EClassCastFlags>>) -> bool {
        self.class
            .as_ref()
            .map(|it| it.cast_flags.contains(flags))
            .unwrap_or_default()
    }

    pub fn iter_outers(&self) -> impl Iterator<Item = &UObject> {
        StructTraverser {
            current: self.outer.as_ref(),
            get_next: &|el| el.outer.as_ref(),
        }
    }

    pub(crate) fn process_event<T>(&self, func: &UFunction, parms: &mut T) {
        // Cast the v_table to a pointer to a pointer (vptr) to an array of function pointers (vtable).
        let v_table = self.v_table as *const *const c_void;

        // Safely obtain the function pointer from the vtable.
        // v_table.add(Offsets::INDEX_PROCESSEVENT) returns a pointer to the function pointer,
        // so we dereference it to get the actual function pointer.
        let fn_ptr = unsafe {
            let process_event_ptr = *v_table.add(offsets::INDEX_PROCESSEVENT);
            std::mem::transmute::<*const c_void, ProcessEventFn>(process_event_ptr)
        };

        // Call the function using the function pointer.
        let this = self;
        fn_ptr(this, func, parms as *mut T as *mut c_void);
    }

    pub fn is_default_obj(&self) -> bool {
        !self.flags.contains(EObjectFlags::ClassDefaultObject)
    }
}

impl UStruct {
    /// Iterate though the parents of this struct.
    pub fn iter_parents(&self) -> impl Iterator<Item = &UStruct> {
        StructTraverser {
            current: self.super_.as_ref(),
            get_next: &|el| el.super_.as_ref(),
        }
    }

    /// Iterate through the child fields of this struct.
    pub fn iter_children(&self) -> impl Iterator<Item = &UField> {
        StructTraverser {
            current: self.children.as_ref(),
            get_next: &|el| el.next.as_ref(),
        }
    }

    pub fn is_subclass_of(&self, base: &UStruct) -> bool {
        once(self)
            .chain(self.iter_parents())
            .any(|it| std::ptr::eq(it, base))
    }
}

impl UClass {
    pub fn find_function_by_name(&self, func_name: &FName) -> Option<&UFunction> {
        once::<&UStruct>(self)
            .chain(self.iter_parents())
            .flat_map(|parent| parent.iter_children())
            .filter(|child| child.has_type_flag(EClassCastFlags::Function))
            .find(|child| {
                child.name.comparison_index == func_name.comparison_index
                    && child.name.number == func_name.number
            })
            .map(|child| unsafe { std::mem::transmute(child) })
    }

    pub fn find_function(&self, func_name: &str) -> Option<&UFunction> {
        once::<&UStruct>(self)
            .chain(self.iter_parents())
            .flat_map(|parent| parent.iter_children())
            .filter(|child| child.has_type_flag(EClassCastFlags::Function))
            .find(|child| child.name() == func_name)
            .map(|child| unsafe { std::mem::transmute(child) })
    }

    pub fn find_function_mut(&self, func_name: &str) -> Option<&mut UFunction> {
        once::<&UStruct>(self)
            .chain(self.iter_parents())
            .flat_map(|parent| parent.iter_children())
            .filter(|child| child.has_type_flag(EClassCastFlags::Function))
            .find(|child| child.name() == func_name)
            .map(|child| unsafe { std::mem::transmute(child as *const UField) })
    }

    pub fn find(name: &str) -> Option<&'static UClass> {
        CLASS_CACHE.with(|map| {
            let class = map.entry(name.to_string()).or_insert_with(|| unsafe {
                UObject::find_object_of_type(EClassCastFlags::Class, |obj| obj.name() == name)
            });

            *class.value()
        })
    }
}

impl UFunction {
    pub fn child_properties(&self) -> impl Iterator<Item = &FProperty> {
        StructTraverser {
            current: unsafe { (self.child_properties as *const FProperty).as_ref() },
            get_next: &|prop| unsafe { (prop.next as *const FProperty).as_ref() },
        }
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
