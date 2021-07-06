#[macro_use]
extern crate rocket;

use clokwerk::{Scheduler, TimeUnits};
use rocket::data::ToByteUnit;
use rocket::response::Debug;
use rocket::tokio::fs::File;
use rocket::Data;
use rocket_pastebin::fairings::UniqueID;
use rocket_pastebin::param_guards::ID;
use rocket_pastebin::request_guards::UploadRequestGuard;
use rocket_pastebin::util;
use std::time::Duration;

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
async fn upload(paste: Data<'_>, id: UploadRequestGuard) -> Result<String, Debug<std::io::Error>> {
    let filename = format!("upload/{}", id.0);
    let url = format!("{host}/{id}\n", host = "http://localhost:8000", id = id.0);
    paste.open(128.kibibytes()).into_file(filename).await?;
    util::add_id_to_file_for_deletion(id.0, 1).await.unwrap();
    Ok(url)
}

#[get("/<id>")]
async fn retrieve(id: ID) -> Option<File> {
    let filename = format!("upload/{}", id.0);
    File::open(&filename).await.ok()
}

#[launch]
fn rocket() -> _ {
    let mut scheduler = Scheduler::new();

    scheduler
        .every(1.day())
        .at("2:00 am")
        .run(|| util::delete_pastes_from_deletions(""));

    // scheduler
    //     .every(1.seconds())
    //     .run(|| println!("Here we go..."));

    let thread_schedule_handle = scheduler.watch_thread(Duration::from_secs(1));

    let uid = UniqueID::new(1_606_208, 0.01, 4);
    rocket::build()
        .mount("/", routes![index, upload, retrieve])
        .attach(uid)
        .manage(thread_schedule_handle)
}
