use rocket::response;
use std::io;
use std::path::Path;
use std::path::PathBuf;

#[get("/www/static/<file..>")]
pub fn general_files(file: PathBuf) -> io::Result<response::NamedFile> {
    response::NamedFile::open(Path::new("www/static/").join(file))
}

#[get("/www/script/<file..>")]
pub fn javascript(file: PathBuf) -> io::Result<response::NamedFile> {
    response::NamedFile::open(Path::new("www/script/").join(file))
}
