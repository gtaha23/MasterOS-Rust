<div align="center">
  
<img src="mOS-rust.png" alt="MasterOS Logo" width="250">

# 🦀 MasterOS-Rust

> A lightweight experimental operating system written in Rust.

*MasterOS-Rust is the Rust implementation of the MasterOS project. It focuses on learning modern operating system development while exploring memory management, multitasking, interrupts, filesystems, and shell development.*

[![Rust](https://img.shields.io/badge/Rust-orange.svg)](#)
[![Architecture](https://img.shields.io/badge/x86-32%2F64--bit-success.svg)](#)
[![License](https://img.shields.io/badge/License-GPL-green.svg)](#)

</div>

---

## ✨ Features

- ⚡ Rust-based kernel
- 🥾 **rustosdev** bootloader crate
- 🧠 Memory management
- 🔀 Async/Await support
- 🧵 Multitasking
- ⏱️ Timer support
- ⚠️ Interrupt handling
- 📂 Basic filesystem
- 💻 Interactive shell
- 🖥️ QEMU support

---

# 📚 Project History

MasterOS currently exists in two implementations:

- **MasterOS-6** — C implementation
- **MasterOS-Rust** — Rust implementation

The history and lore behind the project can be found in the original MasterOS repository.

---

# 📦 Dependencies

Dependencies are managed using Cargo.

They are listed inside **Cargo.toml** and can be installed with:

```bash
cargo add <crate-name>@<version>
```

---

# 🚀 Getting Started

Clone the repository:

```bash
git clone https://github.com/gtaha23/MasterOS-Rust.git
cd MasterOS-Rust
```

Install required tools:

```bash
rustup component add rust-src
cargo install bootimage
```

Build the bootable image:

```bash
cargo bootimage
```

Build the kernel:

```bash
cargo build --target x86_64-mos.json
cargo build
```

Run inside QEMU:

```bash
cargo run
```

---

# 🧪 Testing

Run all tests:

```bash
cargo test
```

Run a specific test:

```bash
cargo test --test <test_name>
```

---

# 💾 Writing to USB

Write the generated boot image to a USB drive:

```bash
dd if=target/x86_64-mos/debug/bootimage-mos_rust.bin of=/dev/<usb-device> bs=4M status=progress && sync
```

> **Warning**
>
> Replace `<usb-device>` with the correct device path (e.g. `/dev/sdb`).
> Choosing the wrong device may erase important data.

---

# 📈 Development Timeline

| Date | Milestone |
|------|-----------|
| 20.02.2025 | Initial Commit |
| 01.03.2025 | Testing framework |
| 05.07.2026 | Interrupt handling |
| 06.07.2026 | GDT, IDT, Double Fault support |
| 17.07.2026 | Memory improvements, Async/Await, Multitasking |
| 19.07.2026 | Filesystem and Timer |
| 20.07.2026 | Shell development started |
| 21.07.2026 | Initial shell commands |
| 22.07.2026 – Present | Shell command improvements |
| 31.07.2026 | Filesystem integration with shell |

---

# 🛣️ Roadmap

- [x] Interrupt handling
- [x] GDT / IDT
- [x] Memory management
- [x] Async scheduler
- [x] Multitasking
- [x] Filesystem
- [x] Shell
- [ ] Virtual memory
- [ ] ELF executable loader
- [ ] Networking
- [ ] Userspace applications
- [ ] Graphics subsystem

---

# 🤝 Contributing

Contributions are welcome!

If you'd like to improve MasterOS-Rust, feel free to:

- Fork the repository
- Create a feature branch
- Submit a Pull Request
- Open an Issue for bugs or feature requests

---

# 👨‍💻 Developer

Currently maintained by **gtaha23**.

Previous MasterOS projects were developed together with **e0tra**, while MasterOS-Rust is currently maintained by a single active developer.

---

# 📄 License

This project has GPL-3.0 license.
