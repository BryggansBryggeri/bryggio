use rocket::response::NamedFile;
use std::path::Path;
use std::path::PathBuf;

#[get("/static/<file..>")]
pub fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}
