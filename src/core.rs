use crate::util::get_deletion_file_name_with_path;
use chrono::{self, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::ops::{Add, Sub};
use std::path::Path;
use std::result::Result;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
  pub expiry: u64,
  pub key: String,
  pub created_time: String,
}

/// DEFAULT_EXPIRY = 604_800 => seconds for 1 week
pub const DEFAULT_EXPIRY: u64 = 604_800;

impl Record {
  /// key => Unique ID | expiry => in seconds
  pub fn new(key: String, expiry: u64) -> Self {
    let created_time = chrono::offset::Local::now().to_rfc2822();
    Record {
      expiry,
      key,
      created_time,
    }
  }

  fn internal_log(&self, filename: &str) -> Result<(), Box<dyn Error>> {
    if !Path::new(&filename).exists() {
      let mut f = File::create(&filename)?;
      f.flush()?;
    }

    let mut file = OpenOptions::new().write(true).append(true).open(filename)?;
    let json_data = serde_json::to_string(&self)?;
    writeln!(file, "{}", json_data)?;
    file.flush()?;

    Ok(())
  }

  pub fn log(&self) -> Result<(), Box<dyn Error>> {
    let (file_path_with_name, _) = get_deletion_file_name_with_path(0);
    self.internal_log(&file_path_with_name)?;
    Ok(())
  }

  /// date => 2006-01-25
  pub fn log_to_particular_day(&self, date: &str) -> Result<(), Box<dyn Error>> {
    let filename = format!("deletions/{}.txt", date);
    self.internal_log(&filename)?;
    Ok(())
  }

  /// date => 2006-01-25
  pub fn delete_record_and_file(key: &str, date: &str) -> Result<(), Box<dyn Error>> {
    Record::delete_file(&key)?;

    let filename = format!("deletions/{}.txt", date);

    let mut file = File::open(&filename)?;

    let mut file_contents = String::new();

    file.read_to_string(&mut file_contents)?;

    file.flush()?;

    let filtered_records = file_contents
      .split("\n")
      .filter(|line| line != &"")
      .map(|line| {
        serde_json::from_str::<Record>(line).expect("expected a JSON but found something else.")
      })
      .filter(|record| record.key != key)
      .map(|record| {
        serde_json::to_string::<Record>(&record).expect("expected record to be converted to JSON")
      })
      .reduce(|a, b| format!("{}\n{}", a, b));

    let filtered_records = match filtered_records {
      Some(r) => r,
      None => String::new(),
    };

    let mut file = OpenOptions::new()
      .write(true)
      .truncate(true)
      .open(filename)?;

    file.write_all(&filtered_records.as_bytes())?;

    file.flush()?;

    Ok(())
  }

  pub fn delete_file(record_id: &str) -> Result<bool, Box<dyn Error>> {
    let data_file = format!("upload/{}", record_id);
    let data_file_path = Path::new(&data_file);
    if data_file_path.exists() {
      fs::remove_file(data_file_path)?;
      return Ok(true);
    }

    Ok(false)
  }

  fn internal_get_deletions(seconds: i64) -> String {
    let days_to_add = (seconds as f32 / 86_400 as f32).ceil() as i64;

    let date = Utc::now()
      .checked_add_signed(Duration::days(days_to_add))
      .unwrap();

    date.format(crate::util::SIMPLE_DATE_FORMAT).to_string()
  }

  pub fn get_deletions_date_for_default_expiry() -> String {
    Record::internal_get_deletions(DEFAULT_EXPIRY as i64)
  }

  pub fn get_deletions_date_for_number_of_days(seconds_to_add: i64) -> String {
    Record::internal_get_deletions(seconds_to_add)
  }

  pub fn is_key_expired(&self) -> bool {
    let timestamp = chrono::DateTime::parse_from_rfc2822(&self.created_time).unwrap();
    let added = timestamp.add(Duration::seconds(self.expiry as i64));

    let now = Utc::now();

    if now.ge(&added) {
      return true;
    }

    false
  }

  pub fn remaining_time_to_expiry(&self) -> i64 {
    let created_timestamp = chrono::DateTime::parse_from_rfc2822(&self.created_time).unwrap();
    let created_timestamp = created_timestamp.timestamp() + self.expiry as i64;

    let current_timestamp = Utc::now().timestamp();

    let difference = created_timestamp.sub(current_timestamp);

    if difference <= 0 {
      return 0;
    }

    difference
  }

  /// date => 2006-01-25
  pub fn delete_all_records_from_the_deletions_and_itself(
    date: &str,
  ) -> Result<(), Box<dyn Error>> {
    let filename = format!("deletions/{}.txt", date);

    let mut file = File::open(&filename)?;

    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    for line in contents.split("\n") {
      if line.is_empty() {
        continue;
      }
      let record = serde_json::from_str::<Record>(line)?;

      let upload_filename = format!("upload/{}", record.key);

      if Path::new(&upload_filename).exists() {
        fs::remove_file(upload_filename)?;
      }
    }

    fs::remove_file(filename)?;

    Ok(())
  }
}

impl From<String> for Record {
  fn from(data: String) -> Self {
    let conversion_resp = serde_json::from_str::<Record>(&data);

    conversion_resp.expect("error trying to convert the given string to Record type")
  }
}

#[cfg(test)]
mod tests {
  use std::thread;

  use super::*;

  #[test]
  fn test_for_deletions_date() {
    assert_eq!(
      "2021-07-13",
      Record::get_deletions_date_for_number_of_days(86400)
    );
  }

  #[test]
  fn test_for_deletions_date_2() {
    assert_eq!(
      "2021-07-19",
      Record::get_deletions_date_for_number_of_days(604_800)
    );
  }

  #[test]
  fn test_for_deletions_date_3() {
    assert_eq!(
      "2021-07-15",
      Record::get_deletions_date_for_number_of_days(259_200)
    );
  }

  #[test]
  fn test_for_deletions_date_4() {
    assert_eq!(
      "2021-07-09",
      Record::get_deletions_date_for_number_of_days(-259_200)
    );
  }

  #[test]
  fn test_for_from_method() {
    let data = String::from(r#"{"expiry": 15, "key": "u7F1", "created_time": "2021-07-11"}"#);
    assert_eq!(15, Record::from(data).expiry);
  }

  #[test]
  #[should_panic]
  #[ignore]
  fn test_for_from_method_2() {
    let data = String::from(r#"{"expiry": 15, "key": 89, "created_time": "2021-07-11"}"#);
    Record::from(data);
  }

  #[test]
  fn test_is_key_expired() {
    let r = Record::new("abcd".to_string(), 15);
    assert!(!r.is_key_expired());
  }

  #[test]
  fn test_is_key_expired_2() {
    let r = Record::new("abcd".to_string(), 2);
    assert!(!r.is_key_expired());
    thread::sleep(std::time::Duration::from_secs(3));
    assert!(r.is_key_expired());
  }

  #[test]
  fn test_remaining_time_to_expiry() {
    let r = Record::new("abcd".to_string(), 2);
    assert_eq!(2, r.remaining_time_to_expiry());
  }

  #[test]
  fn test_remaining_time_to_expiry_2() {
    let r = Record::new("abcd".to_string(), 2);
    thread::sleep(std::time::Duration::from_secs(2));
    assert_eq!(0, r.remaining_time_to_expiry());
  }

  #[test]
  fn test_remaining_time_to_expiry_3() {
    let r = Record::new("abcd".to_string(), 30);
    assert_eq!(30, r.remaining_time_to_expiry());
  }
}
