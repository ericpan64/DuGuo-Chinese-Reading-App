use rocket::{
    http::RawStr,
    response::NamedFile
};

const HTML_FILEPATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../frontend/html/index.html"); 

#[get("/login")]
pub fn login() -> NamedFile {
    NamedFile::open(HTML_FILEPATH).unwrap()
}

#[get("/feedback")]
pub fn feedback() -> NamedFile {
    NamedFile::open(HTML_FILEPATH).unwrap()
}

#[get("/sandbox")]
pub fn sandbox() -> NamedFile {
    NamedFile::open(HTML_FILEPATH).unwrap()
}

#[get("/reader/<_doc_id>")]
pub fn reader(_doc_id: &RawStr) -> NamedFile {
    NamedFile::open(HTML_FILEPATH).unwrap()
}

#[get("/u/<_uid>")]
pub fn profile(_uid: &RawStr) -> NamedFile {
    NamedFile::open(HTML_FILEPATH).unwrap()
}