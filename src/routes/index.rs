use bryggio::brewery::BrewState;
use rocket::response;
use rocket::State;
use rocket_contrib::json::Json;
use serde_json;
use std::collections::HashMap;
use std::io;
use std::path;

#[get("/")]
pub fn index() -> io::Result<response::NamedFile> {
    response::NamedFile::open("www/index.html")
}
