# MasterOS-Rust
A lightweight OS written in Rust (bootloader is included as a crate)

# History
The lore behind this project is on MasterOS repo (C#)
This project is currently being developed in 2 ways
- masterOS-ubuntu (C repo), 
- MasterOS-Rust (Rust repo)

# Dependencies
The dependencies are showed in Cargo.toml, you can add them with the command "cargo add crate-name@version"

# Updates
- Testing added
- Integrated Tests added
- Fixes for vga_buffer.rs and main.rs added

## Developers
This project is being developed by me, but any contributer is welcome!
Past projects were being developed by me and e0tra. But currently i am the only developer active here.


# Running
To run mOS in qemu, you need to use these commands ( create an issue if some commands are missing ) last command is for writing the image to a USB

```
cargo new mos_rust
cd mos_rust
rustup target add thumbv7em-none-eabihf
cargo install bootimage
cargo bootimage
cargo build
cargo run

# testing
cargo test

# Use for USB
dd if=target/x86_64-mos/debug/bootimage-rust_app_name.bin of=/dev/usb-name && sync
```

## Logs
- Initial Commit : <20.02.2025>
- Testing Added & fixes : <01.03.2025>
- Coming soon
