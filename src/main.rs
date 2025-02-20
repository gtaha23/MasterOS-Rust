#![no_std]
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;


#[panic_handler]
fn panic_hanler(_info: &PanicInfo) -> ! {
	println!("{}", _info);
	loop {}
}

static _HELLO: &[u8] = b"Hello mOS-Rust!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
	println!("Hello World! \n");
	println!("New line test");
	
	loop {}
}
