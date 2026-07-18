#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mos_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{string::String, vec};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use mos_rust::{allocator, fs::{FileSystem, MemBlockDevice}, memory::{self, BootInfoFrameAllocator}};
use x86_64::VirtAddr;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    mos_rust::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    test_main();
    loop {}
}

#[test_case]
fn create_write_read_roundtrip() {
    let dev = MemBlockDevice::new(256);
    let mut fs = FileSystem::format(dev).expect("format failed");

    fs.create_file("hello.txt").expect("create failed");
    fs.write_file("hello.txt", b"Hello, MasterOS!").expect("write failed");

    let data = fs.read_file("hello.txt").expect("read failed");
    assert_eq!(&data, b"Hello, MasterOS!");
}

#[test_case]
fn list_files_reports_created_files() {
    let dev = MemBlockDevice::new(256);
    let mut fs = FileSystem::format(dev).expect("format failed");

    fs.create_file("a.txt").unwrap();
    fs.create_file("b.txt").unwrap();

    let mut names = fs.list_files().unwrap();
    names.sort();
    assert_eq!(names, vec![String::from("a.txt"), String::from("b.txt")]);
}

#[test_case]
fn overwrite_replaces_contents_and_size() {
    let dev = MemBlockDevice::new(256);
    let mut fs = FileSystem::format(dev).expect("format failed");

    fs.create_file("note.txt").unwrap();
    fs.write_file("note.txt", b"first version, quite a bit longer than the second").unwrap();
    fs.write_file("note.txt", b"short").unwrap();

    assert_eq!(fs.read_file("note.txt").unwrap(), b"short");
}

#[test_case]
fn delete_frees_space_for_reuse() {
    let dev = MemBlockDevice::new(64);
    let mut fs = FileSystem::format(dev).expect("format failed");

    fs.create_file("big.bin").unwrap();
    let payload = vec![0xABu8; 4096];
    fs.write_file("big.bin", &payload).expect("write failed");
    fs.delete_file("big.bin").expect("delete failed");

    fs.create_file("big2.bin").unwrap();
    fs.write_file("big2.bin", &payload)
        .expect("second write should reuse freed blocks");
}

#[test_case]
fn duplicate_create_fails() {
    let dev = MemBlockDevice::new(64);
    let mut fs = FileSystem::format(dev).expect("format failed");
    fs.create_file("dup.txt").unwrap();
    assert!(fs.create_file("dup.txt").is_err());
}

#[test_case]
fn read_missing_file_fails() {
    let dev = MemBlockDevice::new(64);
    let mut fs = FileSystem::format(dev).expect("format failed");
    assert!(fs.read_file("nope.txt").is_err());
}

#[test_case]
fn mount_after_format_round_trips_data() {
    let dev = MemBlockDevice::new(256);
    let dev = {
        let mut fs = FileSystem::format(dev).expect("format failed");
        fs.create_file("persisted.txt").unwrap();
        fs.write_file("persisted.txt", b"still here").unwrap();
        fs.into_device()
    };

    let mut remounted = FileSystem::mount(dev).expect("mount failed");
    assert_eq!(remounted.read_file("persisted.txt").unwrap(), b"still here");
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    mos_rust::test_panic_handler(info)
}