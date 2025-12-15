//! kernel/arch/riscv64/dtb.rs
//!
//! Minimal Flattened Device Tree (FDT/DTB) parser for early boot.
//! - no_std
//! - no alloc
//! - no fmt
//! - bounds-checked reads
//!
//! References: DTB header, structure block tokens (BEGIN_NODE, PROP, END_NODE, END).
//! We parse a small subset sufficient for QEMU virt bring-up:
//! - root compatible
//! - memory nodes: "device_type" == "memory" OR node name starts with "memory"
//! - reg properties for memory and a few device nodes
//! - #address-cells / #size-cells propagation (simple stack)

use core::cmp::min;

const FDT_MAGIC: u32 = 0xD00D_FEED;

const FDT_BEGIN_NODE: u32 = 0x1;
const FDT_END_NODE: u32 = 0x2;
const FDT_PROP: u32 = 0x3;
const FDT_NOP: u32 = 0x4;
const FDT_END: u32 = 0x9;

#[derive(Copy, Clone)]
struct Header {
    totalsize: u32,
    off_dt_struct: u32,
    off_dt_strings: u32,
    size_dt_struct: u32,
    size_dt_strings: u32,
}

fn read_be_u32(buf: &[u8], off: usize) -> Option<u32> {
    let b = buf.get(off..off + 4)?;
    Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

fn align4(x: usize) -> usize {
    (x + 3) & !3
}

fn cstr_len(bytes: &[u8]) -> usize {
    // Return length up to NUL or end of slice
    for (i, &b) in bytes.iter().enumerate() {
        if b == 0 {
            return i;
        }
    }
    bytes.len()
}

/// Console interface for early boot printing (UART-backed).
pub trait Console {
    fn putc(&self, ch: u8);
    fn puts(&self, s: &str);

    fn put_hex_u64(&self, v: u64){
        self.puts("0x");
        // Print full width (16 hex digits) for determinism
        for i in (0..16).rev() {
            let n = ((v >> (i * 4)) & 0xF) as u8;
            let ch = if n <= 9 { b'0' + n } else { b'a' + (n - 10) };
            self.putc(ch);
        }
    }

    fn put_dec_usize(&self, mut v: usize) {
        if v == 0 {
            self.putc(b'0');
            return;
        }
        let mut buf = [0u8; 32];
        let mut i = 0usize;
        while v > 0 && i < buf.len() {
            buf[i] = b'0' + (v % 10) as u8;
            v /= 10;
            i += 1;
        }
        while i > 0 {
            i -= 1;
            self.putc(buf[i]);
        }
    }
}

/// Parse DTB from a physical pointer (as given by OpenSBI in a1) and print useful info.
pub fn parse_and_print(dtb_ptr: usize, con: &impl Console) {
    if dtb_ptr == 0 {
        con.puts("[DTB] dtb_ptr is NULL\r\n");
        return;
    }

    // Safety: DTB is provided by firmware; we must bounds-check everything.
    // We'll read header fields first, then create bounded slices using totalsize.
    let base = dtb_ptr as *const u8;

    let hdr_bytes = unsafe { core::slice::from_raw_parts(base, 40) }; // header is 40 bytes
    let magic = match read_be_u32(hdr_bytes, 0) {
        Some(v) => v,
        None => {
            con.puts("[DTB] cannot read header\r\n");
            return;
        }
    };

    if magic != FDT_MAGIC {
        con.puts("[DTB] bad magic: ");
        con.put_hex_u64(magic as u64);
        con.puts("\r\n");
        return;
    }

    let totalsize = read_be_u32(hdr_bytes, 4).unwrap_or(0);
    let off_dt_struct = read_be_u32(hdr_bytes, 8).unwrap_or(0);
    let off_dt_strings = read_be_u32(hdr_bytes, 12).unwrap_or(0);
    let size_dt_struct = read_be_u32(hdr_bytes, 36).unwrap_or(0);
    let size_dt_strings = read_be_u32(hdr_bytes, 32).unwrap_or(0);

    if totalsize < 40 {
        con.puts("[DTB] totalsize too small\r\n");
        return;
    }

    let dtb = unsafe { core::slice::from_raw_parts(base, totalsize as usize) };

    let h = Header {
        totalsize,
        off_dt_struct,
        off_dt_strings,
        size_dt_struct,
        size_dt_strings,
    };

    // Basic bounds sanity
    let struct_off = h.off_dt_struct as usize;
    let strings_off = h.off_dt_strings as usize;
    if struct_off >= dtb.len() || strings_off >= dtb.len() {
        con.puts("[DTB] offsets out of range\r\n");
        return;
    }

    // Derive block slices (bounded by sizes if provided, else by totalsize)
    let struct_len = if h.size_dt_struct != 0 {
        min(h.size_dt_struct as usize, dtb.len().saturating_sub(struct_off))
    } else {
        dtb.len().saturating_sub(struct_off)
    };
    let strings_len = if h.size_dt_strings != 0 {
        min(h.size_dt_strings as usize, dtb.len().saturating_sub(strings_off))
    } else {
        dtb.len().saturating_sub(strings_off)
    };

    let dt_struct = &dtb[struct_off..struct_off + struct_len];
    let dt_strings = &dtb[strings_off..strings_off + strings_len];

    con.puts("[DTB] ok, totalsize=");
    con.put_dec_usize(h.totalsize as usize);
    con.puts(" bytes\r\n");

    walk_structure(dt_struct, dt_strings, con);
}

fn walk_structure(dt_struct: &[u8], dt_strings: &[u8], con: &impl Console) {
    // Stack of (#address-cells, #size-cells) for each node scope.
    // DT spec defaults: if missing, commonly address=2 size=1 or 2;
    // For QEMU virt root, it's usually 2/2. We'll default to 2/2.
    let mut stack_addr_cells: [u32; 64] = [0; 64];
    let mut stack_size_cells: [u32; 64] = [0; 64];
    let mut depth: usize = 0;

    // Defaults at root
    stack_addr_cells[0] = 2;
    stack_size_cells[0] = 2;

    let mut off: usize = 0;

    // Some state about current node
    let mut cur_name_is_memory = false;
    let mut cur_device_type_memory = false;
    let mut cur_node_name_buf: [u8; 64] = [0; 64];
    let mut cur_node_name_len: usize = 0;

    con.puts("[DTB] walking structure...\r\n");

    while off + 4 <= dt_struct.len() {
        let token = match read_be_u32(dt_struct, off) {
            Some(v) => v,
            None => break,
        };
        off += 4;

        match token {
            FDT_BEGIN_NODE => {
                // node name: NUL-terminated string, padded to 4 bytes
                let name_bytes = &dt_struct[off..];
                let nlen = cstr_len(name_bytes);
                let name = &name_bytes[..min(nlen, name_bytes.len())];

                // Save truncated node name in buffer for diagnostics
                cur_node_name_len = min(name.len(), cur_node_name_buf.len());
                cur_node_name_buf[..cur_node_name_len].copy_from_slice(&name[..cur_node_name_len]);

                // Identify memory-ish node by name prefix
                cur_name_is_memory = starts_with(name, b"memory");

                // reset device_type flag for this node
                cur_device_type_memory = false;

                // Enter new depth
                if depth + 1 < stack_addr_cells.len() {
                    depth += 1;
                    // inherit parent values by default
                    stack_addr_cells[depth] = stack_addr_cells[depth - 1];
                    stack_size_cells[depth] = stack_size_cells[depth - 1];
                } else {
                    con.puts("[DTB] depth overflow, stopping\r\n");
                    return;
                }

                off += align4(nlen + 1); // include NUL
            }
            FDT_END_NODE => {
                // Leave node
                if depth > 0 {
                    depth -= 1;
                }
                // Reset node flags (safe, next BEGIN_NODE will set again)
                cur_name_is_memory = false;
                cur_device_type_memory = false;
                cur_node_name_len = 0;
            }
            FDT_PROP => {
                // struct fdt_property:
                // u32 len; u32 nameoff; u8 value[len]; padding
                let len = match read_be_u32(dt_struct, off) {
                    Some(v) => v as usize,
                    None => return,
                };
                let nameoff = match read_be_u32(dt_struct, off + 4) {
                    Some(v) => v as usize,
                    None => return,
                };
                off += 8;

                if off + len > dt_struct.len() {
                    con.puts("[DTB] property value out of range\r\n");
                    return;
                }

                let prop_name = get_string(dt_strings, nameoff);
                let value = &dt_struct[off..off + len];

                // Track address/size cells
                if eq_str(prop_name, b"#address-cells") && len >= 4 {
                    if let Some(v) = read_be_u32(value, 0) {
                        stack_addr_cells[depth] = v;
                    }
                } else if eq_str(prop_name, b"#size-cells") && len >= 4 {
                    if let Some(v) = read_be_u32(value, 0) {
                        stack_size_cells[depth] = v;
                    }
                }

                // Track device_type == "memory"
                if eq_str(prop_name, b"device_type") {
                    if contains_cstr(value, b"memory") {
                        cur_device_type_memory = true;
                    }
                }

                // Print root compatible (depth==1 corresponds to root "/" node in most DTBs)
                if eq_str(prop_name, b"compatible") && depth == 1 {
                    con.puts("[DTB] root compatible: ");
                    print_compat_list(value, con);
                    con.puts("\r\n");
                }

                // Print "reg" for interesting nodes:
                if eq_str(prop_name, b"reg") {
                    let addr_cells = stack_addr_cells[depth] as usize;
                    let size_cells = stack_size_cells[depth] as usize;

                    // Memory node?
                    let is_memory_node = cur_name_is_memory || cur_device_type_memory;
                    if is_memory_node || is_interesting_node(&cur_node_name_buf[..cur_node_name_len]) {
                        con.puts("[DTB] reg @ node '");
                        print_bytes_as_ascii(&cur_node_name_buf[..cur_node_name_len], con);
                        con.puts("': ");

                        print_reg_list(value, addr_cells, size_cells, con);
                        con.puts("\r\n");
                    }
                }

                off += align4(len);
            }
            FDT_NOP => {
                // Ignore
            }
            FDT_END => {
                con.puts("[DTB] end\r\n");
                return;
            }
            _ => {
                con.puts("[DTB] unknown token: ");
                con.put_hex_u64(token as u64);
                con.puts("\r\n");
                return;
            }
        }
    }

    con.puts("[DTB] structure walk finished (no END token)\r\n");
}

fn get_string(dt_strings: &[u8], off: usize) -> &[u8] {
    if off >= dt_strings.len() {
        return b"";
    }
    let s = &dt_strings[off..];
    let nlen = cstr_len(s);
    &s[..min(nlen, s.len())]
}

fn eq_str(a: &[u8], b: &[u8]) -> bool {
    a == b
}

fn starts_with(s: &[u8], prefix: &[u8]) -> bool {
    s.len() >= prefix.len() && &s[..prefix.len()] == prefix
}

fn contains_cstr(value: &[u8], needle: &[u8]) -> bool {
    // value is expected to be a NUL-terminated string or list; check first string only
    let nlen = cstr_len(value);
    &value[..min(nlen, value.len())] == needle
}

fn print_bytes_as_ascii(bytes: &[u8], con: &impl Console) {
    for &b in bytes {
        let ch = if b.is_ascii_graphic() || b == b'@' || b == b'/' || b == b'-' || b == b'_' {
            b
        } else {
            b'.'
        };
        con.putc(ch);
    }
}

fn print_compat_list(value: &[u8], con: &impl Console) {
    // compatible is a list of NUL-terminated strings
    let mut i = 0usize;
    let mut first = true;
    while i < value.len() {
        let rest = &value[i..];
        let n = cstr_len(rest);
        if n == 0 {
            break;
        }
        if !first {
            con.puts(", ");
        }
        print_bytes_as_ascii(&rest[..min(n, rest.len())], con);
        first = false;
        i += n + 1;
    }
}

fn is_interesting_node(name: &[u8]) -> bool {
    // minimal heuristic for QEMU virt:
    // uart*, plic*, clint*, aclint*, virtio_mmio*, chosen
    starts_with(name, b"uart")
        || starts_with(name, b"plic")
        || starts_with(name, b"clint")
        || starts_with(name, b"aclint")
        || starts_with(name, b"virtio")
        || starts_with(name, b"chosen")
}

fn print_reg_list(value: &[u8], addr_cells: usize, size_cells: usize, con: &impl Console) {
    // reg can contain multiple (addr,size) tuples.
    // Each cell is 32-bit big-endian.
    let cells_per_tuple = addr_cells + size_cells;
    if cells_per_tuple == 0 {
        con.puts("<invalid cells>");
        return;
    }
    if value.len() % 4 != 0 {
        con.puts("<unaligned reg>");
        return;
    }

    let total_cells = value.len() / 4;
    let tuples = total_cells / cells_per_tuple;

    for t in 0..tuples {
        if t != 0 {
            con.puts("; ");
        }
        let base_cell = t * cells_per_tuple;

        let addr = read_cells_u64(value, base_cell, addr_cells);
        let size = read_cells_u64(value, base_cell + addr_cells, size_cells);

        con.put_hex_u64(addr);
        con.puts(" + ");
        con.put_hex_u64(size);
    }

    if tuples == 0 {
        con.puts("<empty>");
    }
}

fn read_cells_u64(value: &[u8], cell_index: usize, cells: usize) -> u64 {
    // Combine up to 2 cells into u64 (common for 64-bit addresses/sizes).
    // If more than 2 cells, we will fold but keep lower 64 bits deterministically.
    let mut out: u64 = 0;
    for i in 0..cells {
        let off = (cell_index + i) * 4;
        if off + 4 > value.len() {
            break;
        }
        let part = u32::from_be_bytes([value[off], value[off + 1], value[off + 2], value[off + 3]]) as u64;
        out = (out << 32) | part;
    }
    out
}
