use stq_router::RouteParser;
use stq_types::UserId;

/// List of all routes with params for the app
#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Healthcheck,
    Users,
    User(UserId),
    UserBySagaId(String),
    UserByEmail,
    Current,
    JWTEmail,
    JWTGoogle,
    JWTFacebook,
    JWTRenew,
    UserRoles,
    UserRole(UserId),
    DefaultRole(UserId),
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

    // Users/:id route
    router.add_route_with_params(r"^/user_by_saga_id/(.+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<String>().ok())
            .map(Route::UserBySagaId)
    });

    // User Routes
    router.add_route(r"^/user_roles$", || Route::UserRoles);

    // Users/:id route
    router.add_route_with_params(r"^/user_roles/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<UserId>().ok())
            .map(Route::UserRole)
    });

    // roles/default/:id route
    router.add_route_with_params(r"^/roles/default/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<UserId>().ok())
            .map(Route::DefaultRole)
    });

    // /users/password_change route
    router.add_route(r"^/users/password_change$", || Route::PasswordChange);

    // /users/password_reset_token route
    router.add_route(r"^/users/password_reset_token$", || Route::UserPasswordResetToken);

    // User email verification route
    router.add_route(r"^/users/email_verify_token$", || Route::UserEmailVerifyToken);

    router
}
