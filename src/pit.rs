//! Programmable Interval Timer (PIT) driver.
//!
//! Configures PIT channel 0 (wired to legacy IRQ0 / `InterruptIndex::Timer`)
//! to fire at a fixed frequency, and tracks elapsed ticks so the kernel has
//! a notion of time. This does not replace the existing timer interrupt
//! handler in `interrupts.rs` — it hooks into it via [`tick`].

use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::instructions::port::Port;

// The PIT's oscillator runs at ~1.193182 MHz.
const PIT_FREQUENCY: u32 = 1_193_182;

pub const TIMER_HZ: u32 = 100;

static TICKS: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    let divisor = PIT_FREQUENCY / TIMER_HZ;
    // The PIT's divisor register is 16 bits, so TIMER_HZ can't go below ~18.2 Hz.
    assert!(
        divisor <= u16::MAX as u32,
        "TIMER_HZ too low for a 16-bit PIT divisor"
    );

    let mut command: Port<u8> = Port::new(0x43);
    let mut channel0: Port<u8> = Port::new(0x40);

    unsafe {
        command.write(0b00_11_011_0u8);
        channel0.write((divisor & 0xff) as u8);
        channel0.write(((divisor >> 8) & 0xff) as u8);
    }
}

pub(crate) fn tick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

pub fn uptime_ms() -> u64 {
    ticks() * 1000 / TIMER_HZ as u64
}

pub fn sleep_ms(ms: u64) {
    let target = ticks() + (ms * TIMER_HZ as u64) / 1000;
    while ticks() < target {
        x86_64::instructions::hlt();
    }
}