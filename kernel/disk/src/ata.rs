// kernel/disk/src/ata.rs
// ATA/IDE Disk Driver
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]

use crate::block_device::*;

// ATA I/O Ports
pub const ATA_PRIMARY_IO: u16 = 0x1F0;
pub const ATA_SECONDARY_IO: u16 = 0x170;
pub const ATA_PRIMARY_CTRL: u16 = 0x3F6;
pub const ATA_SECONDARY_CTRL: u16 = 0x376;

// ATA Registers (offset from base)
pub const ATA_REG_DATA: u16 = 0;
pub const ATA_REG_ERROR: u16 = 1;
pub const ATA_REG_FEATURES: u16 = 1;
pub const ATA_REG_SECCOUNT: u16 = 2;
pub const ATA_REG_LBA_LO: u16 = 3;
pub const ATA_REG_LBA_MID: u16 = 4;
pub const ATA_REG_LBA_HI: u16 = 5;
pub const ATA_REG_DEVICE: u16 = 6;
pub const ATA_REG_STATUS: u16 = 7;
pub const ATA_REG_COMMAND: u16 = 7;

// ATA Commands
pub const ATA_CMD_READ_PIO: u8 = 0x20;
pub const ATA_CMD_READ_PIO_EXT: u8 = 0x24;
pub const ATA_CMD_WRITE_PIO: u8 = 0x30;
pub const ATA_CMD_WRITE_PIO_EXT: u8 = 0x34;
pub const ATA_CMD_CACHE_FLUSH: u8 = 0xE7;
pub const ATA_CMD_CACHE_FLUSH_EXT: u8 = 0xEA;
pub const ATA_CMD_IDENTIFY: u8 = 0xEC;

// ATA Status bits
pub const ATA_SR_BSY: u8 = 0x80; // Busy
pub const ATA_SR_DRDY: u8 = 0x40; // Drive ready
pub const ATA_SR_DF: u8 = 0x20; // Drive fault
pub const ATA_SR_DSC: u8 = 0x10; // Drive seek complete
pub const ATA_SR_DRQ: u8 = 0x08; // Data request
pub const ATA_SR_ERR: u8 = 0x01; // Error

pub struct AtaDisk {
    pub base: u16,
    pub ctrl: u16,
    pub is_master: bool,
    pub info: DiskInfo,
}

impl AtaDisk {
    pub const fn new(base: u16, ctrl: u16, is_master: bool) -> Self {
        Self {
            base,
            ctrl,
            is_master,
            info: DiskInfo::empty(),
        }
    }

    fn wait_ready(&self) -> Result<(), DiskError> {
        for _ in 0..10000 {
            let status = self.inb(ATA_REG_STATUS);
            if (status & ATA_SR_BSY) == 0 {
                return Ok(());
            }
            // Small delay
            for _ in 0..100 {
                unsafe {
                    core::arch::asm!("nop");
                }
            }
        }
        Err(DiskError::Timeout)
    }

    fn wait_drq(&self) -> Result<(), DiskError> {
        for _ in 0..10000 {
            let status = self.inb(ATA_REG_STATUS);
            if (status & ATA_SR_DRQ) != 0 {
                return Ok(());
            }
            if (status & ATA_SR_ERR) != 0 {
                return Err(DiskError::ReadError);
            }
            for _ in 0..100 {
                unsafe {
                    core::arch::asm!("nop");
                }
            }
        }
        Err(DiskError::Timeout)
    }

    fn select_drive(&self) {
        let drive_select = if self.is_master { 0xA0 } else { 0xB0 };
        self.outb(ATA_REG_DEVICE, drive_select);

        // 400ns delay (read status 15 times)
        for _ in 0..15 {
            self.inb(ATA_REG_STATUS);
        }
    }

    fn inb(&self, reg: u16) -> u8 {
        unsafe {
            let mut value: u8;
            core::arch::asm!(
                "in al, dx",
                in("dx") self.base + reg,
                out("al") value,
                options(nostack, preserves_flags)
            );
            value
        }
    }

    fn outb(&self, reg: u16, value: u8) {
        unsafe {
            core::arch::asm!(
                "out dx, al",
                in("dx") self.base + reg,
                in("al") value,
                options(nostack, preserves_flags)
            );
        }
    }

    fn inw(&self, reg: u16) -> u16 {
        unsafe {
            let mut value: u16;
            core::arch::asm!(
                "in ax, dx",
                in("dx") self.base + reg,
                out("ax") value,
                options(nostack, preserves_flags)
            );
            value
        }
    }

    fn outw(&self, reg: u16, value: u16) {
        unsafe {
            core::arch::asm!(
                "out dx, ax",
                in("dx") self.base + reg,
                in("ax") value,
                options(nostack, preserves_flags)
            );
        }
    }
}

impl DiskDriver for AtaDisk {
    fn read_sectors(&mut self, lba: u64, count: u32, buf: &mut [u8]) -> Result<(), DiskError> {
        if buf.len() < (count as usize * SECTOR_SIZE) {
            return Err(DiskError::InvalidBlock);
        }

        self.wait_ready()?;
        self.select_drive();

        let use_lba48 = lba >= 0x10000000 || count > 256;

        if use_lba48 {
            // LBA48 mode
            self.outb(ATA_REG_SECCOUNT, ((count >> 8) & 0xFF) as u8);
            self.outb(ATA_REG_LBA_LO, ((lba >> 24) & 0xFF) as u8);
            self.outb(ATA_REG_LBA_MID, ((lba >> 32) & 0xFF) as u8);
            self.outb(ATA_REG_LBA_HI, ((lba >> 40) & 0xFF) as u8);

            self.outb(ATA_REG_SECCOUNT, (count & 0xFF) as u8);
            self.outb(ATA_REG_LBA_LO, (lba & 0xFF) as u8);
            self.outb(ATA_REG_LBA_MID, ((lba >> 8) & 0xFF) as u8);
            self.outb(ATA_REG_LBA_HI, ((lba >> 16) & 0xFF) as u8);

            self.outb(ATA_REG_COMMAND, ATA_CMD_READ_PIO_EXT);
        } else {
            // LBA28 mode
            let drive_head = if self.is_master { 0xE0 } else { 0xF0 };
            self.outb(ATA_REG_DEVICE, drive_head | (((lba >> 24) & 0x0F) as u8));

            self.outb(ATA_REG_SECCOUNT, (count & 0xFF) as u8);
            self.outb(ATA_REG_LBA_LO, (lba & 0xFF) as u8);
            self.outb(ATA_REG_LBA_MID, ((lba >> 8) & 0xFF) as u8);
            self.outb(ATA_REG_LBA_HI, ((lba >> 16) & 0xFF) as u8);

            self.outb(ATA_REG_COMMAND, ATA_CMD_READ_PIO);
        }

        // Read data
        for sector in 0..count as usize {
            self.wait_drq()?;

            let offset = sector * SECTOR_SIZE;
            for word in 0..(SECTOR_SIZE / 2) {
                let data = self.inw(ATA_REG_DATA);
                buf[offset + word * 2] = (data & 0xFF) as u8;
                buf[offset + word * 2 + 1] = ((data >> 8) & 0xFF) as u8;
            }
        }

        Ok(())
    }

    fn write_sectors(&mut self, lba: u64, count: u32, buf: &[u8]) -> Result<(), DiskError> {
        if buf.len() < (count as usize * SECTOR_SIZE) {
            return Err(DiskError::InvalidBlock);
        }

        self.wait_ready()?;
        self.select_drive();

        let use_lba48 = lba >= 0x10000000 || count > 256;

        if use_lba48 {
            // LBA48 mode
            self.outb(ATA_REG_SECCOUNT, ((count >> 8) & 0xFF) as u8);
            self.outb(ATA_REG_LBA_LO, ((lba >> 24) & 0xFF) as u8);
            self.outb(ATA_REG_LBA_MID, ((lba >> 32) & 0xFF) as u8);
            self.outb(ATA_REG_LBA_HI, ((lba >> 40) & 0xFF) as u8);

            self.outb(ATA_REG_SECCOUNT, (count & 0xFF) as u8);
            self.outb(ATA_REG_LBA_LO, (lba & 0xFF) as u8);
            self.outb(ATA_REG_LBA_MID, ((lba >> 8) & 0xFF) as u8);
            self.outb(ATA_REG_LBA_HI, ((lba >> 16) & 0xFF) as u8);

            self.outb(ATA_REG_COMMAND, ATA_CMD_WRITE_PIO_EXT);
        } else {
            // LBA28 mode
            let drive_head = if self.is_master { 0xE0 } else { 0xF0 };
            self.outb(ATA_REG_DEVICE, drive_head | (((lba >> 24) & 0x0F) as u8));

            self.outb(ATA_REG_SECCOUNT, (count & 0xFF) as u8);
            self.outb(ATA_REG_LBA_LO, (lba & 0xFF) as u8);
            self.outb(ATA_REG_LBA_MID, ((lba >> 8) & 0xFF) as u8);
            self.outb(ATA_REG_LBA_HI, ((lba >> 16) & 0xFF) as u8);

            self.outb(ATA_REG_COMMAND, ATA_CMD_WRITE_PIO);
        }

        // Write data
        for sector in 0..count as usize {
            self.wait_drq()?;

            let offset = sector * SECTOR_SIZE;
            for word in 0..(SECTOR_SIZE / 2) {
                let data =
                    buf[offset + word * 2] as u16 | ((buf[offset + word * 2 + 1] as u16) << 8);
                self.outw(ATA_REG_DATA, data);
            }
        }

        self.wait_ready()?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), DiskError> {
        self.wait_ready()?;
        self.select_drive();

        let cmd = if self.info.supports_lba48 {
            ATA_CMD_CACHE_FLUSH_EXT
        } else {
            ATA_CMD_CACHE_FLUSH
        };

        self.outb(ATA_REG_COMMAND, cmd);
        self.wait_ready()?;

        Ok(())
    }

    fn identify(&mut self) -> Result<DiskInfo, DiskError> {
        self.wait_ready()?;
        self.select_drive();

        // Send IDENTIFY command
        self.outb(ATA_REG_COMMAND, ATA_CMD_IDENTIFY);

        // Check if drive exists
        let status = self.inb(ATA_REG_STATUS);
        if status == 0 {
            return Err(DiskError::DeviceNotFound);
        }

        self.wait_drq()?;

        // Read 256 words (512 bytes)
        let mut identify_data = [0u16; 256];
        for i in 0..256 {
            identify_data[i] = self.inw(ATA_REG_DATA);
        }

        // Parse identify data
        let mut info = DiskInfo::empty();

        // Model string (words 27-46)
        for i in 0..20 {
            let word = identify_data[27 + i];
            info.model[i * 2] = ((word >> 8) & 0xFF) as u8;
            info.model[i * 2 + 1] = (word & 0xFF) as u8;
        }

        // Serial number (words 10-19)
        for i in 0..10 {
            let word = identify_data[10 + i];
            info.serial[i * 2] = ((word >> 8) & 0xFF) as u8;
            info.serial[i * 2 + 1] = (word & 0xFF) as u8;
        }

        // Firmware revision (words 23-26)
        for i in 0..4 {
            let word = identify_data[23 + i];
            info.firmware[i * 2] = ((word >> 8) & 0xFF) as u8;
            info.firmware[i * 2 + 1] = (word & 0xFF) as u8;
        }

        // LBA48 support (word 83, bit 10)
        info.supports_lba48 = (identify_data[83] & (1 << 10)) != 0;

        // DMA support (word 49, bit 8)
        info.supports_dma = (identify_data[49] & (1 << 8)) != 0;

        // Total sectors
        if info.supports_lba48 {
            // LBA48: words 100-103
            info.total_sectors = identify_data[100] as u64
                | ((identify_data[101] as u64) << 16)
                | ((identify_data[102] as u64) << 32)
                | ((identify_data[103] as u64) << 48);
        } else {
            // LBA28: words 60-61
            info.total_sectors = identify_data[60] as u64 | ((identify_data[61] as u64) << 16);
        }

        info.sector_size = 512;

        self.info = info;
        Ok(info)
    }
}

pub fn probe_ata_disks() -> usize {
    let mut count = 0u32;

    // Probe primary master
    let mut primary_master = AtaDisk::new(ATA_PRIMARY_IO, ATA_PRIMARY_CTRL, true);
    if let Ok(info) = primary_master.identify() {
        let mut device = BlockDevice::new(count, DriverType::ATA);
        device.init(info);
        if register_disk(device).is_some() {
            count += 1;
        }
    }

    // Probe primary slave
    let mut primary_slave = AtaDisk::new(ATA_PRIMARY_IO, ATA_PRIMARY_CTRL, false);
    if let Ok(info) = primary_slave.identify() {
        let mut device = BlockDevice::new(count, DriverType::ATA);
        device.init(info);
        if register_disk(device).is_some() {
            count += 1;
        }
    }

    // Probe secondary master
    let mut secondary_master = AtaDisk::new(ATA_SECONDARY_IO, ATA_SECONDARY_CTRL, true);
    if let Ok(info) = secondary_master.identify() {
        let mut device = BlockDevice::new(count, DriverType::ATA);
        device.init(info);
        if register_disk(device).is_some() {
            count += 1;
        }
    }

    // Probe secondary slave
    let mut secondary_slave = AtaDisk::new(ATA_SECONDARY_IO, ATA_SECONDARY_CTRL, false);
    if let Ok(info) = secondary_slave.identify() {
        let mut device = BlockDevice::new(count, DriverType::ATA);
        device.init(info);
        if register_disk(device).is_some() {
            count += 1;
        }
    }

    count as usize
}
