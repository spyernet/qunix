use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::hal::drivers::pci::{PciDevice, find_usb_controllers, enable_bus_mastering, enable_memory_space, get_bar_address};
use crate::println;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbControllerType {
    Uhci,
    Ohci,
    Ehci,
    Xhci,
    Unknown,
}

impl UsbControllerType {
    fn from_prog_if(prog_if: u8) -> Self {
        match prog_if {
            0x00 => UsbControllerType::Uhci,
            0x10 => UsbControllerType::Ohci,
            0x20 => UsbControllerType::Ehci,
            0x30 => UsbControllerType::Xhci,
            _ => UsbControllerType::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UsbController {
    pub pci_device: PciDevice,
    pub controller_type: UsbControllerType,
    pub base_addr: u64,
    pub io_base: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbSpeed {
    Low,
    Full,
    High,
    Super,
    SuperPlus,
}

#[derive(Debug, Clone)]
pub struct UsbDevice {
    pub address: u8,
    pub speed: UsbSpeed,
    pub vendor_id: u16,
    pub product_id: u16,
    pub class: u8,
    pub subclass: u8,
    pub protocol: u8,
    pub manufacturer: Option<alloc::string::String>,
    pub product: Option<alloc::string::String>,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct UsbDeviceDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub usb_version: u16,
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub max_packet_size: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_version: u16,
    pub manufacturer_index: u8,
    pub product_index: u8,
    pub serial_index: u8,
    pub num_configurations: u8,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct UsbConfigDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub total_length: u16,
    pub num_interfaces: u8,
    pub config_value: u8,
    pub config_index: u8,
    pub attributes: u8,
    pub max_power: u8,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct UsbInterfaceDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub num_endpoints: u8,
    pub interface_class: u8,
    pub interface_subclass: u8,
    pub interface_protocol: u8,
    pub interface_index: u8,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct UsbEndpointDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u16,
    pub interval: u8,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct UsbSetupPacket {
    pub request_type: u8,
    pub request: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

impl UsbSetupPacket {
    pub fn get_descriptor(descriptor_type: u8, descriptor_index: u8, length: u16) -> Self {
        UsbSetupPacket {
            request_type: 0x80,
            request: 0x06,
            value: ((descriptor_type as u16) << 8) | (descriptor_index as u16),
            index: 0,
            length,
        }
    }
    
    pub fn set_address(address: u8) -> Self {
        UsbSetupPacket {
            request_type: 0x00,
            request: 0x05,
            value: address as u16,
            index: 0,
            length: 0,
        }
    }
    
    pub fn set_configuration(config: u8) -> Self {
        UsbSetupPacket {
            request_type: 0x00,
            request: 0x09,
            value: config as u16,
            index: 0,
            length: 0,
        }
    }
}

pub const USB_DESCRIPTOR_DEVICE: u8 = 1;
pub const USB_DESCRIPTOR_CONFIG: u8 = 2;
pub const USB_DESCRIPTOR_STRING: u8 = 3;
pub const USB_DESCRIPTOR_INTERFACE: u8 = 4;
pub const USB_DESCRIPTOR_ENDPOINT: u8 = 5;

pub const USB_CLASS_HID: u8 = 0x03;
pub const USB_CLASS_MASS_STORAGE: u8 = 0x08;
pub const USB_CLASS_HUB: u8 = 0x09;

lazy_static! {
    static ref USB_CONTROLLERS: Mutex<Vec<UsbController>> = Mutex::new(Vec::new());
    static ref USB_DEVICES: Mutex<Vec<UsbDevice>> = Mutex::new(Vec::new());
}

pub fn init() {
    let controllers = find_usb_controllers();
    
    for pci_dev in controllers {
        let controller_type = UsbControllerType::from_prog_if(pci_dev.prog_if);
        
        enable_bus_mastering(&pci_dev);
        enable_memory_space(&pci_dev);
        
        let base_addr = get_bar_address(pci_dev.bar[0]);
        let io_base = (pci_dev.bar[4] & 0xFFFC) as u16;
        
        println!("  [USB] {:?} controller at {:08x}", controller_type, base_addr);
        
        let controller = UsbController {
            pci_device: pci_dev,
            controller_type,
            base_addr,
            io_base,
        };
        
        USB_CONTROLLERS.lock().push(controller);
    }
}

pub fn get_controllers() -> Vec<UsbController> {
    USB_CONTROLLERS.lock().clone()
}

pub fn get_devices() -> Vec<UsbDevice> {
    USB_DEVICES.lock().clone()
}

pub fn reset_port(_controller: &UsbController, _port: u8) -> Result<(), &'static str> {
    Ok(())
}

pub fn enumerate_devices(_controller: &UsbController) -> Result<Vec<UsbDevice>, &'static str> {
    Ok(Vec::new())
}

pub trait UsbDriver: Send + Sync {
    fn name(&self) -> &'static str;
    fn probe(&self, device: &UsbDevice) -> bool;
    fn attach(&mut self, device: &UsbDevice) -> Result<(), &'static str>;
    fn detach(&mut self) -> Result<(), &'static str>;
}

lazy_static! {
    static ref USB_DRIVERS: Mutex<Vec<&'static dyn UsbDriver>> = Mutex::new(Vec::new());
}

pub fn register_driver(driver: &'static dyn UsbDriver) {
    USB_DRIVERS.lock().push(driver);
}
