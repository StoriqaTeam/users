use std::sync::Arc;

use diesel;
use diesel::select;
use diesel::dsl::exists;
use diesel::prelude::*;
use futures::future;
use futures::Future;
use futures_cpupool::CpuPool;

use common::{TheConnection, ThePool};
use models::schema::users::dsl::*;
use models::user::{User};
use payloads::user::{NewUser, UpdateUser};

/// Users repository, responsible for handling users
pub struct UsersRepo {
    pub r2d2_pool: Arc<ThePool>,
    pub cpu_pool: Arc<CpuPool>
}

impl UsersRepo {
    fn get_connection(&self) -> TheConnection {
        match self.r2d2_pool.get() {
            Ok(connection) => connection,
            Err(e) => panic!("Error obtaining connection from pool: {}", e)
        }
    }

    /// Find specific user by ID
    pub fn find(&self, user_id: i32) -> Box<Future<Item=User, Error=diesel::result::Error>> {
        let conn = self.get_connection();
        let query = users.find(user_id);
        //query.get_result::<User>(&*conn)

        let future = self.cpu_pool.spawn_fn(move || {
            query.get_result(&*conn)
        }).then(|r| match r {
            Ok(data) => future::ok(data),
            Err(err) => future::err(err)
        });

        Box::new(future)
    }

    /// Checks if e-mail is already registered
    pub fn email_exists(&self, needle: String) -> diesel::QueryResult<bool> {
        let conn = self.get_connection();
        let query = select(exists(users.filter(email.eq(needle))));
        query.get_result::<bool>(&*conn)
    }

    /// Returns list of users, limited by `from` and `count` parameters
    pub fn list(&self, from: i32, count: i64) -> diesel::QueryResult<Vec<User>> {
        let conn = self.get_connection();
        let query = users.filter(is_active.eq(true)).filter(id.gt(from)).order(id).limit(count);
        query.get_results::<User>(&*conn)
    }

    /// Creates new user
    pub fn create(&self, payload: NewUser) -> diesel::QueryResult<User> {
        let conn = self.get_connection();
        let query = diesel::insert_into(users).values(&payload);
        query.get_result::<User>(&*conn)
    }

    /// Updates specific user
    pub fn update(&self, user_id: i32, payload: &UpdateUser) -> diesel::QueryResult<User> {
        let conn = self.get_connection();
        let filter = users.filter(id.eq(user_id)).filter(is_active.eq(true));
        let query = diesel::update(filter).set(email.eq(payload.email));
        query.get_result::<User>(&*conn)
    }

    /// Deactivates specific user
    pub fn deactivate(&self, user_id: i32) -> diesel::QueryResult<User> {
        let conn = self.get_connection();
        let filter = users.filter(id.eq(user_id)).filter(is_active.eq(true));
        let query = diesel::update(filter).set(is_active.eq(false));
        query.get_result::<User>(&*conn)
    }
}
