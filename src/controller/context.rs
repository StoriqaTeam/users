//! `Context` is a top level module containg static context and dynamic context for each request
use std::sync::Arc;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use stq_http::client::{ClientHandle, TimeLimitedHttpClient};
use stq_router::RouteParser;
use stq_types::UserId;

use super::routes::*;
use config::{ApiMode, Config};
use repos::repo_factory::*;
use services::jwt::profile::{FacebookProfile, GoogleProfile};
use services::jwt::{JWTProviderService, JWTProviderServiceImpl};
use services::mocks::jwt::JWTProviderServiceMock;

/// Static context for all app
pub struct StaticContext<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
{
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub config: Arc<Config>,
    pub route_parser: Arc<RouteParser<Route>>,
    pub client_handle: ClientHandle,
    pub repo_factory: F,
    pub jwt_private_key: Vec<u8>,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > StaticContext<T, M, F>
{
    /// Create a new static context
    pub fn new(
        db_pool: Pool<M>,
        cpu_pool: CpuPool,
        client_handle: ClientHandle,
        config: Arc<Config>,
        repo_factory: F,
        jwt_private_key: Vec<u8>,
    ) -> Self {
        let route_parser = Arc::new(create_route_parser());
        Self {
            route_parser,
            db_pool,
            cpu_pool,
            client_handle,
            config,
            repo_factory,
            jwt_private_key,
        }
    }

    /// Creates dynamic context services
    pub fn dynamic_context_services(&self, time_limited_http_client: TimeLimitedHttpClient<ClientHandle>) -> DynamicContextServices {
        let google_provider_service: Arc<JWTProviderService<GoogleProfile>> =
            if self.config.testmode.as_ref().and_then(|t| t.get("jwt")) == Some(&ApiMode::Mock) {
                Arc::new(JWTProviderServiceMock)
            } else {
                Arc::new(JWTProviderServiceImpl {
                    http_client: time_limited_http_client.clone(),
                })
            };

        let facebook_provider_service: Arc<JWTProviderService<FacebookProfile>> =
            if self.config.testmode.as_ref().and_then(|t| t.get("jwt")) == Some(&ApiMode::Mock) {
                Arc::new(JWTProviderServiceMock)
            } else {
                Arc::new(JWTProviderServiceImpl {
                    http_client: time_limited_http_client,
                })
            };

        DynamicContextServices {
            google_provider_service,
            facebook_provider_service,
        }
    }
}

pub struct DynamicContextServices {
    pub google_provider_service: Arc<JWTProviderService<GoogleProfile>>,
    pub facebook_provider_service: Arc<JWTProviderService<FacebookProfile>>,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > Clone for StaticContext<T, M, F>
{
    fn clone(&self) -> Self {
        Self {
            cpu_pool: self.cpu_pool.clone(),
            db_pool: self.db_pool.clone(),
            route_parser: self.route_parser.clone(),
            client_handle: self.client_handle.clone(),
            config: self.config.clone(),
            repo_factory: self.repo_factory.clone(),
            jwt_private_key: self.jwt_private_key.clone(),
        }
    }
}

/// Dynamic context for each request
#[derive(Clone)]
pub struct DynamicContext {
    pub user_id: Option<UserId>,
    pub correlation_token: String,
    pub http_client: TimeLimitedHttpClient<ClientHandle>,
    pub google_provider_service: Arc<JWTProviderService<GoogleProfile>>,
    pub facebook_provider_service: Arc<JWTProviderService<FacebookProfile>>,
}

impl DynamicContext {
    /// Create a new dynamic context for each request
    pub fn new(
        user_id: Option<UserId>,
        correlation_token: String,
        http_client: TimeLimitedHttpClient<ClientHandle>,
        google_provider_service: Arc<JWTProviderService<GoogleProfile>>,
        facebook_provider_service: Arc<JWTProviderService<FacebookProfile>>,
    ) -> Self {
        Self {
            user_id,
            correlation_token,
            http_client,
            google_provider_service,
            facebook_provider_service,
        }
    }

    pub fn is_super_admin(&self) -> bool {
        self.user_id == Some(UserId(1))
    }
}
