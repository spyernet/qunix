use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::hal::drivers::pci::{PciDevice, find_ahci_controllers, enable_bus_mastering, enable_memory_space, get_bar_address};
use crate::println;

const AHCI_CAP: u32 = 0x00;
const AHCI_GHC: u32 = 0x04;
const AHCI_IS: u32 = 0x08;
const AHCI_PI: u32 = 0x0C;
const AHCI_VS: u32 = 0x10;

const AHCI_GHC_AE: u32 = 1 << 31;
const AHCI_GHC_IE: u32 = 1 << 1;
const AHCI_GHC_HR: u32 = 1 << 0;

const PORT_CLB: u32 = 0x00;
const PORT_CLBU: u32 = 0x04;
const PORT_FB: u32 = 0x08;
const PORT_FBU: u32 = 0x0C;
const PORT_IS: u32 = 0x10;
const PORT_IE: u32 = 0x14;
const PORT_CMD: u32 = 0x18;
const PORT_TFD: u32 = 0x20;
const PORT_SIG: u32 = 0x24;
const PORT_SSTS: u32 = 0x28;
const PORT_SCTL: u32 = 0x2C;
const PORT_SERR: u32 = 0x30;
const PORT_SACT: u32 = 0x34;
const PORT_CI: u32 = 0x38;

const PORT_CMD_ST: u32 = 1 << 0;
const PORT_CMD_FRE: u32 = 1 << 4;
const PORT_CMD_FR: u32 = 1 << 14;
const PORT_CMD_CR: u32 = 1 << 15;

const SATA_SIG_ATA: u32 = 0x00000101;
const SATA_SIG_ATAPI: u32 = 0xEB140101;
const SATA_SIG_SEMB: u32 = 0xC33C0101;
const SATA_SIG_PM: u32 = 0x96690101;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortType {
    None,
    Sata,
    Satapi,
    Semb,
    Pm,
}

#[derive(Debug, Clone)]
pub struct AhciPort {
    pub port_num: u8,
    pub port_type: PortType,
    pub base_addr: u64,
    pub implemented: bool,
}

#[derive(Debug, Clone)]
pub struct AhciController {
    pub pci_device: PciDevice,
    pub abar: u64,
    pub ports: Vec<AhciPort>,
    pub version: u32,
    pub port_count: u8,
    pub command_slots: u8,
}

lazy_static! {
    static ref AHCI_CONTROLLERS: Mutex<Vec<AhciController>> = Mutex::new(Vec::new());
}

fn read_reg(abar: u64, offset: u32) -> u32 {
    let addr = (abar + offset as u64) as *const u32;
    unsafe { core::ptr::read_volatile(addr) }
}

fn write_reg(abar: u64, offset: u32, value: u32) {
    let addr = (abar + offset as u64) as *mut u32;
    unsafe { core::ptr::write_volatile(addr, value) }
}

fn read_port_reg(abar: u64, port: u8, offset: u32) -> u32 {
    let port_base = 0x100 + (port as u32 * 0x80);
    read_reg(abar, port_base + offset)
}

fn write_port_reg(abar: u64, port: u8, offset: u32, value: u32) {
    let port_base = 0x100 + (port as u32 * 0x80);
    write_reg(abar, port_base + offset, value)
}

fn get_port_type(abar: u64, port: u8) -> PortType {
    let ssts = read_port_reg(abar, port, PORT_SSTS);
    let ipm = (ssts >> 8) & 0x0F;
    let det = ssts & 0x0F;
    
    if det != 3 || ipm != 1 {
        return PortType::None;
    }
    
    let sig = read_port_reg(abar, port, PORT_SIG);
    match sig {
        SATA_SIG_ATA => PortType::Sata,
        SATA_SIG_ATAPI => PortType::Satapi,
        SATA_SIG_SEMB => PortType::Semb,
        SATA_SIG_PM => PortType::Pm,
        _ => PortType::None,
    }
}

fn stop_port(abar: u64, port: u8) {
    let cmd = read_port_reg(abar, port, PORT_CMD);
    
    if cmd & PORT_CMD_ST != 0 {
        write_port_reg(abar, port, PORT_CMD, cmd & !PORT_CMD_ST);
    }
    
    if cmd & PORT_CMD_FRE != 0 {
        write_port_reg(abar, port, PORT_CMD, cmd & !PORT_CMD_FRE);
    }
    
    for _ in 0..1000 {
        let cmd = read_port_reg(abar, port, PORT_CMD);
        if (cmd & PORT_CMD_FR) == 0 && (cmd & PORT_CMD_CR) == 0 {
            break;
        }
        crate::hal::drivers::pit::busy_wait_us(1000);
    }
}

fn start_port(abar: u64, port: u8) {
    for _ in 0..1000 {
        let cmd = read_port_reg(abar, port, PORT_CMD);
        if (cmd & PORT_CMD_CR) == 0 {
            break;
        }
        crate::hal::drivers::pit::busy_wait_us(1000);
    }
    
    let cmd = read_port_reg(abar, port, PORT_CMD);
    write_port_reg(abar, port, PORT_CMD, cmd | PORT_CMD_FRE | PORT_CMD_ST);
}

pub fn init() {
    let controllers = find_ahci_controllers();
    
    for pci_dev in controllers {
        enable_bus_mastering(&pci_dev);
        enable_memory_space(&pci_dev);
        
        let abar = get_bar_address(pci_dev.bar[5]);
        if abar == 0 {
            continue;
        }
        
        let ghc = read_reg(abar, AHCI_GHC);
        write_reg(abar, AHCI_GHC, ghc | AHCI_GHC_AE);
        
        let cap = read_reg(abar, AHCI_CAP);
        let version = read_reg(abar, AHCI_VS);
        let pi = read_reg(abar, AHCI_PI);
        
        let port_count = ((cap & 0x1F) + 1) as u8;
        let command_slots = (((cap >> 8) & 0x1F) + 1) as u8;
        
        let mut ports = Vec::new();
        
        for i in 0..32 {
            if pi & (1 << i) != 0 {
                let port_type = get_port_type(abar, i);
                let port_base = abar + 0x100 + (i as u64 * 0x80);
                
                ports.push(AhciPort {
                    port_num: i,
                    port_type,
                    base_addr: port_base,
                    implemented: true,
                });
                
                if port_type != PortType::None {
                    println!("  [AHCI] Port {} - {:?}", i, port_type);
                }
            }
        }
        
        let controller = AhciController {
            pci_device: pci_dev,
            abar,
            ports,
            version,
            port_count,
            command_slots,
        };
        
        AHCI_CONTROLLERS.lock().push(controller);
    }
}

pub fn get_controllers() -> Vec<AhciController> {
    AHCI_CONTROLLERS.lock().clone()
}

pub fn get_sata_ports() -> Vec<AhciPort> {
    AHCI_CONTROLLERS.lock()
        .iter()
        .flat_map(|c| c.ports.iter())
        .filter(|p| p.port_type == PortType::Sata)
        .cloned()
        .collect()
}

#[repr(C, packed)]
pub struct FisRegH2D {
    pub fis_type: u8,
    pub flags: u8,
    pub command: u8,
    pub features_low: u8,
    pub lba0: u8,
    pub lba1: u8,
    pub lba2: u8,
    pub device: u8,
    pub lba3: u8,
    pub lba4: u8,
    pub lba5: u8,
    pub features_high: u8,
    pub count_low: u8,
    pub count_high: u8,
    pub icc: u8,
    pub control: u8,
    pub reserved: [u8; 4],
}

impl FisRegH2D {
    pub fn new() -> Self {
        FisRegH2D {
            fis_type: 0x27,
            flags: 0,
            command: 0,
            features_low: 0,
            lba0: 0,
            lba1: 0,
            lba2: 0,
            device: 0,
            lba3: 0,
            lba4: 0,
            lba5: 0,
            features_high: 0,
            count_low: 0,
            count_high: 0,
            icc: 0,
            control: 0,
            reserved: [0; 4],
        }
    }
    
    pub fn set_command(&mut self, cmd: u8) {
        self.command = cmd;
        self.flags |= 0x80;
    }
    
    pub fn set_lba(&mut self, lba: u64) {
        self.lba0 = (lba & 0xFF) as u8;
        self.lba1 = ((lba >> 8) & 0xFF) as u8;
        self.lba2 = ((lba >> 16) & 0xFF) as u8;
        self.lba3 = ((lba >> 24) & 0xFF) as u8;
        self.lba4 = ((lba >> 32) & 0xFF) as u8;
        self.lba5 = ((lba >> 40) & 0xFF) as u8;
        self.device = 0x40;
    }
    
    pub fn set_count(&mut self, count: u16) {
        self.count_low = (count & 0xFF) as u8;
        self.count_high = ((count >> 8) & 0xFF) as u8;
    }
}

pub const ATA_CMD_READ_DMA_EXT: u8 = 0x25;
pub const ATA_CMD_WRITE_DMA_EXT: u8 = 0x35;
pub const ATA_CMD_IDENTIFY: u8 = 0xEC;
pub const ATA_CMD_FLUSH_CACHE_EXT: u8 = 0xEA;
