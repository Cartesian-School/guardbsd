// Keyboard Module
// BSD 3-Clause License

pub mod ps2;

pub fn handle_interrupt() {
    ps2::handle_interrupt();
}

pub fn read_char() -> Option<u8> {
    ps2::read_char()
}
