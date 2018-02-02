use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures::Future;
use futures_cpupool::CpuPool;

use repos::user_roles::{UserRolesRepoImpl, UserRolesRepo};
use repos::types::DbPool;
use models::authorization::*;

#[derive(Clone)]
pub struct CachedRoles {
    roles_cache: Arc<Mutex<HashMap<i32, Vec<Role>>>>,
}

impl CachedRoles {
    pub fn new() -> Self {
        Self {
            roles_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&mut self, id: i32, r2d2_pool: DbPool, cpu_pool: CpuPool) -> Vec<Role> {
        let id_clone = id.clone();
        let mut mutex = self.roles_cache.lock().unwrap();
        let vec = mutex.entry(id_clone).or_insert_with(|| {
            let repo = UserRolesRepoImpl::new(r2d2_pool, cpu_pool);
            repo.list_for_user(id_clone)
                .wait()
                .map(|users| users.into_iter().map(|u| u.role).collect())
                .unwrap_or_default()
        });
        vec.clone()
    }
}
