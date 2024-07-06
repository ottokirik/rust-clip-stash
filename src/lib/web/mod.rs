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

#[cfg(test)]
pub mod test {
    use crate::test::async_runtime;
    use crate::RocketConfig;
    use rocket::local::blocking::Client;
    use tokio::runtime::Handle;

    pub fn init_test_client() -> (tokio::runtime::Runtime, Client) {
        let rt = async_runtime();
        let config = crate::web::test::config(rt.handle());
        let client = client(config);

        (rt, client)
    }

    pub fn config(handle: &Handle) -> RocketConfig {
        use crate::web::{hit_counter::HitCounter, renderer::Renderer};

        let renderer = Renderer::new("templates/".into());
        let database = crate::data::test::new_db(handle);
        let maintenance = crate::domain::maintenance::Maintenance::spawn(
            database.get_pool().clone(),
            handle.clone(),
        );
        let hit_counter = HitCounter::new(database.get_pool().clone(), handle.clone());

        RocketConfig {
            renderer,
            database,
            hit_counter,
            maintenance,
        }
    }

    pub fn client(config: RocketConfig) -> Client {
        Client::tracked(crate::rocket(config)).expect("failed to build rocket instance")
    }
}
