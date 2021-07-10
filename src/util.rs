use chrono::{Duration, Utc};
use std::error::Error;
// use std::fmt::{Debug, Display};
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::path::Path;

pub async fn add_id_to_file_for_deletion(
  id: String,
  days_to_delete_after: i32,
) -> Result<(), Box<dyn Error>> {
  let (file_path_with_name, _) = get_deletion_file_name_with_path(days_to_delete_after + 1);

  // let opened_file = File::open(&file_path_with_name)
  //   .unwrap_or_else(move |_: std::io::Error| File::open(file_path_with_name).unwrap());

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
      file_name = date.format("%Y-%m-%d")
    ),
    date.format("%Y-%m-%d").to_string(),
  )
}

pub fn delete_pastes_from_deletions(file_path: &str) {
  let mut file_path = String::from(file_path);

  if file_path == "" {
    let (file_path_for_today, _) = get_deletion_file_name_with_path(0);
    file_path = file_path_for_today;
  }

  if Path::new(&file_path).exists() {
    let mut contents = String::new();
    let mut file = std::fs::File::open(&file_path).unwrap();
    file.read_to_string(&mut contents).unwrap();

    let mut total_pastes_deleted: u32 = 0;

    for line in contents.lines() {
      let filename = format!("upload/{}", line);

      if Path::new(&filename).exists() {
        // remove individual paste file
        fs::remove_file(filename).unwrap();
        total_pastes_deleted += 1;
      }
    }
    // removing the deletions file
    fs::remove_file(&file_path).unwrap();

    println!(
      "Deleted {} file along with {} pastes!",
      file_path, total_pastes_deleted
    );
  }
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
