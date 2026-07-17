#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mos_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use mos_rust::{allocator, allocator::HEAP_SIZE,memory::{self, BootInfoFrameAllocator}};
use x86_64::VirtAddr;
use alloc::{vec::Vec, boxed::Box};
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {

    mos_rust::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    test_main();
    loop {}
}

#[test_case]
fn basic_alloc() {
    let heap1 = Box::new(41);
    let heap2 = Box::new(13);
    assert_eq!(*heap1, 41);
    assert_eq!(*heap2, 13);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    mos_rust::test_panic_handler(info)
}

#[test_case]
fn many_boxes() {
    let boxes = Box::new(1);
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*boxes, 1);
}