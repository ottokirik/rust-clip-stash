pub mod api;
pub mod ctx;
pub mod form;
pub mod hit_counter;
pub mod http;
pub mod renderer;

pub const PASSWORD_COOKIE: &str = "password-protected-clip";

#[derive(rocket::Responder)]
pub enum PageError {
    #[response(status = 500)]
    Serialization(String),
    #[response(status = 500)]
    Render(String),
    #[response(status = 404)]
    NotFound(String),
    #[response(status = 500)]
    Internal(String),
}

impl From<handlebars::RenderError> for PageError {
    fn from(err: handlebars::RenderError) -> Self {
        Self::Render(format!("{}", err))
    }
}

impl From<serde_json::Error> for PageError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(format!("{}", err))
    }
}
