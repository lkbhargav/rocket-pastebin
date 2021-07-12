extern crate bloom;

use crate::core;
use crate::{handle_err, loop_through_files_in_dir};
use bloom::BloomFilter;
use chrono::NaiveDate;
use chrono::Utc;
use rand::{self, Rng};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Method};
use std::cell::RefCell;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

const BASE62: &'static [u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub struct UniqueID {
  bloom_instance: RefCell<BloomFilter>,
  id_length: usize,
  pub post_request_counter: AtomicUsize,
}

unsafe impl Sync for UniqueID {}

const MAX_CACHE_KEYS_TO_RETAIN: usize = 500;

impl UniqueID {
  pub fn new(expected_num_items: u32, false_positive_rate: f32, id_length: usize) -> UniqueID {
    let mut filter = BloomFilter::with_rate(false_positive_rate, expected_num_items);

    loop_through_files_in_dir!("deletions", filename, {
      let filename = filename.split(".").into_iter().collect::<Vec<&str>>()[0];

      let parsed_date = NaiveDate::parse_from_str(filename, "%Y-%m-%d").unwrap();

      if parsed_date.le(&Utc::now().date().naive_local()) {
        // this takes care of deleting all pastes (recursively) and the file too
        core::Record::delete_all_records_from_the_deletions_and_itself(filename).unwrap();
      }
    });

    let total_uploads_count = loop_through_files_in_dir!("upload", filter);

    if total_uploads_count > 0 {
      println!("Loaded {} keys to Bloom filter!", total_uploads_count);
    }

    UniqueID {
      bloom_instance: RefCell::new(filter),
      id_length,
      post_request_counter: AtomicUsize::new(1),
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

  async fn on_request(&self, req: &mut rocket::Request<'_>, _data: &mut rocket::Data<'_>) {
    if req.method() == Method::Post {
      let mut id;
      // looping through random ids until a non used unique id is found
      loop {
        id = self.generate_id();

        unsafe {
          if !(*self.bloom_instance.as_ptr()).contains(&id) {
            break;
          }
        }
      }

      let mut filter = self.bloom_instance.borrow_mut();

      filter.insert(&id);

      let header = Header::new("unique-pastebin-id", id);
      req.add_header(header);

      if self.post_request_counter.load(Ordering::Relaxed) == MAX_CACHE_KEYS_TO_RETAIN {
        req.add_header(Header::new("time-to-clear-expired-keys", "yes"));
        let val = self
          .post_request_counter
          .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| Some(x * 0));
        handle_err!(val, "error trying to reset the counter value to zero!", {});

        // TODO: reset the bloom filter
        self.bloom_instance.borrow_mut().clear();
        loop_through_files_in_dir!("upload", filter);
      }
    }
  }

  async fn on_response<'r>(&self, _req: &'r rocket::Request<'_>, _res: &mut rocket::Response<'r>) {
    // only on POST method, increment the CacheCounter value by 1.
    if _req.method() == Method::Post {
      self.post_request_counter.fetch_add(1, Ordering::Relaxed);
    }
  }
}
