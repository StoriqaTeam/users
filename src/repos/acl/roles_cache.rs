//! RolesCache is a module that caches received from db information about user and his roles
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use repos::user_roles::{UserRolesRepo, UserRolesRepoImpl};
use repos::types::{DbConnection, RepoResult};
use models::authorization::*;
use stq_acl::{RolesCache, SystemACL};
use repos::error::RepoError as Error;

#[derive(Clone, Default)]
pub struct RolesCacheImpl {
    roles_cache: Arc<Mutex<HashMap<i32, Vec<Role>>>>,
}

impl RolesCache for RolesCacheImpl {
    type Role = Role;
    type Error = Error;

    fn get(&self, id: i32, db_conn: Option<&DbConnection>) -> Result<Vec<Self::Role>, Self::Error> {
        let mut hash_map = self.roles_cache.lock().unwrap();
        match hash_map.entry(id) {
            Entry::Occupied(o) => Ok(o.get().clone()),
            Entry::Vacant(v) => db_conn
                .ok_or(Error::Connection(format_err!("No connection to db")))
                .and_then(|con| {
                    let repo = UserRolesRepoImpl::new(con, Box::new(SystemACL::default()));
                    repo.list_for_user(id)
                        .map(|users| users.into_iter().map(|u| u.role).collect())
                })
                .and_then(move |vec: Vec<Role>| {
                    v.insert(vec.clone());
                    Ok(vec)
                }),
        }
    }

    fn clear(&self) -> Result<(), Self::Error> {
        let mut hash_map = self.roles_cache.lock().unwrap();
        hash_map.clear();
        Ok(())
    }

    fn remove(&self, id: i32) -> Result<(), Self::Error> {
        let mut hash_map = self.roles_cache.lock().unwrap();
        hash_map.remove(&id);
        Ok(())
    }
}
