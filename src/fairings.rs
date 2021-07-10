extern crate bloom;

use crate::loop_through_files_in_dir;
use crate::util;
use bloom::BloomFilter;
use chrono::NaiveDate;
use chrono::Utc;
use rand::{self, Rng};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Method};
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

const BASE62: &'static [u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub struct UniqueID {
  bloom_instance: BloomFilter,
  id_length: usize,
}

impl UniqueID {
  pub fn new(expected_num_items: u32, false_positive_rate: f32, id_length: usize) -> UniqueID {
    let mut filter = BloomFilter::with_rate(false_positive_rate, expected_num_items);

    loop_through_files_in_dir!("deletions", filename, {
      let filename = filename.split(".").into_iter().collect::<Vec<&str>>()[0];

      let parsed_date = NaiveDate::parse_from_str(filename, "%Y-%m-%d").unwrap();

      if parsed_date.le(&Utc::now().date().naive_local()) {
        // this takes care of deleting all pastes (recursively) and the file too
        util::delete_pastes_from_deletions(&format!("deletions/{}.txt", filename));
      }
    });

    let total_uploads_count = loop_through_files_in_dir!("upload", filter);

    if total_uploads_count > 0 {
      println!("Loaded {} keys to Bloom filter!", total_uploads_count);
    }

    UniqueID {
      bloom_instance: filter,
      id_length,
    }
  }

  pub fn generate_id(&self) -> String {
    let mut id_str = String::with_capacity(self.id_length);
    let mut rng = rand::thread_rng();

    for _ in 0..self.id_length {
      id_str.push(BASE62[rng.gen_range(0..62)] as char);
    }

    id_str
  }
}

#[rocket::async_trait]
impl Fairing for UniqueID {
  fn info(&self) -> Info {
    Info {
      name: "Unique ID",
      kind: Kind::Liftoff | Kind::Request,
    }
  }

  async fn on_liftoff(&self, _rocket: &rocket::Rocket<rocket::Orbit>) {
    println!(
      "Total bits in play: {} | Number of hashes in use: {}",
      self.bloom_instance.num_bits(),
      self.bloom_instance.num_hashes()
    );
  }

  async fn on_request(&self, req: &mut rocket::Request<'_>, _data: &mut rocket::Data<'_>) {
    if req.method() == Method::Post {
      let mut id;
      // looping through random ids until a non used unique id is found
      loop {
        id = self.generate_id();
        if !self.bloom_instance.contains(&id) {
          break;
        }
      }

      let header = Header::new("unique-pastebin-id", id);
      req.add_header(header);
    }

    if req.method() == Method::Get {
      if let Some(r) = req.routed_segment(0) {
        if r == "test" {
          let id = req.routed_segment(1).unwrap_or("");
          if id != "" {
            if self.bloom_instance.contains(&id) {
              println!("Found key ({})!", id);
            } else {
              println!("Key ({}) not found!", id);
            }
          }
        }
      }
    }
  }
}

pub struct CacheCounter(pub AtomicUsize);

const MAX_CACHE_KEYS_TO_RETAIN: usize = 500;

impl CacheCounter {
  pub fn new() -> Self {
    CacheCounter(AtomicUsize::new(1))
  }
}

#[rocket::async_trait]
impl Fairing for CacheCounter {
  fn info(&self) -> Info {
    Info {
      name: "Cache counter",
      kind: Kind::Response,
    }
  }

  async fn on_response<'r>(&self, _req: &'r rocket::Request<'_>, _res: &mut rocket::Response<'r>) {
    // only on POST method, increment the CacheCounter value by 1.
    if _req.method() == Method::Post {
      self.0.fetch_add(1, Ordering::Relaxed);
    }
  }

  async fn on_request(&self, _req: &mut rocket::Request<'_>, _data: &mut rocket::Data<'_>) {
    if _req.method() == Method::Post {
      if self.0.load(Ordering::Relaxed) == MAX_CACHE_KEYS_TO_RETAIN {
        _req.add_header(Header::new("time-to-clear-expired-keys", "yes"));
        let val = self
          .0
          .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| Some(x * 0));

        if val.is_err() {
          println!(
            "error trying to reset the counter value to zero. Actual error: {}",
            val.unwrap_err()
          );
        }
      }
    }
  }
}
