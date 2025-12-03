// Physical Memory Manager - Minimal Bitmap Allocator
// BSD 3-Clause License

#![no_std]

const PAGE_SIZE: usize = 4096;
const MAX_PAGES: usize = 32768; // 128MB
static mut BITMAP: [u64; MAX_PAGES / 64] = [0; MAX_PAGES / 64];
static mut NEXT_PAGE: usize = 256; // Start after kernel (1MB)

pub fn init() {
    unsafe {
        // Mark first 256 pages as used (kernel space)
        for i in 0..4 {
            BITMAP[i] = !0;
        }
    }
}

pub fn alloc_page() -> Option<usize> {
    unsafe {
        for i in NEXT_PAGE..MAX_PAGES {
            let idx = i / 64;
            let bit = i % 64;
            if BITMAP[idx] & (1 << bit) == 0 {
                BITMAP[idx] |= 1 << bit;
                NEXT_PAGE = i + 1;
                return Some(i * PAGE_SIZE);
            }
        }
        None
    }
}

pub fn free_page(addr: usize) {
    let page = addr / PAGE_SIZE;
    if page < MAX_PAGES {
        unsafe {
            let idx = page / 64;
            let bit = page % 64;
            BITMAP[idx] &= !(1 << bit);
        }
    }
}
