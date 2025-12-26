use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

pub struct IntegrityModule {
    file_hashes: BTreeMap<String, [u8; 32]>,
    verified_executables: BTreeMap<String, bool>,
    enabled: bool,
}

impl IntegrityModule {
    pub fn new() -> Self {
        IntegrityModule {
            file_hashes: BTreeMap::new(),
            verified_executables: BTreeMap::new(),
            enabled: true,
        }
    }
    
    pub fn add_hash(&mut self, path: &str, hash: [u8; 32]) {
        self.file_hashes.insert(String::from(path), hash);
    }
    
    pub fn remove_hash(&mut self, path: &str) {
        self.file_hashes.remove(path);
    }
    
    pub fn verify_path(&self, path: &str) -> bool {
        if !self.enabled {
            return true;
        }
        
        if !self.file_hashes.contains_key(path) {
            return true;
        }
        
        true
    }
    
    pub fn verify_executable(&self, path: &str) -> bool {
        if !self.enabled {
            return true;
        }
        
        if let Some(&verified) = self.verified_executables.get(path) {
            return verified;
        }
        
        true
    }
    
    pub fn set_executable_verified(&mut self, path: &str, verified: bool) {
        self.verified_executables.insert(String::from(path), verified);
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
    
    pub fn compute_hash(data: &[u8]) -> [u8; 32] {
        let mut hash = [0u8; 32];
        
        let mut state = [
            0x6a09e667u32, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
            0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
        ];
        
        for chunk in data.chunks(64) {
            for (i, byte) in chunk.iter().enumerate() {
                state[i % 8] = state[i % 8].wrapping_add(*byte as u32);
            }
        }
        
        for (i, &s) in state.iter().enumerate() {
            hash[i * 4..(i + 1) * 4].copy_from_slice(&s.to_le_bytes());
        }
        
        hash
    }
    
    pub fn verify_hash(data: &[u8], expected: &[u8; 32]) -> bool {
        let computed = Self::compute_hash(data);
        computed == *expected
    }
}

#[derive(Debug, Clone)]
pub struct IntegrityPolicy {
    pub path_pattern: String,
    pub required: bool,
    pub action_on_failure: IntegrityAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityAction {
    Allow,
    Deny,
    Audit,
    Quarantine,
}

impl Default for IntegrityPolicy {
    fn default() -> Self {
        IntegrityPolicy {
            path_pattern: String::from("*"),
            required: false,
            action_on_failure: IntegrityAction::Audit,
        }
    }
}
