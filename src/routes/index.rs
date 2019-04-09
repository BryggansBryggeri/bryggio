use rocket::response;
use std::io;

#[get("/")]
pub fn index() -> io::Result<response::NamedFile> {
    response::NamedFile::open("www/index.html")
}
