//! RolesCache is a module that caches received from db information about user and his roles
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use repos::user_roles::{UserRolesRepo, UserRolesRepoImpl};
use repos::types::{DbConnection, RepoResult};
use models::authorization::*;
use repos::acl::SystemACL;
use repos::error::RepoError as Error;

#[derive(Clone)]
pub struct RolesCacheImpl {
    roles_cache: Arc<Mutex<HashMap<i32, Vec<Role>>>>,
}

impl RolesCacheImpl {
    pub fn new() -> Self {
        RolesCacheImpl {
            roles_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

pub trait RolesCache {
    fn get(&mut self, id: i32, db_conn: Option<&DbConnection>) -> RepoResult<Vec<Role>>;
}

impl RolesCache for RolesCacheImpl {
    fn get(&mut self, id: i32, db_conn: Option<&DbConnection>) -> RepoResult<Vec<Role>> {
        let mut hash_map = self.roles_cache.lock().unwrap();
        match hash_map.entry(id) {
            Entry::Occupied(o) => Ok(o.get().clone()),
            Entry::Vacant(v) => db_conn
                .ok_or(Error::Connection(
                    format_err!("No connection to db")
                ))
                .and_then(|con| {
                    let repo = UserRolesRepoImpl::new(con, Box::new(SystemACL::new()));
                    repo.list_for_user(id)
                        .map(|users| users.into_iter().map(|u| u.role).collect())
                })
                .and_then(move |vec: Vec<Role>| {
                    v.insert(vec.clone());
                    Ok(vec)
                }),
        }
    }
}
