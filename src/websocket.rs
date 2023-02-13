//! <https://actix.rs/docs/websockets/>
use crate::{AppState, Message};
use actix::AsyncContext;
use actix::{Actor, SpawnHandle, StreamHandler};
use actix_web::{
    web::{self},
    Error, HttpRequest, HttpResponse,
};
use actix_web_actors::ws;
use tokio::sync::watch::Receiver;

struct MyWs {
    receiver: Receiver<Message>,
    spawn_handle: Option<SpawnHandle>,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let mut receiver = self.receiver.clone();
        self.spawn_handle = Some(ctx.add_stream(async_stream::stream! {
            // TODO: This seems to cause an infinite loop on client reload event
            while receiver.changed().await.is_ok() {
                yield receiver.borrow().to_string()
            };
        }));
    }
}

impl StreamHandler<String> for MyWs {
    fn handle(&mut self, msg: String, ctx: &mut Self::Context) {
        ctx.text(msg);
    }
}

pub(crate) async fn ws_index(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    ws::start(
        MyWs {
            receiver: app_state.receiver.clone(),
            spawn_handle: None,
        },
        &req,
        stream,
    )
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, _msg: Result<ws::Message, ws::ProtocolError>, _ctx: &mut Self::Context) {
        print!("Received a message")
    }
}
