#![no_std]
use core::alloc::{GlobalAlloc, Layout};
use corundum::open_flags::*;
use corundum::*;

type P = corundum::default::Allocator;

pub struct CorundumAlloc;

unsafe impl GlobalAlloc for CorundumAlloc {
    #[inline(always)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        P::alloc(layout.size()).0
    }

    /// De-allocate the memory at the given address with the given alignment and size.
    /// The client must assure the following things:
    /// - the memory is acquired using the same allocator and the pointer points to the start position.
    /// - Other constrains are the same as the rust standard library.
    /// The program may be forced to abort if the constrains are not full-filled.
    #[inline(always)]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        P::dealloc(ptr, layout.size())
    }

    /// Behaves like alloc, but also ensures that the contents are set to zero before being returned.
    #[inline(always)]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        P::alloc_zeroed(layout.size())
    }

    /// Re-allocate the memory at the given address with the given alignment and size.
    /// On success, it returns a pointer pointing to the required memory address.
    /// The memory content within the `new_size` will remains the same as previous.
    /// On failure, it returns a null pointer. In this situation, the previous memory is not returned to the allocator.
    /// The client must assure the following things:
    /// - the memory is acquired using the same allocator and the pointer points to the start position
    /// - `alignment` fulfills all the requirements as `rust_alloc`
    /// - Other constrains are the same as the rust standard library.
    /// The program may be forced to abort if the constrains are not full-filled.
    #[inline(always)]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if ptr.is_null() {
            return P::alloc(new_size).0;
        }

        // P::new_copy(ptr.as_ref().unwrap(), self.j).as_ptr()
        let x = ptr.as_ref().unwrap();

        let (p, off, len, z) = P::pre_alloc(new_size);
        if p.is_null() {
            panic!("Memory exhausted");
        }
        P::drop_on_failure(off, len, z);
        core::ptr::copy_nonoverlapping(x as *const u8, p, new_size);
        P::perform(z);
        utils::read(p)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_frees_allocated_memory() {
        unsafe {
            let layout = Layout::from_size_align(8, 8).unwrap();
            let alloc = CorundumAlloc;
            let _pool = P::open_no_root("/mnt/pmem0p1/test.poll", O_CF | O_8GB).unwrap();
            let ptr = alloc.alloc(layout);
            alloc.dealloc(ptr, layout);
        }
    }

    #[test]
    fn it_allocs_on_pool() {
        let len: usize = 32;
        let cnt: usize = 2;
        let _pool = P::open_no_root("/mnt/pmem0p1/test1.poll", O_CF | O_8GB).unwrap();
        unsafe {
            P::alloc(len);
        };
    }
    #[test]
    fn it_frees_zero_allocated_memory() {
        unsafe {
            let layout = Layout::from_size_align(8, 8).unwrap();
            let alloc = CorundumAlloc;
            let _pool = P::open_no_root("/mnt/pmem0p1/test2.poll", O_CF | O_8GB).unwrap();
            let ptr = alloc.alloc_zeroed(layout);
            alloc.dealloc(ptr, layout);
        }
    }

    #[test]
    fn it_frees_reallocated_memory() {
        unsafe {
            let layout = Layout::from_size_align(8, 8).unwrap();
            let alloc = CorundumAlloc;
            let _pool = P::open_no_root("/mnt/pmem0p1/test3.poll", O_CF | O_8GB).unwrap();
            let ptr = alloc.alloc(layout);
            let ptr = alloc.realloc(ptr, layout, 16);
            alloc.dealloc(ptr, layout);
        }
    }
}
