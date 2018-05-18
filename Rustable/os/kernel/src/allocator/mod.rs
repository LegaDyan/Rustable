mod linked_list;
pub mod page;
pub mod util;

#[path = "first_fit.rs"]
mod imp;

#[cfg(test)]
mod tests;

pub use self::page::Page;

use mutex::Mutex;
use alloc::heap::{Alloc, AllocErr, Layout};
use std::cmp::max;

use pi::atags::Atags;

/// Thread-safe (locking) wrapper around a particular memory allocator.
// #[derive(Debug)]
pub struct Allocator(Mutex<Option<imp::Allocator>>);

impl Allocator {
    /// Returns an uninitialized `Allocator`.
    ///
    /// The allocator must be initialized by calling `initialize()` before the
    /// first memory allocation. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        Allocator(Mutex::new(None))
    }

    /// Initializes the memory allocator.
    ///
    /// # Panics
    ///
    /// Panics if the system's memory map could not be retrieved.
    pub fn initialize(&self) {
        // let (start, end) = memory_map().expect("failed to find memory map");
        *self.0.lock() = Some(imp::Allocator::new());
    }

    pub fn init_memmap(&self, base: usize, npage: usize, begin: usize) {
        self.0.lock().as_mut().expect("allocator uninitialized").init_memmap(base, npage, begin);
    }
}

unsafe impl<'a> Alloc for &'a Allocator {

    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        self.0.lock().as_mut().expect("allocator uninitialized").alloc(layout)
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        self.0.lock().as_mut().expect("allocator uninitialized").dealloc(ptr, layout);
    }
}

extern "C" {
    static _end: u8;
}

/// Returns the (start address, end address) of the available memory on this
/// system if it can be determined. If it cannot, `None` is returned.
///
/// This function is expected to return `Some` under all normal cirumstances.
fn memory_map() -> Option<(usize, usize)> {
    let binary_end = unsafe { (&_end as *const u8) as u32 };
    for atag in Atags::get() {
        match atag.mem() {
            Some(mem) => {
                let start_addr = max(mem.start, binary_end) as usize;
                let end_addr = (start_addr + mem.size as usize) as usize;
                return Some((start_addr, end_addr));
            },
            None => {}
        }
    }
    None
}


