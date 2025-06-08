use std::convert::Infallible;

use crate::config::Settings;
use crate::docs::openapi::ApiDoc;
use crate::middleware::rate_limiter::{GlobalRateLimitLayer, PerIpRateLimiter};
use crate::service::Service;
use crate::storage::Role;
use crate::{
    handlers,
    middleware::{auth::auth, metrics::record_metrics, role::require_role, trace_root::trace_root},
};
use axum::routing::delete;
use axum::Extension;
use axum::{
    middleware::{from_fn, from_fn_with_state},
    routing::{get, patch, post},
    Router,
};

use tower_http::trace::TraceLayer;
use tracing::instrument;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

fn admin_routs(settings: &Settings) -> OpenApiRouter<Service> {
    OpenApiRouter::new()
        .route("/users", get(handlers::admin::get_all))
        .route(
            "/user/{id}",
            get(handlers::admin::get).delete(handlers::admin::delete),
        )
        .route("/user/email/{email}", get(handlers::admin::get_by_email))
        .route("/user/{id}/role", patch(handlers::admin::update))
        .layer(from_fn_with_state(Role::Admin, require_role))
        .layer(GlobalRateLimitLayer::new(
            settings.rate_limiter.admin.global.cells_per_second,
            settings.rate_limiter.admin.global.burst_per_second,
        ))
        .layer(PerIpRateLimiter::new(
            settings.rate_limiter.admin.per_ip.cells_per_second,
            settings.rate_limiter.admin.per_ip.burst_per_second,
        ))
}

fn user_routs(settings: &Settings) -> OpenApiRouter<Service> {
    let global_light_limiter = GlobalRateLimitLayer::new(
        settings.rate_limiter.crud_light.global.cells_per_second,
        settings.rate_limiter.crud_light.global.burst_per_second,
    );
    let per_ip_light_limiter = PerIpRateLimiter::new(
        settings.rate_limiter.crud_light.per_ip.cells_per_second,
        settings.rate_limiter.crud_light.per_ip.burst_per_second,
    );
    let global_heavy_limiter = GlobalRateLimitLayer::new(
        settings.rate_limiter.crud_heavy.global.cells_per_second,
        settings.rate_limiter.crud_heavy.global.burst_per_second,
    );
    let per_ip_heavy_limiter = PerIpRateLimiter::new(
        settings.rate_limiter.crud_heavy.per_ip.cells_per_second,
        settings.rate_limiter.crud_heavy.per_ip.burst_per_second,
    );
    OpenApiRouter::new()
        .route(
            "/",
            get(handlers::todo::get_all)
                .layer::<_, Infallible>(global_light_limiter.clone())
                .layer::<_, Infallible>(per_ip_light_limiter.clone()),
        )
        .route(
            "/",
            post(handlers::todo::add)
                .layer::<_, Infallible>(global_light_limiter.clone())
                .layer::<_, Infallible>(per_ip_light_limiter.clone()),
        )
        .route(
            "/",
            delete(handlers::todo::delete_all)
                .layer::<_, Infallible>(global_heavy_limiter.clone())
                .layer::<_, Infallible>(per_ip_heavy_limiter.clone()),
        )
        .route(
            "/{id}",
            get(handlers::todo::get)
                .layer::<_, Infallible>(global_light_limiter.clone())
                .layer::<_, Infallible>(per_ip_light_limiter.clone()),
        )
        .route(
            "/{id}",
            patch(handlers::todo::update)
                .layer::<_, Infallible>(global_heavy_limiter.clone())
                .layer::<_, Infallible>(per_ip_heavy_limiter.clone()),
        )
        .route(
            "/{id}",
            delete(handlers::todo::delete)
                .layer::<_, Infallible>(global_light_limiter.clone())
                .layer::<_, Infallible>(per_ip_light_limiter.clone()),
        )
}

#[instrument(name = "build_app", skip_all)]
pub fn build_app(service: Service, settings: Settings) -> Router {
    let app_router = OpenApiRouter::new()
        .nest("/admin", admin_routs(&settings))
        .nest("/todos", user_routs(&settings))
        .route("/auth/logout", post(handlers::auth::logout))
        .route("/auth/refresh", post(handlers::auth::refresh))
        .layer(from_fn_with_state(service.clone(), auth))
        .route(
            "/auth/register",
            post(handlers::auth::register)
                .layer::<_, Infallible>(GlobalRateLimitLayer::new(
                    settings.rate_limiter.registration.global.cells_per_second,
                    settings.rate_limiter.registration.global.burst_per_second,
                ))
                .layer::<_, Infallible>(PerIpRateLimiter::new(
                    settings.rate_limiter.registration.per_ip.cells_per_second,
                    settings.rate_limiter.registration.per_ip.burst_per_second,
                )),
        )
        .route(
            "/auth/login",
            post(handlers::auth::login)
                .layer::<_, Infallible>(GlobalRateLimitLayer::new(
                    settings.rate_limiter.login.global.cells_per_second,
                    settings.rate_limiter.login.global.burst_per_second,
                ))
                .layer::<_, Infallible>(PerIpRateLimiter::new(
                    settings.rate_limiter.login.per_ip.cells_per_second,
                    settings.rate_limiter.login.per_ip.burst_per_second,
                )),
        )
        .route("/health", get(handlers::health))
        .layer(from_fn(record_metrics))
        .layer(from_fn(trace_root))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(settings))
        .with_state(service);

    let mut doc = ApiDoc::openapi();
    doc.components.as_mut().unwrap().security_schemes.insert(
        "BearerAuth".to_string(),
        SecurityScheme::Http(
            HttpBuilder::new()
                .scheme(HttpAuthScheme::Bearer)
                .bearer_format("JWT")
                .build(),
        ),
    );

    let (router, api) = OpenApiRouter::with_openapi(doc)
        .merge(app_router)
        .split_for_parts();

    router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()))
}
