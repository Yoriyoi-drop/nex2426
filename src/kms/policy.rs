/// Access Control Policy for Keys.
/// In a 1TB Enterprise environment, different users/processes might have different permissions.

#[derive(Debug, PartialEq)]
pub enum Permission {
    ReadOnly,
    ReadWrite,
    Admin,
}

pub struct KeyPolicy {
    pub owner_id: String,
    pub permissions: Vec<(String, Permission)>, // User -> Permission
    pub expiration: u64, // Unix Timestamp
}

impl KeyPolicy {
    pub fn new(owner: &str) -> Self {
        Self {
            owner_id: owner.to_string(),
            permissions: Vec::new(),
            expiration: 0, // No expiration
        }
    }
    
    pub fn grant(&mut self, user: &str, perm: Permission) {
        self.permissions.push((user.to_string(), perm));
    }
    
    pub fn check_access(&self, user: &str, required: Permission) -> bool {
        if user == self.owner_id { return true; }
        
        for (u, p) in &self.permissions {
            if u == user {
                if *p == Permission::Admin { return true; }
                if *p == required { return true; }
                if *p == Permission::ReadWrite && required == Permission::ReadOnly { return true; }
            }
        }
        false
    }
}
