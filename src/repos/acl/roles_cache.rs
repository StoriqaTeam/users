//! RolesCache is a module that caches received from db information about user and his roles
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use stq_types::{UserId, UsersRole};

#[derive(Default, Clone)]
pub struct RolesCacheImpl {
    roles_cache: Arc<Mutex<HashMap<UserId, Vec<UsersRole>>>>,
}

impl RolesCacheImpl {
    pub fn get(&self, _user_id: UserId) -> Vec<UsersRole> {
        //let mut hash_map = self.roles_cache.lock().unwrap();
        //match hash_map.entry(user_id) {
        //    Entry::Occupied(o) => o.get().clone(),
        //    Entry::Vacant(_) => vec![],
        //}
        vec![]
    }

    pub fn clear(&self) {
        //let mut hash_map = self.roles_cache.lock().unwrap();
        //hash_map.clear();
    }

    pub fn remove(&self, _id: UserId) {
        //let mut hash_map = self.roles_cache.lock().unwrap();
        //hash_map.remove(&id);
    }

    pub fn contains(&self, _id: UserId) -> bool {
        //let hash_map = self.roles_cache.lock().unwrap();
        //hash_map.contains_key(&id)
        false
    }

    pub fn add_roles(&self, _id: UserId, _roles: &[UsersRole]) {
        //let mut hash_map = self.roles_cache.lock().unwrap();
        //hash_map.insert(id, roles.to_vec());
    }
}
