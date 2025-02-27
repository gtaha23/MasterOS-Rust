# MasterOS-Rust
A lightweight OS written in Rust (bootloader is included as a crate)

# History
This project is currently being developed in 2 ways
-masterOS-ubuntu (C repo), 
-MasterOS-Rust (Rust repo)

# Dependencies
The dependencies are showed in Cargo.toml, you can add them with the command "cargo add crate-name@version"

# Updates
- VGA Text Mode is added
- Macros for printing added

## Developers
This project is being developed by me, but any contributer is welcome!
Past projects were being developed by me and e0tra. But currently i am the only developer active here.


# Running
To run mOS in qemu, you need to use these commands ( create an issue if some commands are missing ) last command is for writing the image to a USB

```
cargo new mOS
cd mOS
rustup target add thumbv7em-none-eabihf
cargo install bootimage
cargo bootimage
cargo build
cargo run

# Use for USB
dd if=target/x86_64-mos/debug/bootimage-rust_app_name.bin of=/dev/usb-name && sync
```
## Logs
- Initial Commit : <20.02.2025>
- Coming soon
