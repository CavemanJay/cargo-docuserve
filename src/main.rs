use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use middleware::ScriptInjectionMiddlewareFactory;
use std::{
    path::PathBuf,
    process::Command,
    sync::{self, Arc, Mutex},
    thread,
};
use tokio::sync::watch::{channel, Receiver};
// use tokio::sync::broadcast::{channel, Receiver};
use watchdog::Watchdog;
use websocket::ws_index;

mod middleware;
mod watchdog;
mod websocket;

#[derive(Clone)]
struct AppState {
    receiver: Receiver<Message>,
}

const PORT: u16 = 8080;
#[derive(Debug, Clone, Copy)]
pub enum Message {
    Reload,
}

impl ToString for Message {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let root = project_root();
    let (sender, receiver) = channel(Message::Reload);

    // Spawn sender thread.
    // let _handle = thread::spawn(move || loop {
    //     thread::sleep(Duration::from_secs(2));
    //     println!("Generated a message");
    //     sender
    //         .send(Message { int: 4 })
    //         .expect("Failed to write to channel");
    //     println!("Sent a message");
    // });

    let watcher = Watchdog::new(root);
    let app_state = Data::new(AppState { receiver });
    let _handle = thread::spawn(move || {
        watcher.start(|| {
            gen_docs().unwrap();
            // watcher_sender.send(Message::Reload).unwrap();
            sender
                .send(Message::Reload)
                .expect("Failed to send message");
        })
    });

    // open_browser();

    HttpServer::new(move || {
        App::new()
            .wrap(ScriptInjectionMiddlewareFactory::new())
            .app_data(app_state.clone())
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
