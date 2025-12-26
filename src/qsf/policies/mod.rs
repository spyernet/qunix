use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub name: String,
    pub version: String,
    pub rules: Vec<PolicyRule>,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct PolicyRule {
    pub subject: Subject,
    pub object: Object,
    pub permissions: Permissions,
    pub action: PolicyAction,
}

#[derive(Debug, Clone)]
pub enum Subject {
    User(u32),
    Group(u32),
    Process(u32),
    Role(String),
    Any,
}

#[derive(Debug, Clone)]
pub enum Object {
    File(String),
    Directory(String),
    Process(u32),
    Network(String, u16),
    Capability(String),
    Any,
}

#[derive(Debug, Clone, Copy)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub append: bool,
    pub create: bool,
    pub delete: bool,
    pub setattr: bool,
}

impl Default for Permissions {
    fn default() -> Self {
        Permissions {
            read: false,
            write: false,
            execute: false,
            append: false,
            create: false,
            delete: false,
            setattr: false,
        }
    }
}

impl Permissions {
    pub fn all() -> Self {
        Permissions {
            read: true,
            write: true,
            execute: true,
            append: true,
            create: true,
            delete: true,
            setattr: true,
        }
    }
    
    pub fn read_only() -> Self {
        Permissions {
            read: true,
            ..Default::default()
        }
    }
    
    pub fn read_write() -> Self {
        Permissions {
            read: true,
            write: true,
            ..Default::default()
        }
    }
    
    pub fn read_execute() -> Self {
        Permissions {
            read: true,
            execute: true,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyAction {
    Allow,
    Deny,
    Audit,
    AuditAllow,
    AuditDeny,
}

impl SecurityPolicy {
    pub fn new(name: &str, version: &str) -> Self {
        SecurityPolicy {
            name: String::from(name),
            version: String::from(version),
            rules: Vec::new(),
            enabled: true,
        }
    }
    
    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
    }
    
    pub fn remove_rule(&mut self, index: usize) -> Option<PolicyRule> {
        if index < self.rules.len() {
            Some(self.rules.remove(index))
        } else {
            None
        }
    }
    
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    pub fn check(&self, subject: &Subject, object: &Object, permission: &str) -> PolicyAction {
        if !self.enabled {
            return PolicyAction::Allow;
        }
        
        for rule in &self.rules {
            if matches_subject(&rule.subject, subject) && matches_object(&rule.object, object) {
                let has_permission = match permission {
                    "read" => rule.permissions.read,
                    "write" => rule.permissions.write,
                    "execute" => rule.permissions.execute,
                    "append" => rule.permissions.append,
                    "create" => rule.permissions.create,
                    "delete" => rule.permissions.delete,
                    "setattr" => rule.permissions.setattr,
                    _ => false,
                };
                
                if has_permission {
                    return rule.action;
                }
            }
        }
        
        PolicyAction::Deny
    }
}

fn matches_subject(pattern: &Subject, subject: &Subject) -> bool {
    match (pattern, subject) {
        (Subject::Any, _) => true,
        (Subject::User(p), Subject::User(s)) => p == s,
        (Subject::Group(p), Subject::Group(s)) => p == s,
        (Subject::Process(p), Subject::Process(s)) => p == s,
        (Subject::Role(p), Subject::Role(s)) => p == s,
        _ => false,
    }
}

fn matches_object(pattern: &Object, object: &Object) -> bool {
    match (pattern, object) {
        (Object::Any, _) => true,
        (Object::File(p), Object::File(o)) => o.starts_with(p) || p == "*",
        (Object::Directory(p), Object::Directory(o)) => o.starts_with(p) || p == "*",
        (Object::Process(p), Object::Process(o)) => p == o,
        (Object::Network(pa, pp), Object::Network(oa, op)) => {
            (pa == "*" || pa == oa) && (pp == &0 || pp == op)
        }
        (Object::Capability(p), Object::Capability(o)) => p == o || p == "*",
        _ => false,
    }
}

pub fn default_policy() -> SecurityPolicy {
    let mut policy = SecurityPolicy::new("default", "1.0");
    
    policy.add_rule(PolicyRule {
        subject: Subject::User(0),
        object: Object::Any,
        permissions: Permissions::all(),
        action: PolicyAction::Allow,
    });
    
    policy.add_rule(PolicyRule {
        subject: Subject::Any,
        object: Object::File(String::from("/etc/shadow")),
        permissions: Permissions::default(),
        action: PolicyAction::Deny,
    });
    
    policy.add_rule(PolicyRule {
        subject: Subject::Any,
        object: Object::Directory(String::from("/tmp")),
        permissions: Permissions::all(),
        action: PolicyAction::Allow,
    });
    
    policy
}
