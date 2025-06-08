pub(crate) mod auth;
pub(crate) mod metrics;
pub(crate) mod rate_limiter;
pub(crate) mod role;
pub(crate) mod trace_root;

use once_cell::sync::Lazy;
use regex::Regex;

static UUID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")
        .expect("Invalid UUID regex")
});

pub(crate) fn normalize_uri(path: &str) -> String {
    if path.starts_with("/admin/user/email") {
        return "/admin/user/email/{email}".to_string();
    }
    UUID_REGEX.replace_all(path, "{id}").to_string()
}

#[cfg(test)]
mod tests {
    use super::normalize_uri;

    #[test]
    fn normalize_uri_test() {
        let ok_str = normalize_uri("/todos/27436a8c-3f55-498a-8fb8-9a3ab17f9930");
        assert_eq!(ok_str, "/todos/{id}");
    }

    #[test]
    fn normalize_uri_with_email_test() {
        let ok_str = normalize_uri("/admin/user/email/nm98P5VIrUdHUGLZVEW8");
        assert_eq!(ok_str, "/admin/user/email/{email}");
    }
}
