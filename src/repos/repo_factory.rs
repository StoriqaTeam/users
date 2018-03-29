use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;

use repos::*;
use models::*;
use stq_acl::{Acl, SystemACL, UnauthorizedACL};
use repos::error::RepoError;

pub trait ReposFactory<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>
    : Clone + Send + Sync + 'static {
    fn create_users_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<UsersRepo + 'a>;
    fn create_users_repo_with_sys_acl<'a>(&self, db_conn: &'a C) -> Box<UsersRepo + 'a>;
    fn create_identities_repo<'a>(&self, db_conn: &'a C) -> Box<IdentitiesRepo + 'a>;
    fn create_reset_token_repo<'a>(&self, db_conn: &'a C) -> Box<ResetTokenRepo + 'a>;
    fn create_user_roles_repo<'a>(&self, db_conn: &'a C) -> Box<UserRolesRepo + 'a>;
}

#[derive(Clone)]
pub struct ReposFactoryImpl {
    roles_cache: RolesCacheImpl,
}

impl ReposFactoryImpl {
    pub fn new(roles_cache: RolesCacheImpl) -> Self {
        Self { roles_cache }
    }

    pub fn get_roles<'a, C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>(
        &self,
        id: i32,
        db_conn: &'a C,
    ) -> Vec<Role> {
        if self.roles_cache.contains(id) {
            self.roles_cache.get(id)
        } else {
            UserRolesRepoImpl::new(
                db_conn,
                Box::new(SystemACL::default()) as Box<Acl<Resource, Action, Scope, RepoError, UserRole>>,
            ).list_for_user(id)
                .and_then(|ref r| {
                    if !r.is_empty() {
                        self.roles_cache.add_roles(id, r);
                    }
                    Ok(r.clone())
                })
                .ok()
                .unwrap_or_default()
        }
    }

    fn get_acl<'a, T, C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>(
        &self,
        db_conn: &'a C,
        user_id: Option<i32>,
    ) -> Box<Acl<Resource, Action, Scope, RepoError, T>> {
        user_id.map_or(
            Box::new(UnauthorizedACL::default()) as Box<Acl<Resource, Action, Scope, RepoError, T>>,
            |id| {
                let roles = self.get_roles(id, db_conn);
                (Box::new(ApplicationAcl::new(roles, id)) as Box<Acl<Resource, Action, Scope, RepoError, T>>)
            },
        )
    }
}

impl<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ReposFactory<C> for ReposFactoryImpl {
    fn create_users_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<UsersRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(UsersRepoImpl::new(db_conn, acl)) as Box<UsersRepo>
    }

    fn create_users_repo_with_sys_acl<'a>(&self, db_conn: &'a C) -> Box<UsersRepo + 'a> {
        Box::new(UsersRepoImpl::new(
            db_conn,
            Box::new(SystemACL::default()) as Box<Acl<Resource, Action, Scope, RepoError, User>>,
        )) as Box<UsersRepo>
    }

    fn create_identities_repo<'a>(&self, db_conn: &'a C) -> Box<IdentitiesRepo + 'a> {
        Box::new(IdentitiesRepoImpl::new(db_conn)) as Box<IdentitiesRepo>
    }

    fn create_reset_token_repo<'a>(&self, db_conn: &'a C) -> Box<ResetTokenRepo + 'a> {
        Box::new(ResetTokenRepoImpl::new(db_conn)) as Box<ResetTokenRepo>
    }

    fn create_user_roles_repo<'a>(&self, db_conn: &'a C) -> Box<UserRolesRepo + 'a> {
        Box::new(UserRolesRepoImpl::new(
            db_conn,
            Box::new(SystemACL::default()) as Box<Acl<Resource, Action, Scope, RepoError, UserRole>>,
        )) as Box<UserRolesRepo>
    }
}
