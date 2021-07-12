use crate::core::Record;
use crate::{handle_err, loop_through_files_in_dir};
use chrono::NaiveDate;
use chrono::{Duration, Utc};
use r_cache::cache::Cache;
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

pub const SIMPLE_DATE_FORMAT: &str = "%Y-%m-%d";

pub async fn add_id_to_file_for_deletion(
  id: String,
  days_to_delete_after: i32,
) -> Result<(), Box<dyn Error>> {
  let (file_path_with_name, _) = get_deletion_file_name_with_path(days_to_delete_after + 1);

  if !Path::new(&file_path_with_name).exists() {
    File::create(&file_path_with_name)?;
  }

  let mut file = OpenOptions::new()
    .write(true)
    .append(true)
    .open(file_path_with_name)
    .unwrap();
  writeln!(file, "{}", id)?;
  Ok(())
}

pub fn get_deletion_file_name_with_path(days_to_add: i32) -> (String, String) {
  let date = Utc::now()
    .checked_add_signed(Duration::days(days_to_add as i64))
    .unwrap();
  (
    format!(
      "deletions/{file_name}.txt",
      file_name = date.format(SIMPLE_DATE_FORMAT)
    ),
    date.format(SIMPLE_DATE_FORMAT).to_string(),
  )
}

pub fn loop_through_files_in_directory<F>(directory_name: &str, callback: F) -> u32
where
  F: Fn(String),
{
  let mut counter: u32 = 0;
  for entry in fs::read_dir(directory_name).unwrap() {
    let dir_entry = entry.unwrap();
    let filename = dir_entry.file_name().into_string().unwrap();
    callback(filename);
    counter += 1;
  }

  counter
}

pub async fn populate_cache_on_first_run(cache: &Cache<String, String>) {
  let today = Utc::now().naive_utc().date();
  loop_through_files_in_dir!("deletions", filename, {
    let date = filename.split(".").collect::<Vec<&str>>()[0];

    let parse_resp = NaiveDate::parse_from_str(date, SIMPLE_DATE_FORMAT);
    handle_err!(parse_resp, "trying to parse the date from the deletions", {
    });

    let date = parse_resp.unwrap();

    if date.ge(&today) {
      let file_resp = std::fs::File::open("deletions/".to_string() + &filename);
      handle_err!(
        file_resp.as_ref(),
        format!(
          "trying to open the qualifying deletions file ({})",
          filename
        ),
        {}
      );

      let reader = BufReader::new(file_resp.unwrap());

      for line in reader.lines() {
        let r = Record::from(line.expect("Something is not right with this line"));

        if !r.is_key_expired() {
          cache
            .set(
              r.key.clone(),
              "".to_string(),
              Some(std::time::Duration::from_secs(
                r.remaining_time_to_expiry() as u64
              )),
            )
            .await;
        }
      }
    }
  });
}
