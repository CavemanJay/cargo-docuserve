use std::{cell::RefCell, pin::Pin, rc::Rc, str};

use actix_http::body::BoxBody;
use actix_web::{
    body,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::Error,
    http::StatusCode,
    HttpResponseBuilder,
};

use futures_util::{
    future::{ok, ready, Future, Ready},
    task::{Context, Poll},
};

struct ScriptInjectionResponse;
pub struct ScriptInjectionMiddleware<S> {
    service: Rc<RefCell<S>>,
}

impl<S: 'static> Transform<S, ServiceRequest> for ScriptInjectionResponse
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::error::Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = ScriptInjectionMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ScriptInjectionMiddleware {
            service: Rc::new(RefCell::new(service)),
        })
    }
}

impl<S> Service<ServiceRequest> for ScriptInjectionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        println!("Middleware called");
        let svc = self.service.clone();

        Box::pin(async move {
            /* EXTRACT THE BODY OF REQUEST */
            // let mut request_body = BytesMut::new();
            // while let Some(chunk) = req.take_payload().next().await {
            //     request_body.extend_from_slice(&chunk?);
            // }

            // let mut orig_payload = Payload::empty();
            // orig_payload.unread_data(request_body.freeze());
            // req.set_payload(actix_http::Payload::from(orig_payload));

            /* now process the response */
            let res: ServiceResponse = svc.call(req).await?;

            let content_type = match res.headers().get("content-type") {
                None => "unknown",
                Some(header) => match header.to_str() {
                    Ok(value) => value,
                    Err(_) => "unknown",
                },
            };

            return match res.response().error() {
                None if content_type.starts_with("text/html") => {
                    /* EXTRACT THE BODY OF RESPONSE */
                    let new_req = res.request().clone();
                    let body_bytes = body::to_bytes(res.into_body()).await?;
                    let body_data = match str::from_utf8(&body_bytes) {
                        Ok(str) => str,
                        Err(_) => "Unknown",
                    };
                    let body = inject(body_data);
                    let new_response = HttpResponseBuilder::new(StatusCode::OK).body(body);
                    Ok(ServiceResponse::new(new_req, new_response))
                }
                _ => Ok(res),
            };
        })
    }
}

pub struct ScriptInjectionMiddlewareFactory;

impl ScriptInjectionMiddlewareFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S> Transform<S, ServiceRequest> for ScriptInjectionMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = ScriptInjectionMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ScriptInjectionMiddleware {
            // auth_data: self.auth_data.clone(),
            service: Rc::new(service.into()),
        }))
    }
}

fn inject(html: &str) -> String {
    let x = html.replace("</body>", "<script src=\"/script\"></script></body>");
    x
}
