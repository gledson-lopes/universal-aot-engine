use std::alloc::{Layout, alloc, dealloc};
use std::cell::Cell;
use std::ptr::NonNull;

/// Bump allocator arena - O(1) allocation, bulk deallocation
pub struct Arena {
    ptr: Cell<*mut u8>,
    end: *mut u8,
    chunks: Vec<(NonNull<u8>, Layout)>,
}

impl Arena {
    pub fn new(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, 8).unwrap();
        let ptr = unsafe { alloc(layout) };
        Self {
            ptr: Cell::new(ptr),
            end: unsafe { ptr.add(capacity) },
            chunks: vec![(NonNull::new(ptr).unwrap(), layout)],
        }
    }

    /// Bump allocation - no locks, no metadata per allocation
    pub fn alloc(&self, size: usize, align: usize) -> *mut u8 {
        let align_mask = align - 1;
        let current = self.ptr.get();
        let aligned = ((current as usize + align_mask) & !align_mask) as *mut u8;
        let next = unsafe { aligned.add(size) };

        if next > self.end {
            panic!("Arena out of memory");
        }

        self.ptr.set(next);
        aligned
    }

    /// Allocate enum: [Tag: u32][Payload...]
    pub fn alloc_enum(&self, tag: u32, payload: &[i32]) -> *mut u8 {
        let size = 4 + payload.len() * 4;
        let ptr = self.alloc(size, 4);

        unsafe {
            *(ptr as *mut u32) = tag;
            let payload_ptr = ptr.add(4) as *mut i32;
            std::ptr::copy_nonoverlapping(payload.as_ptr(), payload_ptr, payload.len());
        }

        ptr
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        // 📢 Visual confirmation showing the number of chunks being reallocated/freed
        println!(
            "\x1b[31m[Arena]\x1b[0m Freeing {} backing chunk(s) back to the system memory allocator...",
            self.chunks.len()
        );

        for (ptr, layout) in self.chunks.drain(..) {
            unsafe {
                dealloc(ptr.as_ptr(), layout);
            }
        }
    }
}
