use rocket::form::{Contextual, Form};
use rocket::http::{Cookie, CookieJar, Status};
use rocket::response::content::RawHtml;
use rocket::response::{status, Redirect};
use rocket::{uri, State};

use crate::data::AppDatabase;
use crate::service::{self, action, ServiceError};
use crate::web::hit_counter::HitCounter;
use crate::ShortCode;

use super::renderer::Renderer;
use super::{ctx, form, PageError, PASSWORD_COOKIE};

#[rocket::get("/")]
fn home(renderer: &State<Renderer<'_>>) -> RawHtml<String> {
    let context = ctx::Home::default();
    RawHtml(renderer.render(context, &[]))
}

#[rocket::get("/<short_code>")]
async fn get_clip(
    short_code: ShortCode,
    database: &State<AppDatabase>,
    hit_counter: &State<HitCounter>,
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
            hit_counter.hit(short_code.clone(), 1);
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

#[rocket::post("/", data = "<form>")]
async fn new_clip(
    form: Form<Contextual<'_, form::NewClip>>,
    database: &State<AppDatabase>,
    renderer: &State<Renderer<'_>>,
) -> Result<Redirect, (Status, RawHtml<String>)> {
    let form = form.into_inner();

    if let Some(value) = form.value {
        let req = service::ask::NewClip {
            content: value.content,
            title: value.title,
            expires_at: value.expires_at,
            password: value.password,
        };

        match action::new_clip(req, database.get_pool()).await {
            Ok(clip) => Ok(Redirect::to(uri!(get_clip(short_code = clip.short_code)))),
            Err(err) => {
                eprintln!("internal error: {}", err);
                Err((
                    Status::InternalServerError,
                    RawHtml(renderer.render(
                        ctx::Home::default(),
                        &["a server error occurred. please try again"],
                    )),
                ))
            }
        }
    } else {
        let errors = form
            .context
            .errors()
            .map(|err| {
                use rocket::form::error::ErrorKind;

                if let ErrorKind::Validation(msg) = &err.kind {
                    msg.as_ref()
                } else {
                    eprintln!("unhandled error: {}", err);
                    "an error occurred. please try again"
                }
            })
            .collect::<Vec<_>>();
        Err((
            Status::BadRequest,
            RawHtml(renderer.render_with_data(
                ctx::Home::default(),
                ("clip", &form.context),
                &errors,
            )),
        ))
    }
}

#[rocket::post("/clip/<short_code>", data = "<form>")]
async fn submit_clip_password(
    cookies: &CookieJar<'_>,
    form: Form<Contextual<'_, form::PasswordProtectedClip>>,
    short_code: ShortCode,
    hit_counter: &State<HitCounter>,
    database: &State<AppDatabase>,
    renderer: &State<Renderer<'_>>,
) -> Result<RawHtml<String>, PageError> {
    if let Some(form) = &form.value {
        let req = service::ask::GetClip {
            short_code: short_code.clone(),
            password: form.password.clone(),
        };

        match action::get_clip(req, database.get_pool()).await {
            Ok(clip) => {
                hit_counter.hit(short_code.clone(), 1);
                let context = ctx::ViewClip::new(clip);

                cookies.add(Cookie::new(
                    PASSWORD_COOKIE,
                    form.password.clone().into_inner().unwrap_or_default(),
                ));

                Ok(RawHtml(renderer.render(context, &[])))
            }
            Err(err) => match err {
                ServiceError::PermissionError(err) => {
                    let context = ctx::PasswordRequired::new(short_code);
                    Ok(RawHtml(renderer.render(context, &[err.as_str()])))
                }
                ServiceError::NotFound => Err(PageError::NotFound("clip not found".to_owned())),
                _ => Err(PageError::Internal("server error".to_owned())),
            },
        }
    } else {
        let context = ctx::PasswordRequired::new(short_code);
        Ok(RawHtml(renderer.render(
            context,
            &["a password is required to view this clip"],
        )))
    }
}

#[rocket::get("/clip/raw/<short_code>")]
async fn get_raw_clip(
    cookies: &CookieJar<'_>,
    short_code: ShortCode,
    database: &State<AppDatabase>,
    hit_counter: &State<HitCounter>,
) -> Result<status::Custom<String>, Status> {
    use crate::domain::clip::field::Password;

    let req = service::ask::GetClip {
        short_code: short_code.clone(),
        password: cookies
            .get(PASSWORD_COOKIE)
            .map(|c| c.value())
            .map(|raw_password| Password::new(raw_password.to_string()).ok())
            .flatten()
            .unwrap_or_else(Password::default),
    };

    match action::get_clip(req, database.get_pool()).await {
        Ok(clip) => {
            hit_counter.hit(short_code.clone(), 1);
            Ok(status::Custom(Status::Ok, clip.content.into_inner()))
        }
        Err(err) => match err {
            ServiceError::PermissionError(msg) => Ok(status::Custom(Status::Unauthorized, msg)),
            ServiceError::NotFound => Err(Status::NotFound),
            _ => Err(Status::InternalServerError),
        },
    }
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![home, get_clip, new_clip, submit_clip_password, get_raw_clip]
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

#[cfg(test)]
pub mod test {
    use crate::{
        data::AppDatabase,
        web::{test::init_test_client, PASSWORD_COOKIE},
    };
    use rocket::http::Status;

    #[test]
    fn gets_home() {
        let (_, client) = init_test_client();

        let response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn error_on_missing_clip() {
        let (_, client) = init_test_client();

        let response = client.get("/clip/not_found").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn requires_password_when_applicable() {
        use crate::domain::clip::field::{Content, ExpiresAt, Password, Title};
        use crate::service;
        use rocket::http::{ContentType, Cookie};

        let (rt, client) = init_test_client();
        let db = client.rocket().state::<AppDatabase>().unwrap();

        let req = service::ask::NewClip {
            content: Content::new("content").unwrap(),
            expires_at: ExpiresAt::default(),
            password: Password::new("123".to_owned()).unwrap(),
            title: Title::default(),
        };

        let clip = rt
            .block_on(async move { service::action::new_clip(req, db.get_pool()).await })
            .unwrap();

        let response = client
            .get(format!("/clip/{}", clip.short_code.as_str()))
            .dispatch();
        assert_eq!(response.status(), Status::NotFound);

        let response = client
            .get(format!("/clip/raw/{}", clip.short_code.as_str()))
            .dispatch();
        assert_eq!(response.status(), Status::Unauthorized);

        let response = client
            .post(format!("/clip/{}", clip.short_code.as_str()))
            .header(ContentType::Form)
            .body(format!("{}={}", "password", "123"))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);

        let response = client
            .get(format!("/clip/raw/{}", clip.short_code.as_str()))
            .cookie(Cookie::new(PASSWORD_COOKIE, "123"))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);

        let response = client
            .get(format!("/clip/raw/{}", clip.short_code.as_str()))
            .cookie(Cookie::new(PASSWORD_COOKIE, "abc"))
            .dispatch();
        assert_eq!(response.status(), Status::Unauthorized);
    }
}
