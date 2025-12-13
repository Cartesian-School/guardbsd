//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: guardfs
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Kompresja GuardFS (LZ4-inspirowana, uproszczona).

pub const MAX_COMPRESSED_SIZE: usize = 4096 + 128; // Block + overhead

/// Simple LZ4-inspired compression
/// This is a simplified implementation for educational purposes
/// Production systems should use a full LZ4 library
pub fn compress(input: &[u8], output: &mut [u8]) -> Option<usize> {
    if input.is_empty() || output.len() < input.len() {
        return None;
    }

    let mut in_pos = 0;
    let mut out_pos = 0;

    while in_pos < input.len() {
        // Find match in previous data (simple backward search)
        let match_info = find_match(input, in_pos);

        if let Some((match_offset, match_len)) = match_info {
            if match_len >= 4 {
                // Encode literal count (0 = no literals)
                if out_pos + 3 >= output.len() {
                    return None;
                }

                output[out_pos] = 0; // No literals
                out_pos += 1;

                // Encode match (offset:u16, length:u8)
                output[out_pos..out_pos + 2].copy_from_slice(&(match_offset as u16).to_le_bytes());
                out_pos += 2;
                output[out_pos] = match_len as u8;
                out_pos += 1;

                in_pos += match_len;
                continue;
            }
        }

        // No match - emit literal
        if out_pos + 2 >= output.len() {
            return None;
        }

        output[out_pos] = 1; // 1 literal
        out_pos += 1;
        output[out_pos] = input[in_pos];
        out_pos += 1;
        in_pos += 1;
    }

    // Check if compression was effective (< 75% of original)
    if out_pos < (input.len() * 3) / 4 {
        Some(out_pos)
    } else {
        None // Not worth compressing
    }
}

fn find_match(input: &[u8], pos: usize) -> Option<(usize, usize)> {
    if pos < 4 {
        return None;
    }

    let max_match_len = input.len() - pos;
    let search_start = if pos > 4096 { pos - 4096 } else { 0 };

    let mut best_offset = 0;
    let mut best_len = 0;

    for search_pos in search_start..pos {
        let mut match_len = 0;

        while match_len < max_match_len
            && match_len < 255
            && pos + match_len < input.len()
            && input[search_pos + match_len] == input[pos + match_len]
        {
            match_len += 1;
        }

        if match_len > best_len {
            best_len = match_len;
            best_offset = pos - search_pos;
        }
    }

    if best_len >= 4 {
        Some((best_offset, best_len))
    } else {
        None
    }
}

pub fn decompress(input: &[u8], output: &mut [u8], expected_size: usize) -> Option<usize> {
    if expected_size > output.len() {
        return None;
    }

    let mut in_pos = 0;
    let mut out_pos = 0;

    while in_pos < input.len() && out_pos < expected_size {
        if in_pos + 1 > input.len() {
            break;
        }

        let literal_count = input[in_pos];
        in_pos += 1;

        if literal_count > 0 {
            // Copy literals
            if in_pos + literal_count as usize > input.len() {
                return None;
            }
            if out_pos + literal_count as usize > output.len() {
                return None;
            }

            output[out_pos..out_pos + literal_count as usize]
                .copy_from_slice(&input[in_pos..in_pos + literal_count as usize]);
            in_pos += literal_count as usize;
            out_pos += literal_count as usize;
        } else {
            // Match: read offset and length
            if in_pos + 3 > input.len() {
                break;
            }

            let offset = u16::from_le_bytes([input[in_pos], input[in_pos + 1]]) as usize;
            let length = input[in_pos + 2] as usize;
            in_pos += 3;

            if offset > out_pos || out_pos + length > output.len() {
                return None;
            }

            // Copy match
            let match_start = out_pos - offset;
            for i in 0..length {
                output[out_pos + i] = output[match_start + i];
            }
            out_pos += length;
        }
    }

    if out_pos == expected_size {
        Some(out_pos)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let input = b"Hello World! Hello World! Hello World!";
        let mut compressed = [0u8; 1024];
        let mut decompressed = [0u8; 1024];

        if let Some(comp_size) = compress(input, &mut compressed) {
            let decomp_size = decompress(&compressed[..comp_size], &mut decompressed, input.len());
            assert_eq!(decomp_size, Some(input.len()));
            assert_eq!(&decompressed[..input.len()], input);
        }
    }
}
