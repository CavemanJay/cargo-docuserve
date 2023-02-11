use actix_web::{web, App, HttpServer};
use middleware::ScriptInjectionMiddlewareFactory;
use std::{
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex},
    thread,
};

mod middleware;
mod watchdog;

const PORT: u16 = 8080;

struct State {
    last_modified: Arc<Mutex<u32>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let root = project_root();

    let count: Arc<Mutex<u32>> = Mutex::new(0).into();
    let state = State {
        last_modified: count.clone(),
    };
    let watcher_state = count.clone();

    let watcher = watchdog::Watchdog::new(root);
    let _handle = thread::spawn(move || {
        watcher.start(|| {
            let mut x = watcher_state.lock().unwrap();
            *x += 1;
            dbg!(*x);
            gen_docs().unwrap();
        })
    });

    // open_browser();

    HttpServer::new(|| {
        App::new()
            .wrap(ScriptInjectionMiddlewareFactory::new())
            .route("/script", web::get().to(script))
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
