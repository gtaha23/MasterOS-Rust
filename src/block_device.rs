//! Abstraction over anything that can serve fixed-size block reads/writes.
//!
//! This lets [`crate::fs`] work identically against a real disk
//! ([`crate::ata::AtaBlockDevice`]) or an in-memory backing store
//! ([`crate::fs::MemBlockDevice`], used for testing), without caring which.

pub const BLOCK_SIZE: usize = 512;

pub trait BlockDevice {
    type Error: core::fmt::Debug;

    fn total_blocks(&self) -> u32;

    fn read_block(&mut self, block: u32, buf: &mut [u8; BLOCK_SIZE]) -> Result<(), Self::Error>;

    fn write_block(&mut self, block: u32, buf: &[u8; BLOCK_SIZE]) -> Result<(), Self::Error>;
}