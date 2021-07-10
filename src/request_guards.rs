use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::FromRequest;

#[derive(Debug, Default)]
pub struct UploadRequestGuard {
  pub id: String,
  pub clear_expired_keys_from_cache: bool,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UploadRequestGuard {
  type Error = &'static str;

  async fn from_request(
    request: &'r rocket::Request<'_>,
  ) -> rocket::request::Outcome<Self, Self::Error> {
    let mut tmp = UploadRequestGuard {
      ..Default::default()
    };
    let ids: Vec<&str> = request.headers().get("unique-pastebin-id").collect();
    let clear_cache: Vec<&str> = request
      .headers()
      .get("time-to-clear-expired-keys")
      .collect();

    if clear_cache.len() > 0 {
      if clear_cache[0].contains("yes") {
        tmp.clear_expired_keys_from_cache = true;
      }
    }

    if ids.len() > 0 {
      tmp.id = ids[0].to_string();
      return Outcome::Success(tmp);
    }

    Outcome::Failure((
      Status::InternalServerError,
      "Something went wrong while parsing the ID",
    ))
  }
}
