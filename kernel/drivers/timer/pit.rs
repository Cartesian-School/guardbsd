// PIT (Programmable Interval Timer) Driver
// BSD 3-Clause License

const PIT_CHANNEL0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;
const PIT_FREQUENCY: u32 = 1193182;

pub fn init(hz: u32) {
    let divisor = (PIT_FREQUENCY / hz) as u16;
    
    unsafe {
        // Command: Channel 0, lobyte/hibyte, rate generator
        outb(PIT_COMMAND, 0x36);
        
        // Set frequency divisor
        outb(PIT_CHANNEL0, (divisor & 0xFF) as u8);
        outb(PIT_CHANNEL0, ((divisor >> 8) & 0xFF) as u8);
    }
}

unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val);
}
