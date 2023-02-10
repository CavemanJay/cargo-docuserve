use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::Pin;
use std::rc::Rc;
use std::str;

// use erp_contrib::{actix_http, actix_web, futures, serde_json};

use actix_http::body::BoxBody;
use actix_http::{h1::Payload, header};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::web::BytesMut;
use actix_web::{body, error::Error, http::StatusCode, HttpMessage, HttpResponseBuilder};

use futures_util::future::ready;
use futures_util::StreamExt;
use futures_util::{
    future::{ok, Future, Ready},
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

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
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
                    Ok(value) => dbg!(value),
                    Err(_) => "unknown",
                },
            };

            return match res.response().error() {
                None => {
                    //     /* EXTRACT THE BODY OF RESPONSE */
                    let new_req = res.request().clone();
                    let body_bytes = body::to_bytes(res.into_body()).await?;
                    let body_data = match str::from_utf8(&body_bytes) {
                        Ok(str) => str,
                        Err(_) => "Unknown",
                    };
                    dbg!(&body_data);
                    // Ok(res)
                    let new_response = HttpResponseBuilder::new(StatusCode::OK).finish();
                    Ok(ServiceResponse::new(new_req, new_response))
                }
                Some(error) => {
                    // if content_type.to_uppercase().contains("APPLICATION/JSON") {
                    //     Ok(res)
                    // } else {
                    //     let error = error.to_string();
                    //     let new_request = res.request().clone();

                    //     /* EXTRACT THE BODY OF RESPONSE */
                    //     let _body_data =
                    //         match str::from_utf8(&body::to_bytes(res.into_body()).await?) {
                    //             Ok(str) => str,
                    //             Err(_) => "Unknown",
                    //         };

                    //     let mut errors = HashMap::new();
                    //     errors.insert("general".to_string(), vec![error]);

                    //     // let new_response = match ErrorResponse::new(&false, errors) {
                    //     //     Ok(response) => HttpResponseBuilder::new(StatusCode::BAD_REQUEST)
                    //     //         .insert_header((header::CONTENT_TYPE, "application/json")),
                    //     //         // .body(serde_json::to_string(&response).unwrap()),
                    //     //     Err(_error) => HttpResponseBuilder::new(StatusCode::BAD_REQUEST)
                    //     //         .insert_header((header::CONTENT_TYPE, "application/json"))
                    //     //         // .body("An unknown error occurred."),
                    //     // };
                    //     let new_response = HttpResponseBuilder::new(StatusCode::OK).finish();

                    //     Ok(ServiceResponse::new(new_request, new_response))
                    // }
                    panic!()
                }
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
