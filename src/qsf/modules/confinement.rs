use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;

pub struct ConfinementModule {
    process_confinements: BTreeMap<u32, ProcessConfinement>,
    enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ProcessConfinement {
    pub pid: u32,
    pub allowed_paths: Vec<String>,
    pub denied_paths: Vec<String>,
    pub network_allowed: bool,
    pub allowed_ports: Vec<u16>,
    pub allowed_addresses: Vec<String>,
    pub can_fork: bool,
    pub can_exec: bool,
    pub max_memory: Option<usize>,
    pub max_files: Option<usize>,
}

impl ProcessConfinement {
    pub fn new(pid: u32) -> Self {
        ProcessConfinement {
            pid,
            allowed_paths: Vec::new(),
            denied_paths: Vec::new(),
            network_allowed: true,
            allowed_ports: Vec::new(),
            allowed_addresses: Vec::new(),
            can_fork: true,
            can_exec: true,
            max_memory: None,
            max_files: None,
        }
    }
    
    pub fn sandbox() -> Self {
        ProcessConfinement {
            pid: 0,
            allowed_paths: vec![
                String::from("/tmp"),
                String::from("/dev/null"),
                String::from("/dev/zero"),
                String::from("/dev/urandom"),
            ],
            denied_paths: vec![
                String::from("/etc/passwd"),
                String::from("/etc/shadow"),
                String::from("/root"),
            ],
            network_allowed: false,
            allowed_ports: Vec::new(),
            allowed_addresses: Vec::new(),
            can_fork: false,
            can_exec: false,
            max_memory: Some(64 * 1024 * 1024),
            max_files: Some(16),
        }
    }
    
    pub fn allow_path(&mut self, path: &str) {
        self.allowed_paths.push(String::from(path));
    }
    
    pub fn deny_path(&mut self, path: &str) {
        self.denied_paths.push(String::from(path));
    }
    
    pub fn allow_network(&mut self) {
        self.network_allowed = true;
    }
    
    pub fn deny_network(&mut self) {
        self.network_allowed = false;
    }
    
    pub fn allow_port(&mut self, port: u16) {
        self.allowed_ports.push(port);
    }
    
    pub fn allow_address(&mut self, addr: &str) {
        self.allowed_addresses.push(String::from(addr));
    }
}

impl ConfinementModule {
    pub fn new() -> Self {
        ConfinementModule {
            process_confinements: BTreeMap::new(),
            enabled: true,
        }
    }
    
    pub fn confine(&mut self, pid: u32, paths: Vec<String>, network: bool) {
        let mut confinement = ProcessConfinement::new(pid);
        confinement.allowed_paths = paths;
        confinement.network_allowed = network;
        self.process_confinements.insert(pid, confinement);
    }
    
    pub fn confine_with(&mut self, pid: u32, confinement: ProcessConfinement) {
        self.process_confinements.insert(pid, confinement);
    }
    
    pub fn unconfine(&mut self, pid: u32) {
        self.process_confinements.remove(&pid);
    }
    
    pub fn is_confined(&self, pid: u32) -> bool {
        self.process_confinements.contains_key(&pid)
    }
    
    pub fn check_path_access(&self, pid: u32, path: &str) -> bool {
        if !self.enabled {
            return true;
        }
        
        let confinement = match self.process_confinements.get(&pid) {
            Some(c) => c,
            None => return true,
        };
        
        for denied in &confinement.denied_paths {
            if path.starts_with(denied) {
                return false;
            }
        }
        
        if confinement.allowed_paths.is_empty() {
            return true;
        }
        
        for allowed in &confinement.allowed_paths {
            if path.starts_with(allowed) {
                return true;
            }
        }
        
        false
    }
    
    pub fn check_network_access(&self, pid: u32, addr: &str, port: u16) -> bool {
        if !self.enabled {
            return true;
        }
        
        let confinement = match self.process_confinements.get(&pid) {
            Some(c) => c,
            None => return true,
        };
        
        if !confinement.network_allowed {
            return false;
        }
        
        if !confinement.allowed_ports.is_empty() && !confinement.allowed_ports.contains(&port) {
            return false;
        }
        
        if !confinement.allowed_addresses.is_empty() {
            let mut addr_allowed = false;
            for allowed_addr in &confinement.allowed_addresses {
                if addr.starts_with(allowed_addr) {
                    addr_allowed = true;
                    break;
                }
            }
            if !addr_allowed {
                return false;
            }
        }
        
        true
    }
    
    pub fn can_fork(&self, pid: u32) -> bool {
        if !self.enabled {
            return true;
        }
        
        self.process_confinements
            .get(&pid)
            .map_or(true, |c| c.can_fork)
    }
    
    pub fn can_exec(&self, pid: u32) -> bool {
        if !self.enabled {
            return true;
        }
        
        self.process_confinements
            .get(&pid)
            .map_or(true, |c| c.can_exec)
    }
    
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxProfile {
    Unrestricted,
    Network,
    Filesystem,
    Strict,
    Custom,
}

impl SandboxProfile {
    pub fn to_confinement(self, pid: u32) -> ProcessConfinement {
        match self {
            SandboxProfile::Unrestricted => ProcessConfinement::new(pid),
            SandboxProfile::Network => {
                let mut c = ProcessConfinement::new(pid);
                c.network_allowed = false;
                c
            }
            SandboxProfile::Filesystem => {
                let mut c = ProcessConfinement::new(pid);
                c.allowed_paths = vec![String::from("/tmp")];
                c
            }
            SandboxProfile::Strict => {
                let mut c = ProcessConfinement::sandbox();
                c.pid = pid;
                c
            }
            SandboxProfile::Custom => ProcessConfinement::new(pid),
        }
    }
}
