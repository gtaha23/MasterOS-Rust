#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(mos_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::vec::Vec;
use crate::{println, print, vga_buffer::clear, exit_qemu, QemuExitCode, reboot_qemu, QemuRebootCode};
use crate::task::{keyboard, executor::Executor};
use bootloader::entry_point;

const OS_VER: &str = "0.0.9";
const SHELL_VER: &str = "0.0.3";

pub async fn run() {
    loop {
        print!("mOS> "); 

        let input = keyboard::read_line().await;
        let trimmed = input.trim();

        if trimmed.is_empty() {
            continue;
        }

        let mut parts = trimmed.split_whitespace();
        let command = parts.next().unwrap();
        let _args: Vec<&str> = parts.collect();

        match command {
            "help" => {
                println!("Commands: help, ver, clear, shutdown, reboot, shellver, sleep, mfetch");
            }
            "ver" => {
                println!("MasterOS -Rusty Pipe- {}", OS_VER);
            }
            "clear" => {
                clear();
            }
            "shutdown" => {
                exit_qemu(QemuExitCode::Success);
            }
            "reboot" => {
                reboot_qemu(0x02, QemuRebootCode::Success);
            }
            "shellver" => {
                println!("mShell {}", SHELL_VER);
            }
            "yustupid" => {
                println!("Na im not");
                crate::task::time::sleep(5 * 1000).await;
                println!("Whats nine plus ten?");
                crate::task::time::sleep(5 * 1000).await;
                println!("Twanni one");
            }

            "sleep" => {
                if let Some(arg) = _args.get(0) {
                    if let Ok(seconds) = arg.parse::<u64>() {
                        println!("Sleeping for {} seconds...", seconds);
                
                        crate::task::time::sleep(seconds * 1000).await;
                
                        println!("Woke up!");
                    } else {
                        println!("Usage: sleep <seconds> (must be a positive integer)");
                    }
                } else {
                    println!("Usage: sleep <seconds>");
                }
            }
            "mfetch" => {
            	println!(" ");
            	println!("                     ########  ######## ");
            	println!(" ######## #####    ###    ### ###    ## ");
            	println!("  ######### ####  ##      ### ######    ");
            	println!("  ###  ###  #### ###     ###    ######  ");
            	println!(" ###  ###  #### ##     ### ##    ####   ");
            	println!(" ################ ########  ########    ");
            	println!(" ");
            	println!("  OS:      MasterOS {}", OS_VER);
            	println!("  Kernel:  Custom x86 (32-bit)");
            	println!("  Shell:   mShell {}", SHELL_VER);
            }
            _ => {
                println!("!Command not found! : '{}'", command);
            }
        }
    }
}
