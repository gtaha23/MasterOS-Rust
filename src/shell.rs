/*
!                                     mShell source code of MasterOS operating system
!       This code is a primitive shell for the operating system, any flaw that's pointed out is appreciated.
!       Shell's current features are not so useful but in the nearest updates the documentation and the use cases of the OS will be
!       improved.
!
!                                                                                                           -gtaha23
*/


extern crate alloc;

use alloc::{vec::Vec, string::String};
use crate::{println, print, serial_println, vga_buffer::clear, exit_qemu, QemuExitCode, reboot_qemu, QemuRebootCode};
use crate::fs::{FileSystem, FsError};
use crate::task::{keyboard, time::sleep};
use crate::ata::AtaBlockDevice;

const OS_VER: &str = "0.1.0";
const SHELL_VER: &str = "0.0.8";

pub async fn run() {
    serial_println!("[checkpoint] shell::run() entered");
    let mut prompt = String::from("mOS");
    serial_println!("[checkpoint] before AtaBlockDevice::primary_master()");
    let block_device = AtaBlockDevice::primary_master().expect("ATA Disk Not Found");
    serial_println!("[checkpoint] after AtaBlockDevice::primary_master()");
    let mut sysfs = match FileSystem::mount(block_device) {
        Ok(fs) => {
            serial_println!("[checkpoint] FileSystem::mount() succeeded");
            fs
        }
        Err(FsError::NotFormatted) => {
            println!("Filesystem not formatted, creating a new filesystem...");
            serial_println!("[checkpoint] not formatted, calling AtaBlockDevice::primary_master() again");
            let block_device = AtaBlockDevice::primary_master().expect("ATA Disk Not Found");
            serial_println!("[checkpoint] before FileSystem::format()");
            let fs = FileSystem::format(block_device).expect("Failed to format filesystem");
            serial_println!("[checkpoint] FileSystem::format() succeeded");
            fs
        }
        Err(e) => panic!("FS Mount Error: {:?}", e),
    };
    serial_println!("[checkpoint] entering shell loop");
    loop {
        print!("{}> ", prompt); 

        let input = keyboard::read_line().await;
        let trimmed = input.trim();

        if trimmed.is_empty() {
            continue;
        }

        let mut parts = trimmed.split_whitespace();
        let command = parts.next().unwrap();
        let args: Vec<&str> = parts.collect();

        match command {
            "help" => {
                println!("Commands:\n -help,\n -ver,\n -clear,\n -shutdown,\n -reboot,\n -shellver,\n -sleep,\n -mfetch,\n -prompt,\n -ls,\n -mkdir,\n -cd,\n -pwd,\n -create,\n -write,\n -cat,\n -rm\n");
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
            "ls" => {
                let path = args.get(0).copied().unwrap_or("");
                match sysfs.list_files(path) {
                    Ok(files) => {
                        if files.is_empty() {
                            println!("Directory Empty!");
                        } else {
                            for file in files {
                                println!("{}", file);
                            }
                        }
                    }
                    Err(e) => println!("FS error: {:?}", e),
                }
            }
            "mkdir" => {
                if let Some(path) = args.first() {
                    match sysfs.create_dir(path) {
                        Ok(()) => println!("Created directory '{}'.", path),
                        Err(e) => println!("FS error: {:?}", e),
                    }
                } else {
                    println!("Usage: mkdir <directory>");
                }
            }
            "cd" => {
                if let Some(path) = args.first() {
                    match sysfs.change_dir(path) {
                        Ok(()) => println!("Changed directory to {}", sysfs.cwd()),
                        Err(e) => println!("FS error: {:?}", e),
                    }
                } else {
                    println!("Usage: cd <path>");
                }
            }
            "pwd" => {
                println!("{}", sysfs.cwd());
            }
            "create" => {
                if let Some(name) = args.first() {
                    match sysfs.create_file(name) {
                        Ok(()) => println!("Created file '{}'.", name),
                        Err(e) => println!("FS error: {:?}", e),
                    }
                } else {
                    println!("Usage: create <filename>");
                }
            }
            "write" => {
                if let Some(name) = args.first() {
                    let data = args[1..].join(" ");
                    if data.is_empty() {
                        println!("Usage: write <filename> <data>");
                    } else {
                        match sysfs.write_file(name, data.as_bytes()) {
                            Ok(()) => println!("Wrote {} bytes to '{}'.", data.len(), name),
                            Err(e) => println!("FS error: {:?}", e),
                        }
                    }
                } else {
                    println!("Usage: write <filename> <data>");
                }
            }
            "cat" => {
                if let Some(name) = args.first() {
                    match sysfs.read_file(name) {
                        Ok(bytes) => {
                            if let Ok(text) = core::str::from_utf8(&bytes) {
                                println!("{}", text);
                            } else {
                                println!("File contains non-UTF8 data");
                            }
                        }
                        Err(e) => println!("FS error: {:?}", e),
                    }
                } else {
                    println!("Usage: cat <filename>");
                }
            }
            "rm" => {
                if let Some(name) = args.first() {
                    match sysfs.delete_file(name) {
                        Ok(()) => println!("Deleted file '{}'.", name),
                        Err(e) => println!("FS error: {:?}", e),
                    }
                } else {
                    println!("Usage: rm <filename>");
                }
            }
            "yustupid" => {
                println!("Na im not");
                sleep(5 * 1000).await;
                println!("Whats nine plus ten?");
                sleep(5 * 1000).await;
                println!("Twanni one");
            }

            "sleep" => {
                if let Some(arg) = args.first() {
                    if let Ok(seconds) = arg.parse::<u64>() {
                        println!("Sleeping for {} seconds...", seconds);
                
                        sleep(seconds * 1000).await;
                
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
            "prompt" => {
                print!("Type the prompt you want to be seen -> ");
                let input = keyboard::read_line().await;
                if let Some(selected) = input.split_whitespace().next() {
                    prompt = String::from(selected);
                }
            }
            
            _ => {
                println!("!Command not found! : '{}'", command);
            }
        }
    }
}