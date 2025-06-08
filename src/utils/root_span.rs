use axum::http::StatusCode;
use tracing::{field, info_span, Span};

use crate::storage::{SessionId, TodoId, UserId};

#[allow(dead_code)]
pub enum SamplingPriority {
    Zero,
    One,
}

#[derive(Clone, Debug)]
pub struct RootSpan {
    span: Span,
}

impl RootSpan {
    pub fn new(method: &str, uri: &str) -> Self {
        Self {
            span: info_span!(
                "http_request",
                sampling.priority = tracing::field::Empty,
                method = %method,
                uri    = %uri,
                http_status_code = tracing::field::Empty,
                status = tracing::field::Empty,
                enduser_email = tracing::field::Empty,
                enduser_id = tracing::field::Empty,
                target_user_id = tracing::field::Empty,
                target_user_email = tracing::field::Empty,
                session_id = tracing::field::Empty,
                todo_id = tracing::field::Empty,
            ),
        }
    }

    pub fn enter(&self) -> tracing::span::Entered {
        self.span.enter()
    }

    pub fn record(&self) -> RootSpanRecorder {
        RootSpanRecorder::new(&self.span)
    }
}

pub struct RootSpanRecorder<'a> {
    span: &'a Span,
}

impl<'a> RootSpanRecorder<'a> {
    pub fn new(span: &'a Span) -> Self {
        Self { span }
    }

    pub fn http_status_code(&self, status_code: &StatusCode) -> &Self {
        self.span.record("http_status_code", status_code.as_u16());
        &self
    }

    pub fn status(&self, value: &str) -> &Self {
        self.span.record("status", field::display(value));
        &self
    }

    pub fn enduser_id(&self, id: &UserId) -> &Self {
        self.span.record("enduser_id", field::display(id));
        &self
    }

    pub fn enduser_email(&self, email: &str) -> &Self {
        self.span.record("enduser_email", field::display(email));
        &self
    }

    pub fn target_user_id(&self, id: &UserId) -> &Self {
        self.span.record("target_user_id", field::display(id));
        &self
    }

    pub fn target_user_email(&self, email: &str) -> &Self {
        self.span.record("target_user_email", field::display(email));
        &self
    }

    pub fn todo_id(&self, id: &TodoId) -> &Self {
        self.span.record("todo_id", field::display(id));
        &self
    }

    pub fn session_id(&self, id: &SessionId) -> &Self {
        self.span.record("session_id", field::display(id));
        &self
    }

    pub fn sampling_priority(&self, priority: SamplingPriority) -> &Self {
        let priority = match priority {
            SamplingPriority::One => 1,
            SamplingPriority::Zero => 0,
        };
        self.span
            .record("sampling.priority", field::display(priority));
        &self
    }
}
