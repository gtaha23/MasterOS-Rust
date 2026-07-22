#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mos_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec::Vec, vec};
use mos_rust::{task::{Task, keyboard, executor::Executor}, memory::{self, BootInfoFrameAllocator}, println, allocator, shell};
use x86_64::{VirtAddr};
use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};

const OS_VER: &str = "0.0.9";

entry_point!(kmain);

fn kmain(bi: &'static BootInfo) -> ! {
    println!("MasterOS -Rusty Pipe- {}", OS_VER);
    mos_rust::init();

    let phys_mem_offset = VirtAddr::new(bi.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&bi.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();

    executor.spawn(Task::new(shell::run()));
    executor.run();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    mos_rust::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    mos_rust::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
