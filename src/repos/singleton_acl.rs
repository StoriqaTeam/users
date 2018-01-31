use std::sync::{Arc, Mutex, Once, ONCE_INIT};
use std::{mem};

use repos::acl::{AclImpl, CachedRoles};
use repos::user_roles::UserRolesRepoImpl;

#[derive(Clone)]
pub struct SingletonAcl {
    // Since we will be used in many threads, we need to protect
    // concurrent access
    pub inner: Arc<Mutex<AclImpl<CachedRoles<UserRolesRepoImpl>>>>,
}

pub fn get_acl(repo: UserRolesRepoImpl) -> SingletonAcl {
    // Initialize it to a null value
    static mut SINGLETON: *const SingletonAcl = 0 as *const SingletonAcl;
    static ONCE: Once = ONCE_INIT;

    unsafe {
        ONCE.call_once(|| {
            // Make it
            let cached_roles = CachedRoles::new(repo);
            let acl = AclImpl::new(cached_roles);
            let singleton = SingletonAcl {
                inner: Arc::new(Mutex::new(acl)),
            };

            // Put it in the heap so it can outlive this call
            SINGLETON = mem::transmute(Box::new(singleton));
        });

        // Now we give out a copy of the data that is safe to use concurrently.
        (*SINGLETON).clone()
    }
}