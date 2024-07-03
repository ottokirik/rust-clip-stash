use rocket::http::Status;
use rocket::response::content::RawHtml;
use rocket::response::status;
use rocket::State;

use crate::data::AppDatabase;
use crate::service::{action, ServiceError};
use crate::ShortCode;

use super::renderer::Renderer;
use super::{ctx, PageError};

#[rocket::get("/")]
fn home(renderer: &State<Renderer<'_>>) -> RawHtml<String> {
    let context = ctx::Home::default();
    RawHtml(renderer.render(context, &[]))
}

#[rocket::get("/<short_code>")]
async fn get_clip(
    short_code: ShortCode,
    database: &State<AppDatabase>,
    renderer: &State<Renderer<'_>>,
) -> Result<status::Custom<RawHtml<String>>, PageError> {
    fn render_with_status<T: ctx::PageContext + serde::Serialize + std::fmt::Debug>(
        status: Status,
        context: T,
        renderer: &Renderer,
    ) -> Result<status::Custom<RawHtml<String>>, PageError> {
        Ok(status::Custom(
            status,
            RawHtml(renderer.render(context, &[])),
        ))
    }

    match action::get_clip(short_code.clone().into(), database.get_pool()).await {
        Ok(clip) => {
            let context = ctx::ViewClip::new(clip);

            render_with_status(Status::Ok, context, renderer)
        }
        Err(err) => match err {
            ServiceError::PermissionError(_) => {
                let context = ctx::PasswordRequired::new(short_code);
                render_with_status(Status::Unauthorized, context, renderer)
            }
            ServiceError::NotFound => Err(PageError::NotFound("clip not found".to_owned())),
            _ => Err(PageError::Internal("server error".to_owned())),
        },
    }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![home, get_clip]
}

pub mod catcher {
    use rocket::Request;
    use rocket::{catch, catchers, Catcher};

    #[catch(default)]
    fn default(req: &Request) -> &'static str {
        eprintln!("unhandled request: {}", req);
        "something went wrong..."
    }

    #[catch(500)]
    fn internal_error(req: &Request) -> &'static str {
        eprintln!("internal error: {}", req);
        "internal server error"
    }

    #[catch(404)]
    fn not_found() -> &'static str {
        "404"
    }

    pub fn catchers() -> Vec<Catcher> {
        catchers![default, internal_error, not_found]
    }
}
