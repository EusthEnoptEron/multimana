use std::fmt::{Display, Formatter};
use std::ops::AddAssign;
use tracing::info;
use widestring::{decode_utf16_lossy, WideStr};
use crate::{FName, FNameEntry, FNamePool, FString, FText};

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
            string.add_assign(format!("_{}", self.number - 1).as_str());
        }

        Some(string)
    }

    pub fn to_string(&self) -> Option<String> {
        let output = self.to_raw_string()?;
        if let Some(pos) = output.rfind('/') {
            Some(output[pos + 1..].to_string())
        } else {
            Some(output)
        }
    }
}


impl Display for FString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = decode_utf16_lossy(self.data.iter().copied()).collect::<String>();
        write!(f, "{str}")
    }
}

impl Display for FText {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe { write!(f, "{}", self.text_data.as_ref().map(|it| it.text_source.to_string()).unwrap_or_default()) }
    }
}