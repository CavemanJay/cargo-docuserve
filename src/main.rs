use axum::{
    body::boxed,
    http::{Request, StatusCode},
    middleware::{from_fn, Next},
    response::Response,
    routing::get,
    Router,
};
use server::{file_handler, not_found_handler};
use std::{net::SocketAddr, path::PathBuf, process::Command, thread};

mod server;
mod watchdog;

#[tokio::main]
async fn main() {
    let root = project_root();
    std::env::set_var("docuserve_root", PathBuf::from(&root).file_name().unwrap());

    let watcher = watchdog::Watchdog::new(root);
    let y = thread::spawn(move || {
        watcher.start(|| {
            println!("Generating docs");
            gen_docs().unwrap();
        })
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], 4444));
    let app = Router::new()
        .nest_service("/", get(file_handler).fallback(not_found_handler))
        // .fallback(not_found_handler)
        .layer(from_fn(my_middleware));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    y.join().unwrap();
    // Ok(())
}

async fn my_middleware<B>(request: Request<B>, next: Next<B>) -> Response {
    dbg!(request.uri());
    let response = next.run(request).await;

    match response.status() {
        StatusCode::NOT_FOUND => {
            let mut t = tera::Tera::default();
            let mut context = tera::Context::new();
            context.insert("links", &vec!["index.html"]);
            let html = t
                .render_str(include_str!("../index.html"), &context)
                .unwrap();
            Response::builder().body(html).unwrap().map(boxed)
        }
        _ => response,
    }
}

fn gen_docs() -> Result<std::process::Output, std::io::Error> {
    #[cfg(debug_assertions)]
    {
        Command::new("cargo").arg("doc").arg("--no-deps").output()
    }
    #[cfg(not(debug_assertions))]
    {
        Command::new("cargo").arg("doc").output()
    }
}

pub fn project_root() -> String {
    Command::new("cargo")
        .arg("locate-project")
        .arg("--message-format")
        .arg("plain")
        .output()
        .map_err(|err| err.to_string())
        .and_then(|out| String::from_utf8(out.stdout).map_err(|err| err.to_string()))
        .map(PathBuf::from)
        .and_then(|path| {
            path.parent()
                .ok_or_else(|| String::from("project root does not exist"))
                .map(|p| p.to_string_lossy().to_string())
        })
        .unwrap()
}
