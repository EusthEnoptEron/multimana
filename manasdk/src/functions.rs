use std::cell::LazyCell;
use std::fmt::{Debug, Display};
use std::ops::{AddAssign, Deref};

use flagset::FlagSet;
use serde::de::Expected;
use widestring::WideStr;

use crate::{EClassCastFlags, FName, FNameEntry, FNamePool, Offsets, UClass, UField, UFunction, UObject, UObjectPointer, UStruct};

impl<T: AsRef<UObject>> UObjectPointer<T> {
    pub fn as_ref(&self) -> Option<&T> {
        return unsafe { self.0.as_ref() };
    }

    pub fn as_mut(&mut self) -> Option<&mut T> {
        return unsafe { self.0.as_mut() };
    }
}


impl FNamePool {
    const INSTANCE: LazyCell<&'static FNamePool> = LazyCell::new(|| {
        unsafe { (Offsets::OFFSET_GNAMES as *const FNamePool).as_ref().unwrap() }
    });
    const ENTRY_STRIDE: u32 = 0x0002;
    const BLOCK_OFFSET_BITS: u32 = 0x0010;
    const BLOCK_OFFSET: u32 = 1 << Self::BLOCK_OFFSET_BITS;

    pub fn get() -> &'static Self {
        *Self::INSTANCE.deref()
    }

    pub fn is_valid_index(&self, index: u32, chunk_idx: u32, in_chunk_idx: u32) -> bool {
        chunk_idx <= self.current_block && !(chunk_idx == self.current_block && in_chunk_idx > self.current_byte_cursor)
    }

    pub fn entry_by_index(&self, index: u32) -> Option<&FNameEntry> {
        let chunk_index = index >> Self::BLOCK_OFFSET_BITS;
        let in_chunk = index & (Self::BLOCK_OFFSET - 1);

        if self.is_valid_index(index, chunk_index, in_chunk) {
            let address = self.blocks[chunk_index as usize] + (in_chunk * Self::ENTRY_STRIDE) as usize;
            let pointer: *const FNameEntry = unsafe {
                std::mem::transmute(address)
            };

            unsafe {
                pointer.as_ref()
            }
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum UnrealString<'a> {
    Ascii(&'a str),
    Wide(&'a WideStr),
}

impl<'a> UnrealString<'a> {
    pub fn len(&self) -> usize {
        match self {
            UnrealString::Ascii(ascii) => { ascii.len() }
            UnrealString::Wide(wide) => { wide.len() }
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            UnrealString::Ascii(ascii) => { ascii.to_string() }
            UnrealString::Wide(wide) => { wide.to_string().unwrap() }
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let Self::Ascii(as_str) = self {
            Some(as_str)
        } else {
            None
        }
    }

    pub fn as_widestr(&self) -> Option<&WideStr> {
        if let Self::Wide(as_str) = self {
            Some(as_str)
        } else {
            None
        }
    }
}

impl FNameEntry {
    pub fn is_wide(&self) -> bool {
        self.header.b_is_wide() > 0
    }

    pub fn get_string(&self) -> UnrealString {
        let len = self.header.len() as usize;
        if self.is_wide() {
            UnrealString::Wide(unsafe { WideStr::from_slice(&self.name.wide_name[0..len]) })
        } else {
            UnrealString::Ascii(unsafe { std::mem::transmute(&self.name.ansi_name[0..len]) })
        }
    }
}

impl FName {
    pub fn display_index(&self) -> u32 {
        self.comparison_index as u32
    }

    pub fn to_raw_string(&self) -> Option<String> {
        if self.comparison_index < 0 {
            return None;
        }

        let name = FNamePool::get().entry_by_index(self.display_index())?;
        let mut string = name.get_string().to_string();

        if self.number > 0 {
            string.add_assign(format!("_{}", self.number).as_str());
        }

        Some(string)
    }
    
    pub fn to_string(&self) -> Option<String> {
        let output = self.to_raw_string()?;
        if let Some(pos) = output.rfind('/') {
            Some(output[0..pos].to_string())
        } else {
            Some(output)
        }
    }
}

impl UObject {
    fn name(&self) -> String {
        self.name.to_string().unwrap_or_default()
    }
}

impl UObject {
    pub fn has_type_flag(&self, flags: impl Into<FlagSet<EClassCastFlags>>) -> bool {
        self.class.as_ref().map(|it| it.cast_flags.contains(flags)).unwrap_or_default()
    }
}

impl UStruct {
    /// Iterate though the parents of this struct.
    pub fn iter_parents(&self, inclusive: bool) -> impl Iterator<Item=&UStruct> {
        StructTraverser {
            current_el: if(inclusive) { Some(self) } else { self.super_.as_ref() },
            delegate: &|el| {
                el.super_.as_ref()
            },
        }
    }

    /// Iterate through the child fields of this struct.
    pub fn iter_children(&self) -> impl Iterator<Item=&UField> {
        StructTraverser {
            current_el: self.children.as_ref(),
            delegate: &|el| {
                el.next.as_ref()
            },
        }
    }
}

impl UClass {
    pub fn get_function(&self, class_name: &str, func_name: &str) -> Option<&UFunction> {
        self.iter_parents(true)
            .filter(|parent| parent.name() == class_name)
            .flat_map(|parent| parent.iter_children())
            .filter(|child| child.has_type_flag(EClassCastFlags::Function))
            .find(|child| child.name() == func_name)
            .map(|child| unsafe { std::mem::transmute(child) })
    }
}

struct StructTraverser<'a, 'b, T, Delegate: Fn(&T) -> Option<&T>> {
    current_el: Option<&'b T>,
    delegate: &'a Delegate
}

impl<'a, 'b, T, Delegate: Fn(&T) -> Option<&T>> Iterator for StructTraverser<'a, 'b, T, Delegate> {
    type Item = &'b T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current_el?;
        self.current_el = (self.delegate)(current);
        
        Some(current)
    }
}