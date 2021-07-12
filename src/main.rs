#[macro_use]
extern crate rocket;

use clokwerk::{Scheduler, TimeUnits};
use r_cache::cache::Cache;
use rocket::data::ToByteUnit;
use rocket::http::Status;
use rocket::tokio::fs::File;
use rocket::{Data, State};
use rocket_pastebin::core::{self, Record};
use rocket_pastebin::fairings::UniqueID;
use rocket_pastebin::param_guards::{TimeParam, ID};
use rocket_pastebin::request_guards::UploadRequestGuard;
use rocket_pastebin::CustomConfig;
use rocket_pastebin::{handle_err, util};
use std::time::Duration;

async fn abstracted_upload_functionality(
    upload_request: UploadRequestGuard,
    custom_config: &CustomConfig,
    paste: Data<'_>,
    cache: &State<Cache<String, String>>,
    expiry_in_seconds: u64,
) -> (Status, String) {
    let filename = format!("upload/{}", upload_request.id);
    let url = format!(
        "{host}/{id}",
        host = custom_config.exposable_url,
        id = upload_request.id
    );

    if upload_request.clear_expired_keys_from_cache {
        cache.remove_expired().await;
    }

    let val = paste.open(128.kibibytes()).into_file(filename).await;

    if val.is_err() {
        return (Status::BadRequest, val.unwrap_err().to_string());
    }

    cache
        .set(
            upload_request.id.clone(),
            "".to_string(),
            Some(Duration::from_secs(expiry_in_seconds)),
        )
        .await;

    let record = Record::new(upload_request.id, expiry_in_seconds);
    let log_resp = record.log_to_particular_day(&Record::get_deletions_date_for_number_of_days(
        expiry_in_seconds as i64,
    ));

    if log_resp.is_err() {
        return (
            Status::InternalServerError,
            log_resp.unwrap_err().to_string(),
        );
    }

    (Status::Ok, url)
}

#[get("/")]
fn index() -> &'static str {
    "
    USAGE

      POST /

          accepts raw data in the body of the request and responds with a URL of
          a page containing the body's content

      GET /<id>

          retrieves the content for the paste with id `<id>`
    "
}

#[post("/", data = "<paste>")]
async fn upload(
    paste: Data<'_>,
    upload_request: UploadRequestGuard,
    cache: &State<Cache<String, String>>,
    custom_config: &State<CustomConfig>,
) -> (Status, String) {
    abstracted_upload_functionality(
        upload_request,
        custom_config.inner(),
        paste,
        cache,
        core::DEFAULT_EXPIRY,
    )
    .await
}

#[get("/<id>")]
async fn retrieve(id: ID, cache: &State<Cache<String, String>>) -> (Status, Option<File>) {
    let val = cache.get(&id.0).await;

    if val.is_none() {
        return (Status::NotFound, None);
    }

    let filename = format!("upload/{}", id.0);
    (Status::Ok, File::open(&filename).await.ok())
}

#[post("/<time>", data = "<paste>")]
async fn custom_upload(
    time: TimeParam,
    upload_request: UploadRequestGuard,
    cache: &State<Cache<String, String>>,
    custom_config: &State<CustomConfig>,
    paste: Data<'_>,
) -> (Status, String) {
    if time.error != "" {
        return (Status::BadRequest, time.error);
    }

    abstracted_upload_functionality(
        upload_request,
        custom_config.inner(),
        paste,
        cache,
        time.duration.as_secs(),
    )
    .await
}

#[launch]
async fn rocket() -> _ {
    let mut scheduler = Scheduler::new();
    let cache = Cache::<String, String>::new(Some(Duration::from_secs(2 * 60 * 60)));

    // populating the cache from the saved pastes
    util::populate_cache_on_first_run(&cache).await;

    let custom_config = CustomConfig::new();

    scheduler.every(1.day()).at("2:00 am").run(|| {
        let val = Record::delete_all_records_from_the_deletions_and_itself(
            // we do `-` before the Math to get the past file
            &Record::get_deletions_date_for_number_of_days(-(86_400 * 7)),
        );
        handle_err!(
            val,
            "Error while running a cron job to delete previous 7th day deletions file!",
            { return }
        );
    });

    // scheduler
    //     .every(1.seconds())
    //     .run(|| println!("Here we go..."));

    let thread_schedule_handle = scheduler.watch_thread(Duration::from_secs(1));

    let uid = UniqueID::new(1_606_208, 0.01, 4);
    rocket::build()
        .mount("/", routes![index, upload, retrieve, custom_upload])
        .attach(uid)
        .manage(thread_schedule_handle)
        .manage(cache)
        .manage(custom_config)
}
