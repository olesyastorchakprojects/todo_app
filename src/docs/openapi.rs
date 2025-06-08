use utoipa::OpenApi;

use crate::handlers::error::AppError;
use crate::handlers::types::RegisterUser;
use crate::handlers::LoginToken;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::auth::register,
        crate::handlers::auth::login,
        crate::handlers::auth::logout,
        crate::handlers::auth::refresh,
        crate::handlers::admin::get_all,
        crate::handlers::admin::update,
        crate::handlers::admin::delete,
        crate::handlers::admin::get,
        crate::handlers::admin::get_by_email,
        crate::handlers::todo::get_all,
        crate::handlers::todo::get,
        crate::handlers::todo::add,
        crate::handlers::todo::update,
        crate::handlers::todo::delete,
        crate::handlers::todo::delete_all,
    ),
    components(
        schemas(RegisterUser, AppError, LoginToken),
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "todos", description = "Endpoints to create and manage todo items"),
        (name = "admin", description = "Endpoints to manage users, accessible only with Admin role")
    ),
    info(
        title = "Todo API",
        version = "1.0.0"
    )
)]
pub struct ApiDoc;
