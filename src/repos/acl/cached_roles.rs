use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures::Future;

use repos::user_roles::UserRolesRepo;
use models::authorization::*;

#[derive(Clone)]
pub struct CachedRoles<U: UserRolesRepo + 'static + Clone> {
    roles_cache: Arc<Mutex<HashMap<i32, Vec<Role>>>>,
    users_role_repo: U,
}

impl<U: UserRolesRepo + 'static + Clone> CachedRoles<U> {
    pub fn new(repo: U) -> Self {
        Self {
            roles_cache: Arc::new(Mutex::new(HashMap::new())),
            users_role_repo: repo,
        }
    }

    pub fn get(&mut self, id: i32) -> Vec<Role> {
        let id_clone = id.clone();
        let repo = self.users_role_repo.clone();
        let mut mutex = self.roles_cache.lock().unwrap();
        let vec = mutex.entry(id_clone).or_insert_with(|| {
            repo.list_for_user(id_clone)
                .wait()
                .map(|users| users.into_iter().map(|u| u.role).collect())
                .unwrap_or_default()
        });
        vec.clone()
    }
}
