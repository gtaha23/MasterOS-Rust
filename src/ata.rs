//! Minimal ATA PIO-mode disk driver (primary bus, master drive, 28-bit LBA).
//!
//! This is polling-based, not interrupt-driven: commands busy-wait on the
//! status register instead of registering an IRQ handler. That trades some
//! CPU efficiency for simplicity and makes it safe to call from anywhere
//! (no coordination needed with the IDT/PIC setup in `interrupts.rs`).
//!
//! Only 28-bit LBA PIO read/write is implemented — good for up to 128GiB
//! disks and for QEMU's default IDE emulation. No DMA, no 48-bit LBA, no
//! secondary bus or slave drive support (yet).

use x86_64::instructions::port::{Port, PortWriteOnly};
use crate::block_device::{BlockDevice, BLOCK_SIZE};

const DATA: u16 = 0x1F0;
const ERROR: u16 = 0x1F1;
const SECTOR_COUNT: u16 = 0x1F2;
const LBA_LO: u16 = 0x1F3;
const LBA_MID: u16 = 0x1F4;
const LBA_HI: u16 = 0x1F5;
const DRIVE_HEAD: u16 = 0x1F6;
const STATUS: u16 = 0x1F7;
const COMMAND: u16 = 0x1F7;

const CMD_READ_SECTORS: u8 = 0x20;
const CMD_WRITE_SECTORS: u8 = 0x30;
const CMD_IDENTIFY: u8 = 0xEC;

const STATUS_ERR: u8 = 1 << 0;
const STATUS_DRQ: u8 = 1 << 3;
const STATUS_BSY: u8 = 1 << 7;

pub const SECTOR_SIZE: usize = 512;

#[derive(Debug)]
pub enum AtaError {
    DeviceError(u8),
    NoDrive,
    Timeout,
}

pub struct AtaDisk {
    data: Port<u16>,
    error: Port<u8>,
    sector_count: Port<u8>,
    lba_lo: Port<u8>,
    lba_mid: Port<u8>,
    lba_hi: Port<u8>,
    drive_head: Port<u8>,
    status: Port<u8>,
    command: PortWriteOnly<u8>,
}

impl AtaDisk {
    pub const fn primary_master() -> Self {
        AtaDisk {
            data: Port::new(DATA),
            error: Port::new(ERROR),
            sector_count: Port::new(SECTOR_COUNT),
            lba_lo: Port::new(LBA_LO),
            lba_mid: Port::new(LBA_MID),
            lba_hi: Port::new(LBA_HI),
            drive_head: Port::new(DRIVE_HEAD),
            status: Port::new(STATUS),
            command: PortWriteOnly::new(COMMAND),
        }
    }

    fn wait_while_busy(&mut self) -> Result<(), AtaError> {
        for _ in 0..100_000 {
            if unsafe { self.status.read() } & STATUS_BSY == 0 {
                return Ok(());
            }
        }
        Err(AtaError::Timeout)
    }

    fn wait_for_drq(&mut self) -> Result<(), AtaError> {
        for _ in 0..100_000 {
            let status = unsafe { self.status.read() };
            if status & STATUS_ERR != 0 {
                return Err(AtaError::DeviceError(unsafe { self.error.read() }));
            }
            if status & STATUS_DRQ != 0 {
                return Ok(());
            }
        }
        Err(AtaError::Timeout)
    }

    pub fn identify(&mut self) -> Result<[u16; 256], AtaError> {
        unsafe {
            self.drive_head.write(0xA0);
            self.sector_count.write(0);
            self.lba_lo.write(0);
            self.lba_mid.write(0);
            self.lba_hi.write(0);
            self.command.write(CMD_IDENTIFY);
        }

        if unsafe { self.status.read() } == 0 {
            return Err(AtaError::NoDrive);
        }

        self.wait_while_busy()?;
        self.wait_for_drq()?;

        let mut buf = [0u16; 256];
        for word in buf.iter_mut() {
            *word = unsafe { self.data.read() };
        }
        Ok(buf)
    }

    pub fn read_sectors(&mut self, lba: u32, count: u8, buf: &mut [u16]) -> Result<(), AtaError> {
        assert!(
            buf.len() >= count as usize * 256,
            "buffer too small for requested sector count"
        );
        assert!(lba < 1 << 28, "LBA out of 28-bit range");

        self.wait_while_busy()?;
        unsafe {
            self.drive_head.write(0xE0 | ((lba >> 24) & 0x0F) as u8);
            self.sector_count.write(count);
            self.lba_lo.write((lba & 0xFF) as u8);
            self.lba_mid.write(((lba >> 8) & 0xFF) as u8);
            self.lba_hi.write(((lba >> 16) & 0xFF) as u8);
            self.command.write(CMD_READ_SECTORS);
        }

        for sector in 0..count as usize {
            self.wait_while_busy()?;
            self.wait_for_drq()?;
            for i in 0..256 {
                buf[sector * 256 + i] = unsafe { self.data.read() };
            }
        }

        Ok(())
    }

    pub fn write_sectors(&mut self, lba: u32, count: u8, buf: &[u16]) -> Result<(), AtaError> {
        assert!(
            buf.len() >= count as usize * 256,
            "buffer too small for requested sector count"
        );
        assert!(lba < 1 << 28, "LBA out of 28-bit range");

        self.wait_while_busy()?;
        unsafe {
            self.drive_head.write(0xE0 | ((lba >> 24) & 0x0F) as u8);
            self.sector_count.write(count);
            self.lba_lo.write((lba & 0xFF) as u8);
            self.lba_mid.write(((lba >> 8) & 0xFF) as u8);
            self.lba_hi.write(((lba >> 16) & 0xFF) as u8);
            self.command.write(CMD_WRITE_SECTORS);
        }

        for sector in 0..count as usize {
            self.wait_while_busy()?;
            self.wait_for_drq()?;
            for i in 0..256 {
                unsafe { self.data.write(buf[sector * 256 + i]) };
            }
        }

        Ok(())
    }
}

pub struct AtaBlockDevice {
    disk: AtaDisk,
    total_sectors: u32,
}

impl AtaBlockDevice {
    pub fn primary_master() -> Result<Self, AtaError> {
        let mut disk = AtaDisk::primary_master();
        let id = disk.identify()?;
        let total_sectors = (id[60] as u32) | ((id[61] as u32) << 16);
        Ok(AtaBlockDevice { disk, total_sectors })
    }
}

impl BlockDevice for AtaBlockDevice {
    type Error = AtaError;

    fn total_blocks(&self) -> u32 {
        self.total_sectors
    }

    fn read_block(&mut self, block: u32, buf: &mut [u8; BLOCK_SIZE]) -> Result<(), AtaError> {
        let mut words = [0u16; BLOCK_SIZE / 2];
        self.disk.read_sectors(block, 1, &mut words)?;
        for i in 0..BLOCK_SIZE / 2 {
            let bytes = words[i].to_le_bytes();
            buf[i * 2] = bytes[0];
            buf[i * 2 + 1] = bytes[1];
        }
        Ok(())
    }

    fn write_block(&mut self, block: u32, buf: &[u8; BLOCK_SIZE]) -> Result<(), AtaError> {
        let mut words = [0u16; BLOCK_SIZE / 2];
        for i in 0..BLOCK_SIZE / 2 {
            words[i] = u16::from_le_bytes([buf[i * 2], buf[i * 2 + 1]]);
        }
        self.disk.write_sectors(block, 1, &words)
    }
}