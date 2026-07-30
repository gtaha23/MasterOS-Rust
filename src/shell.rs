extern crate alloc;

use alloc::{vec::Vec, string::String, boxed::Box};
use crate::{println, print, vga_buffer::clear, exit_qemu, QemuExitCode, reboot_qemu, QemuRebootCode};
use crate::task::{keyboard, time::sleep};


const OS_VER: &str = "0.0.9";
const SHELL_VER: &str = "0.0.5";

pub async fn run() {
    let mut prompt: &str ="mOS";
    loop {
        print!("{}> ", prompt); 

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
                println!("Commands: help, ver, clear, shutdown, reboot, shellver, sleep, mfetch, prompt");
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
                sleep(5 * 1000).await;
                println!("Whats nine plus ten?");
                sleep(5 * 1000).await;
                println!("Twanni one");
            }

            "sleep" => {
                if let Some(arg) = _args.first() {
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
                    let leaked: &'static str = Box::leak(String::from(selected).into_boxed_str());
                    prompt = leaked;
                }
            }
            _ => {
                println!("!Command not found! : '{}'", command);
            }
        }
    }
}
