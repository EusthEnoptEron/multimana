use std::cell::LazyCell;
use std::collections::hash_set::IntoIter;
use std::ffi::c_void;
use std::fmt::{Debug, Formatter};
use std::mem::ManuallyDrop;
use std::sync::LazyLock;
use tracing::info;

use crate::{BASE_ADDRESS, FNameEntry, Offsets, UObject, UObjectPointer};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FNamePool {
    _padding: [u8; 8],
    current_block: u32,
    current_byte_cursor: u32,
    blocks: [usize; 0x2000],
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TArray<T> {
    pub data: *const T,
    pub num_elements: u32,
    pub max_elements: u32,
}

#[repr(C)]
#[derive(Debug, Clone)]
/** The result of a sparse array allocation. */
pub struct FSparseArrayAllocationInfo
{
    pub index: i32,
    pub pointer: *const c_void,
}

#[repr(C)]
pub union TSparseArrayElementOrFreeListLink<T> {
    pub element_data: ManuallyDrop<T>,
    pub prev_next_free_index: (i32, i32),
}

impl<T> Clone for TSparseArrayElementOrFreeListLink<T> {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl<T> Debug for TSparseArrayElementOrFreeListLink<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown")
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct InlineAllocator<const NUM_INLINE_ELEMENTS: usize, T> {
    pub data: [T; NUM_INLINE_ELEMENTS],
    pub secondary_data: *const T,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FBitArray {
    pub data: InlineAllocator<4, i32>,
    pub num_bits: i32,
    pub max_bits: i32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TSparseArray<T> {
    pub data: TArray<TSparseArrayElementOrFreeListLink<T>>,
    pub allocation_flags: FBitArray,
    pub first_free_index: i32,
    pub num_free_indices: i32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TSet<T> {
    pub elements: TSparseArray<T>,
    pub hash: InlineAllocator<1, i32>,
    pub hash_size: i32,
}


#[repr(C)]
#[derive(Debug, Clone)]
pub struct TMap<T1, T2> {
    pub elements: TSet<TPair<T1, T2>>,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TPair<T1, T2> {
    pub key: T1,
    pub value: T2,
}


#[repr(C)]
#[derive(Debug, Clone)]
pub struct FUObjectItem {
    pub object: UObjectPointer<UObject>,
    pub flags: i32,
    pub cluster_root_index: i32,
    pub serial_number: i32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TUObjectArray {
    /// Master table to chunks of pointers
    pub objects: *const *const FUObjectItem,
    /// If requested, a contiguous memory where all objects are allocated
    pub pre_allocated_objects: *const FUObjectItem,
    /// Maximum number of elements
    pub max_elements: i32,
    /// Number of elements we currently have
    pub num_elements: i32,
    /// Maximum number of chunks
    pub max_chunks: i32,
    /// Number of chunks we currently have
    pub num_chunks: i32,
}
unsafe impl Sync for TUObjectArray {}


static FNAME_POOL: LazyLock<&'static FNamePool> = LazyLock::new(|| {
    let address = *BASE_ADDRESS + Offsets::OFFSET_GNAMES;
    info!("GNames Address=0x{:x}", address);
    unsafe { (address as *const FNamePool).as_ref().expect("Unable to find GNames") }
});

impl FNamePool {
    const ENTRY_STRIDE: u32 = 0x0002;
    const BLOCK_OFFSET_BITS: u32 = 0x0010;
    const BLOCK_OFFSET: u32 = 1 << Self::BLOCK_OFFSET_BITS;

    pub fn get() -> &'static Self {
        &FNAME_POOL
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


impl TUObjectArray {
    const ELEMENTS_PER_CHUNK: usize = 0x10000;

    pub fn len(&self) -> usize {
        if self.num_elements < 0 {
            0
        } else {
            self.num_elements as usize
        }
    }

    pub fn get_by_index(&self, index: usize) -> Option<&UObject> {
        let chunk_index = index / Self::ELEMENTS_PER_CHUNK;
        let in_chunk_index = index % Self::ELEMENTS_PER_CHUNK;

        if chunk_index as i32 >= self.num_chunks || index as i32 >= self.num_elements {
            return None;
        }

        let chunk = unsafe { self.objects.add(chunk_index).as_ref() }?.clone();
        let object = &unsafe { chunk.add(in_chunk_index).as_ref() }?.object;

        object.as_ref()
    }

    pub fn iter(&self) -> impl Iterator<Item=&UObject> {
        TUObjectIter {
            index: 0,
            array: self,
        }
    }
}


struct TUObjectIter<'a> {
    index: usize,
    array: &'a TUObjectArray,
}

impl<'a> Iterator for TUObjectIter<'a> {
    type Item = &'a UObject;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.array.len() {
            let current = self.index;
            self.index += 1;

            if let Some(pointer) = self.array.get_by_index(current) {
                return Some(pointer);
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.array.len() - self.index;

        (remaining, Some(remaining))
    }
}