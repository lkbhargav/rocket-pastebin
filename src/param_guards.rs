use rocket::request::FromParam;

pub struct ID(pub String);

impl<'r> FromParam<'r> for ID {
  type Error = &'r str;

  fn from_param(param: &'r str) -> Result<Self, Self::Error> {
    match param.chars().all(|c| c.is_ascii_alphanumeric()) {
      true => Ok(ID(param.into())),
      false => Err(param),
    }
  }
}
