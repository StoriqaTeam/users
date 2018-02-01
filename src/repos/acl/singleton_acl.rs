use std::sync::{Arc, Mutex};

use futures_cpupool::CpuPool;

use repos::types::DbPool;
use repos::user_roles::UserRolesRepoImpl;
use models::authorization::*;
use super::{AclImpl, CachedRoles};
use super::acl::*;


#[derive(Clone)]
pub struct SingletonAcl {
    inner: Arc<Mutex<AclImpl<UserRolesRepoImpl>>>,
}

impl SingletonAcl {
     pub fn new(r2d2_pool: DbPool, cpu_pool:CpuPool) -> Self {
        let user_roles_repo = UserRolesRepoImpl::new(r2d2_pool, cpu_pool);
        let cached_roles = CachedRoles::new(user_roles_repo);
        let aclimpl = AclImpl::new(cached_roles);
        Self {
            inner: Arc::new(Mutex::new(aclimpl)),
        }
    }
}

impl Acl for SingletonAcl {
    fn can (&mut self, resource: Resource, action: Action, user_id: i32, resources_with_scope: Vec<&WithScope>) -> bool {
        self.inner.lock().unwrap().can(resource, action, user_id, resources_with_scope)
    }    
}
