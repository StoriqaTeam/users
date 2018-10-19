use stq_router::RouteParser;
use stq_types::{RoleId, UserId};

/// List of all routes with params for the app
#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Healthcheck,
    Users,
    User(UserId),
    UserBlock(UserId),
    UserUnblock(UserId),
    UserBySagaId(String),
    UserCount,
    UsersSearch,
    UsersSearchByEmail,
    UserByEmail,
    Current,
    JWTEmail,
    JWTGoogle,
    JWTFacebook,
    JWTRenew,
    Roles,
    RoleById { id: RoleId },
    RolesByUserId { user_id: UserId },
    PasswordChange,
    UserPasswordResetToken,
    UserEmailVerifyToken,
}

pub fn create_route_parser() -> RouteParser<Route> {
    let mut router = RouteParser::default();

    // Healthcheck
    router.add_route(r"^/healthcheck$", || Route::Healthcheck);

    // Users Routes
    router.add_route(r"^/users$", || Route::Users);

    // User by email Route
    router.add_route(r"^/users/by_email$", || Route::UserByEmail);

    // Users Routes
    router.add_route(r"^/users/current$", || Route::Current);

    // JWT email route
    router.add_route(r"^/jwt/email$", || Route::JWTEmail);

    // JWT google route
    router.add_route(r"^/jwt/google$", || Route::JWTGoogle);

    // JWT facebook route
    router.add_route(r"^/jwt/facebook$", || Route::JWTFacebook);

    // Users/:id route
    router.add_route_with_params(r"^/users/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<UserId>().ok())
            .map(Route::User)
    });

    // Users/:id/block route
    router.add_route_with_params(r"^/users/(\d+)/block$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<UserId>().ok())
            .map(Route::UserBlock)
    });

    // Users/:id/unblock route
    router.add_route_with_params(r"^/users/(\d+)/unblock$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<UserId>().ok())
            .map(Route::UserUnblock)
    });

    // Users/:id route
    router.add_route_with_params(r"^/user_by_saga_id/(.+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<String>().ok())
            .map(Route::UserBySagaId)
    });

    router.add_route(r"^/roles$", || Route::Roles);
    router.add_route_with_params(r"^/roles/by-user-id/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|user_id| Route::RolesByUserId { user_id })
    });
    router.add_route_with_params(r"^/roles/by-id/([a-zA-Z0-9-]+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|id| Route::RoleById { id })
    });

    // /users/count route
    router.add_route(r"^/users/count$", || Route::UserCount);

    // /users/password_change route
    router.add_route(r"^/users/password_change$", || Route::PasswordChange);

    // /users/password_reset_token route
    router.add_route(r"^/users/password_reset_token$", || Route::UserPasswordResetToken);

    // User email verification route
    router.add_route(r"^/users/email_verify_token$", || Route::UserEmailVerifyToken);

    // Search users
    router.add_route(r"^/users/search$", || Route::UsersSearch);

    // Users search by email fuzzy Routes
    router.add_route(r"^/users/search/by_email$", || Route::UsersSearchByEmail);

    router
}
