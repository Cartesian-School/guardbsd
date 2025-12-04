#![cfg(feature = "user_mode_test")]
#![no_std]

#[no_mangle]
pub extern "C" fn dummy_user_entry() -> ! {
    loop {}
}
