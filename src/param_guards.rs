use crate::core::Record;
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

#[derive(Debug)]
pub struct TimeParam {
  pub duration: std::time::Duration,
  pub is_more_than_a_day: bool,
  pub error: String,
  pub deletion_date: String,
}

impl<'r> FromParam<'r> for TimeParam {
  type Error = String;

  fn from_param(param: &'r str) -> Result<Self, Self::Error> {
    let params = param
      .split(&['d', 'm', 'h', 's'][..])
      .collect::<Vec<&str>>();

    let mut u32_params = vec![];

    for param in params {
      if param.is_empty() {
        continue;
      }

      let parsed_data = param.parse::<u32>();

      if parsed_data.is_err() {
        continue;
      }

      u32_params.push(parsed_data.unwrap());
    }

    if u32_params.len() == 0 {
      return Err(format!(
        "This route call might not be intentional! Input: ({})",
        param
      ));
    }

    let params = u32_params;

    let mut error_message = String::new();

    #[derive(Default, Debug)]
    struct Holder {
      day: u32,
      minute: u32,
      hour: u32,
      second: u32,
    }

    let mut h = Holder {
      ..Default::default()
    };

    let mut n = 1;

    for i in ["s", "m", "h", "d"] {
      if param.contains(i) {
        let val = params[params.len() - n];
        if i == "s" {
          if val > 59 {
            error_message = format!(
              "error parsing `second` param ({}). `second` has to be less than 60.",
              param
            );
            break;
          }
          h.second = val;
        } else if i == "m" {
          if val > 59 {
            error_message = format!(
              "error parsing `minute` param ({}). `minute` has to be less than 60.",
              param
            );
            break;
          }
          h.minute = val;
        } else if i == "h" {
          if val > 23 {
            error_message = format!(
              "error parsing `hour` param ({}). `hour` has to be less than 24.",
              param
            );
            break;
          }
          h.hour = val;
        } else {
          if val > 30 {
            error_message = format!(
              "error parsing `day` param ({}). `day` has to be less than 31.",
              param
            );
            break;
          }
          h.day = val;
        }
        n += 1;
      }
    }

    let literal = (h.day * 24 * 60 * 60) + (h.hour * 60 * 60) + (h.minute * 60) + h.second;

    Ok(TimeParam {
      duration: std::time::Duration::from_secs(literal as u64),
      is_more_than_a_day: literal >= 86_400,
      error: error_message,
      deletion_date: Record::get_deletions_date_for_number_of_days(literal as i64),
    })
  }
}
