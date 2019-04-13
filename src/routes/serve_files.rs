use rocket::response;
use rocket::response::status;
use std::path;

#[get("/")]
pub fn index() -> Result<response::NamedFile, status::NotFound<String>> {
    let path = path::Path::new("www/index.html");
    response::NamedFile::open(&path)
        .map_err(|_| status::NotFound(format!("Not found: Index file: {}", path.to_str().unwrap())))
}

#[get("/www/static/<file..>")]
pub fn general_files(file: path::PathBuf) -> Result<response::NamedFile, status::NotFound<String>> {
    let path = path::Path::new("www/static/").join(file);
    response::NamedFile::open(&path)
        .map_err(|_| status::NotFound(format!("Static file not found: {}", path.to_str().unwrap())))
}

#[get("/www/script/<file..>")]
pub fn javascript(file: path::PathBuf) -> Result<response::NamedFile, status::NotFound<String>> {
    let path = path::Path::new("www/script/").join(file);
    response::NamedFile::open(&path)
        .map_err(|_| status::NotFound(format!("Script file not found: {}", path.to_str().unwrap())))
}
