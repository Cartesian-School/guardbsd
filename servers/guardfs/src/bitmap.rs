// servers/guardfs/src/bitmap.rs
// GuardFS Block Allocation Bitmap
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

pub const BITMAP_SIZE: usize = 16384; // 16K bytes = 131,072 blocks

pub struct BlockBitmap {
    bitmap: [u64; BITMAP_SIZE / 8], // 2048 u64s
}

impl BlockBitmap {
    pub const fn new() -> Self {
        Self {
            bitmap: [0xFFFFFFFFFFFFFFFF; BITMAP_SIZE / 8], // All free
        }
    }

    pub fn init(&mut self, reserved_blocks: u64) {
        // Mark reserved blocks as used (metadata)
        for block in 0..reserved_blocks {
            self.mark_used(block);
        }
    }

    pub fn allocate(&mut self) -> Option<u64> {
        for (word_idx, word) in self.bitmap.iter_mut().enumerate() {
            if *word != 0 {
                let bit = word.trailing_zeros() as usize;
                if bit < 64 {
                    *word &= !(1u64 << bit);
                    let block_num = (word_idx * 64 + bit) as u64;
                    return Some(block_num);
                }
            }
        }
        None
    }

    pub fn allocate_contiguous(&mut self, count: u32) -> Option<u64> {
        if count == 0 {
            return None;
        }

        if count == 1 {
            return self.allocate();
        }

        // Find contiguous free blocks
        let mut start_block = None;
        let mut free_count = 0u32;

        for block in 0..(BITMAP_SIZE * 8) as u64 {
            if self.is_free(block) {
                if free_count == 0 {
                    start_block = Some(block);
                }
                free_count += 1;

                if free_count == count {
                    // Found enough contiguous blocks
                    if let Some(start) = start_block {
                        // Mark all as used
                        for b in start..(start + count as u64) {
                            self.mark_used(b);
                        }
                        return Some(start);
                    }
                }
            } else {
                // Reset search
                start_block = None;
                free_count = 0;
            }
        }

        None // Not enough contiguous blocks
    }

    pub fn free(&mut self, block_num: u64) {
        if block_num < (BITMAP_SIZE * 8) as u64 {
            let word_idx = (block_num / 64) as usize;
            let bit = (block_num % 64) as usize;
            self.bitmap[word_idx] |= 1u64 << bit;
        }
    }

    pub fn free_extent(&mut self, start: u64, count: u32) {
        for i in 0..count {
            self.free(start + i as u64);
        }
    }

    pub fn is_free(&self, block_num: u64) -> bool {
        if block_num >= (BITMAP_SIZE * 8) as u64 {
            return false;
        }

        let word_idx = (block_num / 64) as usize;
        let bit = (block_num % 64) as usize;
        (self.bitmap[word_idx] & (1u64 << bit)) != 0
    }

    pub fn mark_used(&mut self, block_num: u64) {
        if block_num < (BITMAP_SIZE * 8) as u64 {
            let word_idx = (block_num / 64) as usize;
            let bit = (block_num % 64) as usize;
            self.bitmap[word_idx] &= !(1u64 << bit);
        }
    }

    pub fn count_free(&self) -> u64 {
        let mut count = 0u64;
        for word in &self.bitmap {
            count += word.count_ones() as u64;
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation() {
        let mut bitmap = BlockBitmap::new();
        bitmap.init(298); // Reserve metadata

        let block = bitmap.allocate();
        assert!(block.is_some());
        assert!(block.unwrap() >= 298);
    }

    #[test]
    fn test_contiguous_allocation() {
        let mut bitmap = BlockBitmap::new();
        bitmap.init(100);

        let extent = bitmap.allocate_contiguous(10);
        assert!(extent.is_some());

        // Verify all blocks allocated
        let start = extent.unwrap();
        for i in 0..10 {
            assert!(!bitmap.is_free(start + i));
        }
    }
}
