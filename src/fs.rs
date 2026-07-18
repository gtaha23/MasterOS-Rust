//! A small custom filesystem ("SimpleFS") built on top of any [`BlockDevice`].
//!
//! ## On-disk layout
//!
//! ```text
//! block 0             : superblock
//! blocks 1..I         : inode table (fixed-size 64-byte inodes, MAX_FILES total)
//! blocks I..I+B        : free-block bitmap (1 bit per data block)
//! blocks I+B..end       : data blocks
//! ```
//!
//! ## Design choices (and their tradeoffs)
//!
//! - **Flat namespace, no directories.** Every inode carries its own
//!   filename, so the inode table doubles as the root directory listing.
//!   Simple to implement and reason about; there's just nowhere to put a
//!   subfolder.
//! - **Fixed-size inode table (`MAX_FILES` entries).** No dynamic growth,
//!   so the inode table's block cost is paid up front regardless of disk
//!   size. With the current constants that's 16 blocks (8KiB) — meaning a
//!   device needs to be at least ~18-20 blocks just to have room for any
//!   actual file data.
//! - **6 direct block pointers + 1 singly-indirect block per file.**
//!   Max file size is `6*512 + 128*512` bytes (~67KiB). Good enough for
//!   text files, configs, small binaries — not for anything large.
//! - **No journaling, no crash consistency guarantees.** A power loss
//!   mid-write can leave a file's inode and its data blocks inconsistent.
//!   Fine for a hobby OS; a real answer would need write-ahead logging.
//! - **Linear scans for free inodes/blocks.** `create_file` and
//!   `alloc_block` scan the inode table / bitmap byte-by-byte rather than
//!   maintaining a free list. Simple and correct; O(n) rather than O(1).

use crate::block_device::{BlockDevice, BLOCK_SIZE};
use alloc::{string::String, vec, vec::Vec};

pub const MAGIC: u32 = 0x53465331; // "SFS1"
pub const MAX_FILES: usize = 128;
pub const MAX_NAME_LEN: usize = 28;
pub const DIRECT_POINTERS: usize = 6;
const INODE_SIZE: usize = 64;
const POINTERS_PER_BLOCK: usize = BLOCK_SIZE / 4;

#[derive(Debug)]
pub enum FsError<E> {
    Io(E),
    NotFound,
    AlreadyExists,
    NoSpace,
    NoFreeInode,
    NotFormatted,
    NameTooLong,
    FileTooLarge,
}

fn ceil_div(a: u32, b: u32) -> u32 {
    (a + b - 1) / b
}

#[derive(Clone, Copy)]
struct Superblock {
    magic: u32,
    total_blocks: u32,
    inode_table_start: u32,
    inode_table_blocks: u32,
    bitmap_start: u32,
    bitmap_blocks: u32,
    data_start: u32,
    max_files: u32,
}

impl Superblock {
    fn to_bytes(&self) -> [u8; BLOCK_SIZE] {
        let mut buf = [0u8; BLOCK_SIZE];
        buf[0..4].copy_from_slice(&self.magic.to_le_bytes());
        buf[4..8].copy_from_slice(&self.total_blocks.to_le_bytes());
        buf[8..12].copy_from_slice(&self.inode_table_start.to_le_bytes());
        buf[12..16].copy_from_slice(&self.inode_table_blocks.to_le_bytes());
        buf[16..20].copy_from_slice(&self.bitmap_start.to_le_bytes());
        buf[20..24].copy_from_slice(&self.bitmap_blocks.to_le_bytes());
        buf[24..28].copy_from_slice(&self.data_start.to_le_bytes());
        buf[28..32].copy_from_slice(&self.max_files.to_le_bytes());
        buf
    }

    fn from_bytes(buf: &[u8; BLOCK_SIZE]) -> Self {
        let u32_at = |off: usize| u32::from_le_bytes(buf[off..off + 4].try_into().unwrap());
        Superblock {
            magic: u32_at(0),
            total_blocks: u32_at(4),
            inode_table_start: u32_at(8),
            inode_table_blocks: u32_at(12),
            bitmap_start: u32_at(16),
            bitmap_blocks: u32_at(20),
            data_start: u32_at(24),
            max_files: u32_at(28),
        }
    }
}

#[derive(Clone)]
struct Inode {
    used: bool,
    name: [u8; MAX_NAME_LEN],
    name_len: u8,
    size: u32,
    direct: [u32; DIRECT_POINTERS],
    indirect: u32,
}

impl Inode {
    fn empty() -> Self {
        Inode {
            used: false,
            name: [0; MAX_NAME_LEN],
            name_len: 0,
            size: 0,
            direct: [0; DIRECT_POINTERS],
            indirect: 0,
        }
    }

    fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len as usize]).unwrap_or("")
    }

    fn to_bytes(&self) -> [u8; INODE_SIZE] {
        let mut buf = [0u8; INODE_SIZE];
        buf[0] = self.used as u8;
        buf[1] = self.name_len;
        buf[4..8].copy_from_slice(&self.size.to_le_bytes());
        for (i, ptr) in self.direct.iter().enumerate() {
            buf[8 + i * 4..12 + i * 4].copy_from_slice(&ptr.to_le_bytes());
        }
        buf[32..36].copy_from_slice(&self.indirect.to_le_bytes());
        buf[36..36 + MAX_NAME_LEN].copy_from_slice(&self.name);
        buf
    }

    fn from_bytes(buf: &[u8; INODE_SIZE]) -> Self {
        let u32_at = |off: usize| u32::from_le_bytes(buf[off..off + 4].try_into().unwrap());
        let mut direct = [0u32; DIRECT_POINTERS];
        for (i, slot) in direct.iter_mut().enumerate() {
            *slot = u32_at(8 + i * 4);
        }
        let mut name = [0u8; MAX_NAME_LEN];
        name.copy_from_slice(&buf[36..36 + MAX_NAME_LEN]);
        Inode {
            used: buf[0] != 0,
            name,
            name_len: buf[1],
            size: u32_at(4),
            direct,
            indirect: u32_at(32),
        }
    }
}

pub struct FileSystem<D: BlockDevice> {
    dev: D,
    sb: Superblock,
}

impl<D: BlockDevice> FileSystem<D> {
    pub fn format(mut dev: D) -> Result<Self, FsError<D::Error>> {
        let total_blocks = dev.total_blocks();

        let inode_table_blocks = ceil_div((MAX_FILES * INODE_SIZE) as u32, BLOCK_SIZE as u32);

        let bitmap_blocks = ceil_div(total_blocks, (BLOCK_SIZE * 8) as u32).max(1);

        let inode_table_start = 1;
        let bitmap_start = inode_table_start + inode_table_blocks;
        let data_start = bitmap_start + bitmap_blocks;

        if data_start >= total_blocks {
            return Err(FsError::NoSpace);
        }

        let sb = Superblock {
            magic: MAGIC,
            total_blocks,
            inode_table_start,
            inode_table_blocks,
            bitmap_start,
            bitmap_blocks,
            data_start,
            max_files: MAX_FILES as u32,
        };

        dev.write_block(0, &sb.to_bytes()).map_err(FsError::Io)?;

        let zero = [0u8; BLOCK_SIZE];
        for b in inode_table_start..inode_table_start + inode_table_blocks {
            dev.write_block(b, &zero).map_err(FsError::Io)?;
        }
        for b in bitmap_start..bitmap_start + bitmap_blocks {
            dev.write_block(b, &zero).map_err(FsError::Io)?;
        }

        Ok(FileSystem { dev, sb })
    }

    pub fn mount(mut dev: D) -> Result<Self, FsError<D::Error>> {
        let mut buf = [0u8; BLOCK_SIZE];
        dev.read_block(0, &mut buf).map_err(FsError::Io)?;
        let sb = Superblock::from_bytes(&buf);
        if sb.magic != MAGIC {
            return Err(FsError::NotFormatted);
        }
        Ok(FileSystem { dev, sb })
    }

    pub fn into_device(self) -> D {
        self.dev
    }

    fn read_inode(&mut self, idx: u32) -> Result<Inode, FsError<D::Error>> {
        let per_block = (BLOCK_SIZE / INODE_SIZE) as u32;
        let block = self.sb.inode_table_start + idx / per_block;
        let offset = (idx % per_block) as usize * INODE_SIZE;
        let mut buf = [0u8; BLOCK_SIZE];
        self.dev.read_block(block, &mut buf).map_err(FsError::Io)?;
        let inode_bytes: [u8; INODE_SIZE] = buf[offset..offset + INODE_SIZE].try_into().unwrap();
        Ok(Inode::from_bytes(&inode_bytes))
    }

    fn write_inode(&mut self, idx: u32, inode: &Inode) -> Result<(), FsError<D::Error>> {
        let per_block = (BLOCK_SIZE / INODE_SIZE) as u32;
        let block = self.sb.inode_table_start + idx / per_block;
        let offset = (idx % per_block) as usize * INODE_SIZE;
        let mut buf = [0u8; BLOCK_SIZE];
        self.dev.read_block(block, &mut buf).map_err(FsError::Io)?;
        buf[offset..offset + INODE_SIZE].copy_from_slice(&inode.to_bytes());
        self.dev.write_block(block, &buf).map_err(FsError::Io)?;
        Ok(())
    }

    fn find_inode_by_name(&mut self, name: &str) -> Result<Option<(u32, Inode)>, FsError<D::Error>> {
        for idx in 0..self.sb.max_files {
            let inode = self.read_inode(idx)?;
            if inode.used && inode.name_str() == name {
                return Ok(Some((idx, inode)));
            }
        }
        Ok(None)
    }

    fn find_free_inode(&mut self) -> Result<u32, FsError<D::Error>> {
        for idx in 0..self.sb.max_files {
            if !self.read_inode(idx)?.used {
                return Ok(idx);
            }
        }
        Err(FsError::NoFreeInode)
    }

    fn alloc_block(&mut self) -> Result<u32, FsError<D::Error>> {
        let data_blocks = self.sb.total_blocks - self.sb.data_start;
        let bits_per_block = BLOCK_SIZE as u32 * 8;

        for bitmap_block_idx in 0..self.sb.bitmap_blocks {
            let mut buf = [0u8; BLOCK_SIZE];
            self.dev
                .read_block(self.sb.bitmap_start + bitmap_block_idx, &mut buf)
                .map_err(FsError::Io)?;
            let block_bit_base = bitmap_block_idx * bits_per_block;

            for (byte_idx, &byte) in buf.iter().enumerate() {
                if byte == 0xFF {
                    continue; // all 8 blocks in this byte are taken
                }
                for bit in 0..8u32 {
                    let rel = block_bit_base + byte_idx as u32 * 8 + bit;
                    if rel >= data_blocks {
                        return Err(FsError::NoSpace);
                    }
                    if byte & (1 << bit) == 0 {
                        let mut updated = buf;
                        updated[byte_idx] |= 1 << bit;
                        self.dev
                            .write_block(self.sb.bitmap_start + bitmap_block_idx, &updated)
                            .map_err(FsError::Io)?;
                        return Ok(self.sb.data_start + rel);
                    }
                }
            }
        }
        Err(FsError::NoSpace)
    }

    fn free_block(&mut self, abs_block: u32) -> Result<(), FsError<D::Error>> {
        let rel = abs_block - self.sb.data_start;
        let byte_idx = rel / 8;
        let bit_idx = rel % 8;
        let block = self.sb.bitmap_start + byte_idx / BLOCK_SIZE as u32;
        let offset = (byte_idx % BLOCK_SIZE as u32) as usize;
        let mut buf = [0u8; BLOCK_SIZE];
        self.dev.read_block(block, &mut buf).map_err(FsError::Io)?;
        buf[offset] &= !(1 << bit_idx);
        self.dev.write_block(block, &buf).map_err(FsError::Io)?;
        Ok(())
    }

    fn free_inode_blocks(&mut self, inode: &Inode) -> Result<(), FsError<D::Error>> {
        for &b in inode.direct.iter() {
            if b != 0 {
                self.free_block(b)?;
            }
        }
        if inode.indirect != 0 {
            let mut buf = [0u8; BLOCK_SIZE];
            self.dev.read_block(inode.indirect, &mut buf).map_err(FsError::Io)?;
            for i in 0..POINTERS_PER_BLOCK {
                let p = u32::from_le_bytes(buf[i * 4..i * 4 + 4].try_into().unwrap());
                if p != 0 {
                    self.free_block(p)?;
                }
            }
            self.free_block(inode.indirect)?;
        }
        Ok(())
    }

    pub fn list_files(&mut self) -> Result<Vec<String>, FsError<D::Error>> {
        let mut names = Vec::new();
        for idx in 0..self.sb.max_files {
            let inode = self.read_inode(idx)?;
            if inode.used {
                names.push(String::from(inode.name_str()));
            }
        }
        Ok(names)
    }

    pub fn create_file(&mut self, name: &str) -> Result<(), FsError<D::Error>> {
        if name.len() > MAX_NAME_LEN {
            return Err(FsError::NameTooLong);
        }
        if self.find_inode_by_name(name)?.is_some() {
            return Err(FsError::AlreadyExists);
        }
        let idx = self.find_free_inode()?;
        let mut inode = Inode::empty();
        inode.used = true;
        inode.name_len = name.len() as u8;
        inode.name[..name.len()].copy_from_slice(name.as_bytes());
        self.write_inode(idx, &inode)?;
        Ok(())
    }

    pub fn write_file(&mut self, name: &str, data: &[u8]) -> Result<(), FsError<D::Error>> {
        let (idx, mut inode) = self.find_inode_by_name(name)?.ok_or(FsError::NotFound)?;

        self.free_inode_blocks(&inode)?;
        inode.direct = [0; DIRECT_POINTERS];
        inode.indirect = 0;

        let blocks_needed = ceil_div(data.len() as u32, BLOCK_SIZE as u32);
        if blocks_needed as usize > DIRECT_POINTERS + POINTERS_PER_BLOCK {
            return Err(FsError::FileTooLarge);
        }

        let mut indirect_ptrs: Option<Vec<u32>> = None;

        for i in 0..blocks_needed {
            let block = self.alloc_block()?;
            let start = i as usize * BLOCK_SIZE;
            let end = core::cmp::min(start + BLOCK_SIZE, data.len());
            let mut buf = [0u8; BLOCK_SIZE];
            buf[..end - start].copy_from_slice(&data[start..end]);
            self.dev.write_block(block, &buf).map_err(FsError::Io)?;

            if (i as usize) < DIRECT_POINTERS {
                inode.direct[i as usize] = block;
            } else {
                let ptrs = indirect_ptrs.get_or_insert_with(|| vec![0u32; POINTERS_PER_BLOCK]);
                ptrs[i as usize - DIRECT_POINTERS] = block;
            }
        }

        if let Some(ptrs) = indirect_ptrs {
            let indirect_block = self.alloc_block()?;
            let mut buf = [0u8; BLOCK_SIZE];
            for (i, p) in ptrs.iter().enumerate() {
                buf[i * 4..i * 4 + 4].copy_from_slice(&p.to_le_bytes());
            }
            self.dev.write_block(indirect_block, &buf).map_err(FsError::Io)?;
            inode.indirect = indirect_block;
        }

        inode.size = data.len() as u32;
        self.write_inode(idx, &inode)?;
        Ok(())
    }

    pub fn read_file(&mut self, name: &str) -> Result<Vec<u8>, FsError<D::Error>> {
        let (_, inode) = self.find_inode_by_name(name)?.ok_or(FsError::NotFound)?;
        let mut out = Vec::with_capacity(inode.size as usize);

        let blocks_needed = ceil_div(inode.size, BLOCK_SIZE as u32);
        let mut indirect_ptrs: Vec<u32> = Vec::new();
        if inode.indirect != 0 {
            let mut buf = [0u8; BLOCK_SIZE];
            self.dev.read_block(inode.indirect, &mut buf).map_err(FsError::Io)?;
            for i in 0..POINTERS_PER_BLOCK {
                indirect_ptrs.push(u32::from_le_bytes(buf[i * 4..i * 4 + 4].try_into().unwrap()));
            }
        }

        for i in 0..blocks_needed {
            let block = if (i as usize) < DIRECT_POINTERS {
                inode.direct[i as usize]
            } else {
                indirect_ptrs[i as usize - DIRECT_POINTERS]
            };
            let mut buf = [0u8; BLOCK_SIZE];
            self.dev.read_block(block, &mut buf).map_err(FsError::Io)?;
            let remaining = inode.size as usize - out.len();
            let take = core::cmp::min(BLOCK_SIZE, remaining);
            out.extend_from_slice(&buf[..take]);
        }

        Ok(out)
    }

    pub fn delete_file(&mut self, name: &str) -> Result<(), FsError<D::Error>> {
        let (idx, inode) = self.find_inode_by_name(name)?.ok_or(FsError::NotFound)?;
        self.free_inode_blocks(&inode)?;
        self.write_inode(idx, &Inode::empty())?;
        Ok(())
    }
}

// testing [`FileSystem`] without needing real disk hardware attached to QEMU.
pub struct MemBlockDevice {
    blocks: Vec<[u8; BLOCK_SIZE]>,
}

impl MemBlockDevice {
    pub fn new(total_blocks: u32) -> Self {
        MemBlockDevice {
            blocks: vec![[0u8; BLOCK_SIZE]; total_blocks as usize],
        }
    }
}

impl BlockDevice for MemBlockDevice {
    type Error = core::convert::Infallible;

    fn total_blocks(&self) -> u32 {
        self.blocks.len() as u32
    }

    fn read_block(&mut self, block: u32, buf: &mut [u8; BLOCK_SIZE]) -> Result<(), Self::Error> {
        *buf = self.blocks[block as usize];
        Ok(())
    }

    fn write_block(&mut self, block: u32, buf: &[u8; BLOCK_SIZE]) -> Result<(), Self::Error> {
        self.blocks[block as usize] = *buf;
        Ok(())
    }
}