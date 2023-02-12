use actix_web::{web, App, HttpServer};
use middleware::ScriptInjectionMiddlewareFactory;
use std::{path::PathBuf, process::Command, thread};
use websocket::ws_index;

mod middleware;
mod watchdog;
mod websocket;

const PORT: u16 = 8080;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let root = project_root();

    let watcher = watchdog::Watchdog::new(root);
    let _handle = thread::spawn(move || {
        watcher.start(|| {
            gen_docs().unwrap();
        })
    });

    // open_browser();

    HttpServer::new(|| {
        App::new()
            .wrap(ScriptInjectionMiddlewareFactory::new())
            .route("/script", web::get().to(script))
            .route("/ws/", web::get().to(ws_index))
            .service(actix_files::Files::new("/", "target/doc").show_files_listing())
    })
    .bind(("127.0.0.1", PORT))?
    .run()
    .await
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

fn package_name() -> String {
    let mut args = std::env::args().skip_while(|val| !val.starts_with("--manifest-path"));

    let mut cmd = cargo_metadata::MetadataCommand::new();
    let _manifest_path = match args.next() {
        Some(ref p) if p == "--manifest-path" => {
            cmd.manifest_path(args.next().unwrap());
        }
        Some(p) => {
            cmd.manifest_path(p.trim_start_matches("--manifest-path="));
        }
        None => {}
    };

    let _metadata = cmd.exec().unwrap();
    dbg!(_metadata.root_package().unwrap().name.replace("-", "_"))
}

fn open_browser() {
    let name = package_name();
    let (prog, args) = {
        #[cfg(target_os = "windows")]
        {
            ("powershell", vec!["start"])
        }
        #[cfg(target_os = "linux")]
        {
            ("sh", vec!["xdg-open"])
        }
        #[cfg(target_os = "macos")]
        {
            ("open", vec!["-u"])
        }
    };

    Command::new(prog)
        .args(args)
        .arg(format!("http://localhost:{}/{}/index.html", PORT, name))
        .output()
        .unwrap();
}

async fn script() -> &'static str {
    include_str!("script.js")
}
