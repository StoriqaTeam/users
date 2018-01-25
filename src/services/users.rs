use futures::future;
use futures::Future;
use futures_cpupool::CpuPool;

use models::user::{NewUser, Provider, UpdateUser, User};
use repos::identities::{IdentitiesRepo, IdentitiesRepoImpl};
use repos::users::{UsersRepo, UsersRepoImpl};
use super::types::ServiceFuture;
use super::error::Error;
use repos::types::DbPool;


pub trait UsersService {
    /// Returns user by ID
    fn get(&self, user_id: i32) -> ServiceFuture<User>;
    /// Returns current user
    fn current(&self) -> ServiceFuture<User>;
    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<User>>;
    /// Deactivates specific user
    fn deactivate(&self, user_id: i32) -> ServiceFuture<User>;
    /// Creates new user
    fn create(&self, payload: NewUser) -> ServiceFuture<User>;
    /// Updates specific user
    fn update(&self, user_id: i32, payload: UpdateUser) -> ServiceFuture<User>;
}

/// Users services, responsible for User-related CRUD operations
pub struct UsersServiceImpl<U: 'static + UsersRepo + Clone, I: 'static + IdentitiesRepo+ Clone> {
    pub users_repo: U,
    pub ident_repo: I,
    pub user_email: Option<String>
}

impl UsersServiceImpl<UsersRepoImpl, IdentitiesRepoImpl> {
    pub fn new(r2d2_pool: DbPool, cpu_pool:CpuPool, user_email: Option<String>) -> Self {
        let users_repo = UsersRepoImpl::new(r2d2_pool.clone(), cpu_pool.clone());
        let ident_repo = IdentitiesRepoImpl::new(r2d2_pool, cpu_pool);
        Self {
            users_repo: users_repo,
            ident_repo: ident_repo,
            user_email: user_email
        }
    }
}

impl<U: UsersRepo + Clone, I: IdentitiesRepo + Clone> UsersService for UsersServiceImpl<U, I> {
    /// Returns user by ID
    fn get(&self, user_id: i32) -> ServiceFuture<User> {
        Box::new(self.users_repo.find(user_id).map_err(Error::from))
    }

    /// Returns current user
    fn current(&self) -> ServiceFuture<User>{
        if let Some(ref email) = self.user_email {
            Box::new(self.users_repo.find_by_email(email.to_string()).map_err(Error::from))
        } else {
            Box::new(future::err(Error::Unknown(format!("There is no user email in request header."))))
        }
    }
    
    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<User>> {
        Box::new(
            self.users_repo
                .list(from, count)
                .map_err(|e| Error::from(e)),
        )
    }

    /// Deactivates specific user
    fn deactivate(&self, user_id: i32) -> ServiceFuture<User> {
        Box::new(
            self.users_repo
                .deactivate(user_id)
                .map_err(|e| Error::from(e)),
        )
    }

    /// Creates new user
    fn create(&self, payload: NewUser) -> ServiceFuture<User> {
        let users_repo = self.users_repo.clone();
        let ident_repo = self.ident_repo.clone();
        Box::new(
            ident_repo
                .email_provider_exists(payload.email.to_string(), Provider::Email)
                .map(move |exists| (payload, exists))
                .map_err(Error::from)
                .and_then(|(payload, exists)| match exists {
                    false => future::ok(payload),
                    true => future::err(Error::Validate(
                        validation_errors!({"email": ["email" => "Email already exists"]}),
                    )),
                })
                .and_then(move |new_user| {
                    let update_user = UpdateUser::from(new_user.clone());
                    users_repo
                        .create(update_user)
                        .map_err(|e| Error::from(e))
                        .map(|user| (new_user, user))
                })
                .and_then(move |(new_user, user)| {
                    ident_repo
                        .create(new_user, Provider::Email, user.id)
                        .map_err(|e| Error::from(e))
                        .map(|_| user)
                })
                ,
        )
    }

    /// Updates specific user
    fn update(&self, user_id: i32, payload: UpdateUser) -> ServiceFuture<User> {
        let users_repo = self.users_repo.clone();

        Box::new(
            users_repo
                .find(user_id)
                .and_then(move |_user| users_repo.update(user_id, payload))
                .map_err(|e| Error::from(e)),
        )
    }
}
