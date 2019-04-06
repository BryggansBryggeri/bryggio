use rocket::response;
use std::path::Path;
use std::path::PathBuf;

#[get("/www/static/<file..>")]
pub fn general_files(file: PathBuf) -> Option<response::NamedFile> {
    response::NamedFile::open(Path::new("www/static/").join(file)).ok()
}

#[get("/www/script/<file..>")]
pub fn javascript(file: PathBuf) -> Option<response::NamedFile> {
    response::NamedFile::open(Path::new("www/script/").join(file)).ok()
}
