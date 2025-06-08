use axum::{
    body::Body,
    extract::{Request, State},
    http::{Response, StatusCode},
    middleware::Next,
    Extension,
};
use tracing::{error, info, instrument};

use crate::storage::{Role, User};

#[instrument(name = "middleware::require_role", skip_all)]
pub(crate) async fn require_role(
    State(required): State<Role>,
    Extension(user): Extension<User>,
    request: Request,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    info!(?request, user_email = %user.email, user_role = ?user.role, required_role = ?required, "require_role middleware");

    if user.role != required {
        error!(required_role = ?required, actual_role = ?user.role, "Role check failed");
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}
