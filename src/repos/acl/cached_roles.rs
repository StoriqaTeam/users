use std::collections::HashMap;

use futures::Future;

use repos::user_roles::UserRolesRepo;
use models::authorization::*;


pub struct CachedRoles<U: UserRolesRepo + 'static + Clone> {
    roles_cache: HashMap<i32, Vec<Role>>,
    users_role_repo: U,
}

impl<U: UserRolesRepo + 'static + Clone> CachedRoles<U> {
    pub fn new (repo: U) -> Self {
        Self {
            roles_cache: HashMap::new(),
            users_role_repo: repo
        }
    }

    pub fn get(&mut self, id: i32) -> &mut Vec<Role> {
        let id_clone = id.clone();
        let repo = self.users_role_repo.clone();
        self.roles_cache.entry(id_clone)
            .or_insert_with(|| {
                repo
                    .list_for_user(id_clone)
                    .wait()
                    .map(|users| users.into_iter().map(|u| u.role).collect())
                    .unwrap_or_default()
            })
    }
}
