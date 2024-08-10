use std::cell::{Cell, UnsafeCell};
use manasdk::{UClass};

struct Parent {
    pub property: f32
}

struct Child {
    pub property: f32,
    pub property2: f32
}

struct GrandChild {
    pub property: f32,
    pub property2: f32,
    pub property3: f32
}

impl AsRef<Parent> for GrandChild {
    fn as_ref(&self) -> &Parent {
        todo!()
    }
}

fn main() {
    
}