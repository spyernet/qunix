use x86_64::instructions::port::Port;
use spin::Mutex;
use lazy_static::lazy_static;
use alloc::vec::Vec;
use crate::println;

const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PciAddress {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

impl PciAddress {
    pub fn new(bus: u8, device: u8, function: u8) -> Self {
        PciAddress { bus, device, function }
    }
    
    fn config_address(&self, offset: u8) -> u32 {
        let bus = self.bus as u32;
        let device = self.device as u32;
        let function = self.function as u32;
        let offset = (offset & 0xFC) as u32;
        
        (1 << 31) | (bus << 16) | (device << 11) | (function << 8) | offset
    }
}

#[derive(Debug, Clone)]
pub struct PciDevice {
    pub address: PciAddress,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision_id: u8,
    pub header_type: u8,
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub bar: [u32; 6],
}

impl PciDevice {
    pub fn is_mass_storage(&self) -> bool {
        self.class_code == 0x01
    }
    
    pub fn is_network(&self) -> bool {
        self.class_code == 0x02
    }
    
    pub fn is_display(&self) -> bool {
        self.class_code == 0x03
    }
    
    pub fn is_bridge(&self) -> bool {
        self.class_code == 0x06
    }
    
    pub fn is_usb(&self) -> bool {
        self.class_code == 0x0C && self.subclass == 0x03
    }
    
    pub fn is_ahci(&self) -> bool {
        self.class_code == 0x01 && self.subclass == 0x06 && self.prog_if == 0x01
    }
    
    pub fn class_name(&self) -> &'static str {
        match self.class_code {
            0x00 => "Unclassified",
            0x01 => "Mass Storage Controller",
            0x02 => "Network Controller",
            0x03 => "Display Controller",
            0x04 => "Multimedia Controller",
            0x05 => "Memory Controller",
            0x06 => "Bridge",
            0x07 => "Simple Communication Controller",
            0x08 => "Base System Peripheral",
            0x09 => "Input Device Controller",
            0x0A => "Docking Station",
            0x0B => "Processor",
            0x0C => "Serial Bus Controller",
            0x0D => "Wireless Controller",
            0x0E => "Intelligent Controller",
            0x0F => "Satellite Communication Controller",
            0x10 => "Encryption Controller",
            0x11 => "Signal Processing Controller",
            _ => "Unknown",
        }
    }
}

lazy_static! {
    static ref PCI_DEVICES: Mutex<Vec<PciDevice>> = Mutex::new(Vec::new());
}

pub fn read_config_word(addr: PciAddress, offset: u8) -> u16 {
    let value = read_config_dword(addr, offset);
    ((value >> ((offset & 2) * 8)) & 0xFFFF) as u16
}

pub fn read_config_dword(addr: PciAddress, offset: u8) -> u32 {
    unsafe {
        let mut address_port = Port::<u32>::new(PCI_CONFIG_ADDRESS);
        let mut data_port = Port::<u32>::new(PCI_CONFIG_DATA);
        
        address_port.write(addr.config_address(offset));
        data_port.read()
    }
}

pub fn write_config_dword(addr: PciAddress, offset: u8, value: u32) {
    unsafe {
        let mut address_port = Port::<u32>::new(PCI_CONFIG_ADDRESS);
        let mut data_port = Port::<u32>::new(PCI_CONFIG_DATA);
        
        address_port.write(addr.config_address(offset));
        data_port.write(value);
    }
}

pub fn write_config_word(addr: PciAddress, offset: u8, value: u16) {
    let current = read_config_dword(addr, offset);
    let shift = (offset & 2) * 8;
    let mask = !(0xFFFF << shift);
    let new_value = (current & mask) | ((value as u32) << shift);
    write_config_dword(addr, offset, new_value);
}

fn check_device(bus: u8, device: u8, function: u8) -> Option<PciDevice> {
    let addr = PciAddress::new(bus, device, function);
    
    let vendor_id = read_config_word(addr, 0x00);
    if vendor_id == 0xFFFF {
        return None;
    }
    
    let device_id = read_config_word(addr, 0x02);
    let revision_id = (read_config_dword(addr, 0x08) & 0xFF) as u8;
    let prog_if = ((read_config_dword(addr, 0x08) >> 8) & 0xFF) as u8;
    let subclass = ((read_config_dword(addr, 0x08) >> 16) & 0xFF) as u8;
    let class_code = ((read_config_dword(addr, 0x08) >> 24) & 0xFF) as u8;
    let header_type = ((read_config_dword(addr, 0x0C) >> 16) & 0xFF) as u8;
    let interrupt_line = (read_config_dword(addr, 0x3C) & 0xFF) as u8;
    let interrupt_pin = ((read_config_dword(addr, 0x3C) >> 8) & 0xFF) as u8;
    
    let mut bar = [0u32; 6];
    for i in 0..6 {
        bar[i] = read_config_dword(addr, 0x10 + (i as u8 * 4));
    }
    
    Some(PciDevice {
        address: addr,
        vendor_id,
        device_id,
        class_code,
        subclass,
        prog_if,
        revision_id,
        header_type,
        interrupt_line,
        interrupt_pin,
        bar,
    })
}

pub fn scan_bus() {
    let mut devices = PCI_DEVICES.lock();
    devices.clear();
    
    for bus in 0..=255u8 {
        for device in 0..32u8 {
            if let Some(dev) = check_device(bus, device, 0) {
                let is_multifunction = (dev.header_type & 0x80) != 0;
                
                println!("  [PCI] {:02x}:{:02x}.0 - {} [{:04x}:{:04x}]",
                    bus, device, dev.class_name(), dev.vendor_id, dev.device_id);
                
                devices.push(dev);
                
                if is_multifunction {
                    for function in 1..8u8 {
                        if let Some(func_dev) = check_device(bus, device, function) {
                            println!("  [PCI] {:02x}:{:02x}.{} - {} [{:04x}:{:04x}]",
                                bus, device, function, func_dev.class_name(),
                                func_dev.vendor_id, func_dev.device_id);
                            devices.push(func_dev);
                        }
                    }
                }
            }
        }
    }
    
    println!("  [PCI] Found {} device(s)", devices.len());
}

pub fn get_devices() -> Vec<PciDevice> {
    PCI_DEVICES.lock().clone()
}

pub fn find_device(vendor_id: u16, device_id: u16) -> Option<PciDevice> {
    PCI_DEVICES.lock().iter()
        .find(|d| d.vendor_id == vendor_id && d.device_id == device_id)
        .cloned()
}

pub fn find_devices_by_class(class_code: u8, subclass: u8) -> Vec<PciDevice> {
    PCI_DEVICES.lock().iter()
        .filter(|d| d.class_code == class_code && d.subclass == subclass)
        .cloned()
        .collect()
}

pub fn find_ahci_controllers() -> Vec<PciDevice> {
    PCI_DEVICES.lock().iter()
        .filter(|d| d.is_ahci())
        .cloned()
        .collect()
}

pub fn find_usb_controllers() -> Vec<PciDevice> {
    PCI_DEVICES.lock().iter()
        .filter(|d| d.is_usb())
        .cloned()
        .collect()
}

pub fn enable_bus_mastering(device: &PciDevice) {
    let command = read_config_word(device.address, 0x04);
    write_config_word(device.address, 0x04, command | 0x04);
}

pub fn enable_memory_space(device: &PciDevice) {
    let command = read_config_word(device.address, 0x04);
    write_config_word(device.address, 0x04, command | 0x02);
}

pub fn enable_io_space(device: &PciDevice) {
    let command = read_config_word(device.address, 0x04);
    write_config_word(device.address, 0x04, command | 0x01);
}

pub fn get_bar_address(bar: u32) -> u64 {
    if bar & 0x01 != 0 {
        (bar & 0xFFFFFFFC) as u64
    } else {
        match (bar >> 1) & 0x03 {
            0x00 => (bar & 0xFFFFFFF0) as u64,
            0x02 => (bar & 0xFFFFFFF0) as u64,
            _ => 0,
        }
    }
}

pub fn is_bar_io(bar: u32) -> bool {
    bar & 0x01 != 0
}

pub fn is_bar_memory(bar: u32) -> bool {
    bar & 0x01 == 0
}
