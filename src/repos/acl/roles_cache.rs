//! RolesCache is a module that caches received from db information about user and his roles
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures::Future;
use futures_cpupool::CpuPool;

use repos::user_roles::{UserRolesRepo, UserRolesRepoImpl};
use repos::types::DbPool;
use models::authorization::*;

#[derive(Clone)]
pub struct RolesCacheImpl {
    roles_cache: Arc<Mutex<HashMap<i32, Vec<Role>>>>,
    db_pool: DbPool,
    cpu_pool: CpuPool,
}

impl RolesCacheImpl {
    pub fn new(db_pool: DbPool, cpu_pool: CpuPool) -> Self {
        RolesCacheImpl {
            roles_cache: Arc::new(Mutex::new(HashMap::new())),
            db_pool: db_pool,
            cpu_pool: cpu_pool,
        }
    }
}

pub trait RolesCache {
    fn get(&mut self, id: i32) -> Vec<Role>;
}

impl RolesCache for RolesCacheImpl {
    fn get(&mut self, id: i32) -> Vec<Role> {
        let id_clone = id.clone();
        let mut mutex = self.roles_cache.lock().unwrap();
        let db_pool = self.db_pool.clone();
        let cpu_pool = self.cpu_pool.clone();

        let vec = mutex.entry(id_clone).or_insert_with(|| {
            let repo = UserRolesRepoImpl::new(db_pool, cpu_pool);
            repo.list_for_user(id_clone)
                .wait()
                .map(|users| users.into_iter().map(|u| u.role).collect())
                .unwrap_or_default()
        });
        vec.clone()
    }
}
