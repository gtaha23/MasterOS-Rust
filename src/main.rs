#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mos_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use mos_rust::{task::{Task, executor::Executor}, memory::{self, BootInfoFrameAllocator}, println, serial_println, allocator, shell};
use x86_64::{VirtAddr};
use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};

const OS_VER: &str = "0.1.0";

entry_point!(kmain);

fn kmain(bi: &'static BootInfo) -> ! {
    println!("MasterOS -Rusty Pipe- {}", OS_VER);
    serial_println!("[checkpoint] before mos_rust::init()");
    mos_rust::init();
    serial_println!("[checkpoint] after mos_rust::init()");

    let phys_mem_offset = VirtAddr::new(bi.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    serial_println!("[checkpoint] after memory::init()");
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&bi.memory_map) };
    serial_println!("[checkpoint] after BootInfoFrameAllocator::init()");

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
    serial_println!("[checkpoint] after allocator::init_heap()");

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    serial_println!("[checkpoint] after Executor::new()");

    let task = Task::new(shell::run());
    serial_println!("[checkpoint] after Task::new(shell::run()) constructed");
    executor.spawn(task);
    serial_println!("[checkpoint] after executor.spawn()");
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