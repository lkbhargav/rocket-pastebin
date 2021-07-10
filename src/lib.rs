pub mod core;
pub mod fairings;
pub mod macros;
pub mod param_guards;
pub mod request_guards;
pub mod util;

pub struct CustomConfig {
  pub exposable_url: String,
}

impl CustomConfig {
  pub fn new() -> Self {
    let exposable_url = std::env::var("PASTEBIN_EXPOSABLE_URL")
      .or::<String>(Ok(String::from("http://localhost:8000")))
      .unwrap();

    CustomConfig { exposable_url }
  }
}
