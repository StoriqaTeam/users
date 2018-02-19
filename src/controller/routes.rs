use models::UserId;

use stq_router::RouteParser;

/// List of all routes with params for the app
#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Healthcheck,
    Users,
    User(UserId),
    Current,
    JWTEmail,
    JWTGoogle,
    JWTFacebook,
    UserRoles,
    UserRole(i32),
}

pub fn create_route_parser() -> RouteParser<Route> {
    let mut router = RouteParser::default();

    // Healthcheck
    router.add_route(r"^/healthcheck$", || Route::Healthcheck);

    // Users Routes
    router.add_route(r"^/users$", || Route::Users);

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
            .map(|user_id| Route::User(user_id))
    });

    // User Routes
    router.add_route(r"^/user_roles$", || Route::UserRoles);

    // Users/:id route
    router.add_route_with_params(r"^/user_roles/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse::<i32>().ok())
            .map(|user_id| Route::UserRole(user_id))
    });

    router
}
