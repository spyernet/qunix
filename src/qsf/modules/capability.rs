use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use crate::qsf::qsf::Capability;

pub struct CapabilityModule {
    user_capabilities: BTreeMap<u32, BTreeSet<Capability>>,
    process_capabilities: BTreeMap<u32, BTreeSet<Capability>>,
    enabled: bool,
}

impl CapabilityModule {
    pub fn new() -> Self {
        let mut module = CapabilityModule {
            user_capabilities: BTreeMap::new(),
            process_capabilities: BTreeMap::new(),
            enabled: true,
        };
        
        let mut root_caps = BTreeSet::new();
        root_caps.insert(Capability::CapSysAdmin);
        root_caps.insert(Capability::CapDacOverride);
        root_caps.insert(Capability::CapDacReadSearch);
        root_caps.insert(Capability::CapFowner);
        root_caps.insert(Capability::CapKill);
        root_caps.insert(Capability::CapSetuid);
        root_caps.insert(Capability::CapSetgid);
        root_caps.insert(Capability::CapNetAdmin);
        root_caps.insert(Capability::CapNetBindService);
        root_caps.insert(Capability::CapNetRaw);
        root_caps.insert(Capability::CapSysBoot);
        root_caps.insert(Capability::CapSysChroot);
        root_caps.insert(Capability::CapSysModule);
        root_caps.insert(Capability::CapSysRawio);
        root_caps.insert(Capability::CapMknod);
        module.user_capabilities.insert(0, root_caps);
        
        module
    }
    
    pub fn grant(&mut self, uid: u32, cap: Capability) {
        self.user_capabilities
            .entry(uid)
            .or_insert_with(BTreeSet::new)
            .insert(cap);
    }
    
    pub fn revoke(&mut self, uid: u32, cap: Capability) {
        if let Some(caps) = self.user_capabilities.get_mut(&uid) {
            caps.remove(&cap);
        }
    }
    
    pub fn has_capability(&self, uid: u32, cap: Capability) -> bool {
        if !self.enabled {
            return true;
        }
        
        if uid == 0 {
            return true;
        }
        
        self.user_capabilities
            .get(&uid)
            .map_or(false, |caps| caps.contains(&cap))
    }
    
    pub fn get_capabilities(&self, uid: u32) -> Vec<Capability> {
        self.user_capabilities
            .get(&uid)
            .map_or(Vec::new(), |caps| caps.iter().copied().collect())
    }
    
    pub fn check_file_capability(&self, uid: u32, _path: &str, mode: u32) -> bool {
        if !self.enabled || uid == 0 {
            return true;
        }
        
        if mode & 0o200 != 0 {
            if !self.has_capability(uid, Capability::CapDacOverride) {
                return false;
            }
        }
        
        true
    }
    
    pub fn check_exec_capability(&self, uid: u32) -> bool {
        if !self.enabled || uid == 0 {
            return true;
        }
        
        true
    }
    
    pub fn grant_process_capability(&mut self, pid: u32, cap: Capability) {
        self.process_capabilities
            .entry(pid)
            .or_insert_with(BTreeSet::new)
            .insert(cap);
    }
    
    pub fn revoke_process_capability(&mut self, pid: u32, cap: Capability) {
        if let Some(caps) = self.process_capabilities.get_mut(&pid) {
            caps.remove(&cap);
        }
    }
    
    pub fn process_has_capability(&self, pid: u32, cap: Capability) -> bool {
        self.process_capabilities
            .get(&pid)
            .map_or(false, |caps| caps.contains(&cap))
    }
    
    pub fn drop_all_capabilities(&mut self, uid: u32) {
        self.user_capabilities.remove(&uid);
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

pub const CAP_CHOWN: u32 = 0;
pub const CAP_DAC_OVERRIDE: u32 = 1;
pub const CAP_DAC_READ_SEARCH: u32 = 2;
pub const CAP_FOWNER: u32 = 3;
pub const CAP_FSETID: u32 = 4;
pub const CAP_KILL: u32 = 5;
pub const CAP_SETGID: u32 = 6;
pub const CAP_SETUID: u32 = 7;
pub const CAP_SETPCAP: u32 = 8;
pub const CAP_LINUX_IMMUTABLE: u32 = 9;
pub const CAP_NET_BIND_SERVICE: u32 = 10;
pub const CAP_NET_BROADCAST: u32 = 11;
pub const CAP_NET_ADMIN: u32 = 12;
pub const CAP_NET_RAW: u32 = 13;
pub const CAP_IPC_LOCK: u32 = 14;
pub const CAP_IPC_OWNER: u32 = 15;
pub const CAP_SYS_MODULE: u32 = 16;
pub const CAP_SYS_RAWIO: u32 = 17;
pub const CAP_SYS_CHROOT: u32 = 18;
pub const CAP_SYS_PTRACE: u32 = 19;
pub const CAP_SYS_PACCT: u32 = 20;
pub const CAP_SYS_ADMIN: u32 = 21;
pub const CAP_SYS_BOOT: u32 = 22;
pub const CAP_SYS_NICE: u32 = 23;
pub const CAP_SYS_RESOURCE: u32 = 24;
pub const CAP_SYS_TIME: u32 = 25;
pub const CAP_SYS_TTY_CONFIG: u32 = 26;
pub const CAP_MKNOD: u32 = 27;
pub const CAP_LEASE: u32 = 28;
pub const CAP_AUDIT_WRITE: u32 = 29;
pub const CAP_AUDIT_CONTROL: u32 = 30;
pub const CAP_SETFCAP: u32 = 31;
