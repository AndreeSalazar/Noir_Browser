//! Arena Allocator - Bump allocator for fast allocations
use std::cell::RefCell;
use std::rc::Rc;

pub struct Arena {
    chunks: RefCell<Vec<ArenaChunk>>,
    current: RefCell<usize>,
}

struct ArenaChunk {
    data: Vec<u8>,
    used: usize,
}

impl Arena {
    pub fn new() -> Self {
        Self {
            chunks: RefCell::new(vec![ArenaChunk::new(4096)]),
            current: RefCell::new(0),
        }
    }

    /// Allocate bytes in the arena
    pub fn alloc(&self, size: usize) -> *mut u8 {
        let mut chunks = self.chunks.borrow_mut();
        let current_idx = *self.current.borrow();
        if let Some(chunk) = chunks.get_mut(current_idx) {
            if chunk.used + size <= chunk.data.len() {
                let ptr = unsafe { chunk.data.as_mut_ptr().add(chunk.used) };
                chunk.used += size;
                return ptr;
            }
        }
        // Need new chunk
        let new_size = (size.max(4096) + 4095) & !4095;
        let mut new_chunk = ArenaChunk::new(new_size);
        let ptr = unsafe { new_chunk.data.as_mut_ptr().add(new_chunk.used) };
        new_chunk.used += size;
        chunks.push(new_chunk);
        *self.current.borrow_mut() = chunks.len() - 1;
        ptr
    }

    /// Allocate and copy string
    pub fn alloc_str(&self, s: &str) -> &str {
        let ptr = self.alloc(s.len());
        unsafe {
            std::ptr::copy_nonoverlapping(s.as_ptr(), ptr, s.len());
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, s.len()))
        }
    }

    /// Reset arena (free all allocations)
    pub fn reset(&self) {
        for chunk in self.chunks.borrow_mut().iter_mut() {
            chunk.used = 0;
        }
        *self.current.borrow_mut() = 0;
    }

    /// Total bytes allocated
    pub fn used_bytes(&self) -> usize {
        self.chunks.borrow().iter().map(|c| c.used).sum()
    }
}

impl ArenaChunk {
    fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            used: 0,
        }
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}
