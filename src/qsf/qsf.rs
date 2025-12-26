use spin::Mutex;
use lazy_static::lazy_static;
use alloc::vec::Vec;
use alloc::string::String;
use super::modules::{IntegrityModule, CapabilityModule, ConfinementModule};
use super::policies::SecurityPolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityLevel {
    Disabled,
    Permissive,
    Enforcing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessDecision {
    Allow,
    Deny,
    Audit,
}

lazy_static! {
    pub static ref QSF: Mutex<QunixSecurityFramework> = Mutex::new(QunixSecurityFramework::new());
}

pub struct QunixSecurityFramework {
    level: SecurityLevel,
    integrity: IntegrityModule,
    capability: CapabilityModule,
    confinement: ConfinementModule,
    policies: Vec<SecurityPolicy>,
    audit_log: Vec<AuditEntry>,
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub pid: u32,
    pub uid: u32,
    pub action: String,
    pub resource: String,
    pub decision: AccessDecision,
    pub reason: String,
}

impl QunixSecurityFramework {
    pub fn new() -> Self {
        QunixSecurityFramework {
            level: SecurityLevel::Permissive,
            integrity: IntegrityModule::new(),
            capability: CapabilityModule::new(),
            confinement: ConfinementModule::new(),
            policies: Vec::new(),
            audit_log: Vec::new(),
        }
    }
    
    pub fn set_level(&mut self, level: SecurityLevel) {
        self.level = level;
    }
    
    pub fn get_level(&self) -> SecurityLevel {
        self.level
    }
    
    pub fn is_enforcing(&self) -> bool {
        self.level == SecurityLevel::Enforcing
    }
    
    pub fn check_file_access(&mut self, pid: u32, uid: u32, path: &str, mode: u32) -> AccessDecision {
        if self.level == SecurityLevel::Disabled {
            return AccessDecision::Allow;
        }
        
        if !self.integrity.verify_path(path) {
            self.audit(pid, uid, "file_access", path, AccessDecision::Deny, "integrity violation");
            return if self.is_enforcing() { AccessDecision::Deny } else { AccessDecision::Audit };
        }
        
        if !self.capability.check_file_capability(uid, path, mode) {
            self.audit(pid, uid, "file_access", path, AccessDecision::Deny, "capability denied");
            return if self.is_enforcing() { AccessDecision::Deny } else { AccessDecision::Audit };
        }
        
        if !self.confinement.check_path_access(pid, path) {
            self.audit(pid, uid, "file_access", path, AccessDecision::Deny, "confinement violation");
            return if self.is_enforcing() { AccessDecision::Deny } else { AccessDecision::Audit };
        }
        
        AccessDecision::Allow
    }
    
    pub fn check_process_exec(&mut self, pid: u32, uid: u32, path: &str) -> AccessDecision {
        if self.level == SecurityLevel::Disabled {
            return AccessDecision::Allow;
        }
        
        if !self.integrity.verify_executable(path) {
            self.audit(pid, uid, "exec", path, AccessDecision::Deny, "executable integrity failed");
            return if self.is_enforcing() { AccessDecision::Deny } else { AccessDecision::Audit };
        }
        
        if !self.capability.check_exec_capability(uid) {
            self.audit(pid, uid, "exec", path, AccessDecision::Deny, "exec capability denied");
            return if self.is_enforcing() { AccessDecision::Deny } else { AccessDecision::Audit };
        }
        
        AccessDecision::Allow
    }
    
    pub fn check_network_access(&mut self, pid: u32, uid: u32, addr: &str, port: u16) -> AccessDecision {
        if self.level == SecurityLevel::Disabled {
            return AccessDecision::Allow;
        }
        
        let resource = alloc::format!("{}:{}", addr, port);
        
        if !self.confinement.check_network_access(pid, addr, port) {
            self.audit(pid, uid, "network", &resource, AccessDecision::Deny, "network confinement");
            return if self.is_enforcing() { AccessDecision::Deny } else { AccessDecision::Audit };
        }
        
        AccessDecision::Allow
    }
    
    pub fn check_capability(&self, uid: u32, cap: Capability) -> bool {
        if self.level == SecurityLevel::Disabled {
            return true;
        }
        
        self.capability.has_capability(uid, cap)
    }
    
    fn audit(&mut self, pid: u32, uid: u32, action: &str, resource: &str, decision: AccessDecision, reason: &str) {
        let entry = AuditEntry {
            timestamp: crate::hal::drivers::pit::get_ticks(),
            pid,
            uid,
            action: String::from(action),
            resource: String::from(resource),
            decision,
            reason: String::from(reason),
        };
        
        self.audit_log.push(entry);
        
        if self.audit_log.len() > 10000 {
            self.audit_log.remove(0);
        }
    }
    
    pub fn get_audit_log(&self) -> &[AuditEntry] {
        &self.audit_log
    }
    
    pub fn clear_audit_log(&mut self) {
        self.audit_log.clear();
    }
    
    pub fn load_policy(&mut self, policy: SecurityPolicy) {
        self.policies.push(policy);
    }
    
    pub fn add_integrity_hash(&mut self, path: &str, hash: [u8; 32]) {
        self.integrity.add_hash(path, hash);
    }
    
    pub fn grant_capability(&mut self, uid: u32, cap: Capability) {
        self.capability.grant(uid, cap);
    }
    
    pub fn revoke_capability(&mut self, uid: u32, cap: Capability) {
        self.capability.revoke(uid, cap);
    }
    
    pub fn confine_process(&mut self, pid: u32, paths: Vec<String>, network: bool) {
        self.confinement.confine(pid, paths, network);
    }
    
    pub fn unconfine_process(&mut self, pid: u32) {
        self.confinement.unconfine(pid);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum Capability {
    CapChown,
    CapDacOverride,
    CapDacReadSearch,
    CapFowner,
    CapFsetid,
    CapKill,
    CapSetgid,
    CapSetuid,
    CapSetpcap,
    CapLinuxImmutable,
    CapNetBindService,
    CapNetBroadcast,
    CapNetAdmin,
    CapNetRaw,
    CapIpcLock,
    CapIpcOwner,
    CapSysModule,
    CapSysRawio,
    CapSysChroot,
    CapSysPtrace,
    CapSysPacct,
    CapSysAdmin,
    CapSysBoot,
    CapSysNice,
    CapSysResource,
    CapSysTime,
    CapSysTtyConfig,
    CapMknod,
    CapLease,
    CapAuditWrite,
    CapAuditControl,
    CapSetfcap,
    CapMacOverride,
    CapMacAdmin,
    CapSyslog,
    CapWakeAlarm,
    CapBlockSuspend,
    CapAuditRead,
}

pub fn init_qsf() {
    let mut qsf = QSF.lock();
    
    qsf.set_level(SecurityLevel::Permissive);
    
    qsf.grant_capability(0, Capability::CapSysAdmin);
    qsf.grant_capability(0, Capability::CapDacOverride);
    qsf.grant_capability(0, Capability::CapKill);
    qsf.grant_capability(0, Capability::CapSetuid);
    qsf.grant_capability(0, Capability::CapSetgid);
    qsf.grant_capability(0, Capability::CapSysBoot);
    qsf.grant_capability(0, Capability::CapNetAdmin);
    qsf.grant_capability(0, Capability::CapNetBindService);
}

pub fn check_access(pid: u32, uid: u32, path: &str, mode: u32) -> AccessDecision {
    QSF.lock().check_file_access(pid, uid, path, mode)
}

pub fn check_exec(pid: u32, uid: u32, path: &str) -> AccessDecision {
    QSF.lock().check_process_exec(pid, uid, path)
}

pub fn has_capability(uid: u32, cap: Capability) -> bool {
    QSF.lock().check_capability(uid, cap)
}
