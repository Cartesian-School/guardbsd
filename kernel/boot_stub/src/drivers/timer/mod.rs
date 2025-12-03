// Timer Module
// BSD 3-Clause License

pub mod pit;

static mut TICKS: u64 = 0;

pub fn init(hz: u32) {
    pit::init(hz);
}

pub fn handle_interrupt() {
    unsafe {
        TICKS += 1;
    }
}

pub fn get_ticks() -> u64 {
    unsafe { TICKS }
}
