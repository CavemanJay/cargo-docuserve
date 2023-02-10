use axum::{
    body::{boxed, Body, BoxBody},
    http::{Request, Response, StatusCode, Uri},
    response::Redirect,
    routing::get,
};

use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};

pub async fn file_handler(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let res = get_static_file(uri.clone()).await?;

    if res.status() == StatusCode::NOT_FOUND {
        // try with `.html`
        // TODO: handle if the Uri has query parameters
        match format!("{}.html", uri).parse() {
            Ok(uri_html) => get_static_file(uri_html).await,
            Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Invalid URI".to_string())),
        }
    } else {
        Ok(res)
    }
}

async fn get_static_file(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    // dbg!(std::env::var("docuserve_root").unwrap());

    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    match ServeDir::new("target/doc")
        .append_index_html_on_directories(true)
        .oneshot(req)
        .await
    {
        Ok(res) => Ok(res.map(boxed)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", err),
        )),
    }
}

pub async fn not_found_handler(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    dbg!(uri);
    unimplemented!()
}

macro_rules! encode_file_name {
    ($entry:ident) => {
        escape_html_entity(&$entry.file_name().to_string_lossy())
    };
}


/// https://github.com/actix/actix-web/blob/master/actix-files/src/directory.rs
pub(crate) fn directory_listing(){}
// pub(crate) fn directory_listing(
//     dir: &Directory,
//     req: &HttpRequest,
// ) -> Result<ServiceResponse, io::Error> {
//     let index_of = format!("Index of {}", req.path());
//     let mut body = String::new();
//     let base = Path::new(req.path());

//     for entry in dir.path.read_dir()? {
//         if dir.is_visible(&entry) {
//             let entry = entry.unwrap();
//             let p = match entry.path().strip_prefix(&dir.path) {
//                 Ok(p) if cfg!(windows) => base.join(p).to_string_lossy().replace('\\', "/"),
//                 Ok(p) => base.join(p).to_string_lossy().into_owned(),
//                 Err(_) => continue,
//             };

//             // if file is a directory, add '/' to the end of the name
//             if let Ok(metadata) = entry.metadata() {
//                 if metadata.is_dir() {
//                     let _ = write!(
//                         body,
//                         "<li><a href=\"{}\">{}/</a></li>",
//                         encode_file_url!(p),
//                         encode_file_name!(entry),
//                     );
//                 } else {
//                     let _ = write!(
//                         body,
//                         "<li><a href=\"{}\">{}</a></li>",
//                         encode_file_url!(p),
//                         encode_file_name!(entry),
//                     );
//                 }
//             } else {
//                 continue;
//             }
//         }
//     }

//     let html = format!(
//         "<html>\
//          <head><title>{}</title></head>\
//          <body><h1>{}</h1>\
//          <ul>\
//          {}\
//          </ul></body>\n</html>",
//         index_of, index_of, body
//     );
//     Ok(ServiceResponse::new(
//         req.clone(),
//         HttpResponse::Ok()
//             .content_type("text/html; charset=utf-8")
//             .body(html),
//     ))
// }
