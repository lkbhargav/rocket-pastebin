use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::FromRequest;

#[derive(Debug)]
pub struct UploadRequestGuard(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UploadRequestGuard {
  type Error = &'static str;

  async fn from_request(
    request: &'r rocket::Request<'_>,
  ) -> rocket::request::Outcome<Self, Self::Error> {
    let ids: Vec<&str> = request.headers().get("unique-pastebin-id").collect();

    if ids.len() > 0 {
      return Outcome::Success(UploadRequestGuard(ids[0].to_string()));
    }

    Outcome::Failure((
      Status::InternalServerError,
      "Something went wrong while parsing the ID",
    ))
  }
}
