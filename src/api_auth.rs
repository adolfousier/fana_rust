use std::future::{ready, Ready};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, http::StatusCode,
    body::{BoxBody, EitherBody},
};
use futures::future::LocalBoxFuture;
use std::task::{Context, Poll};
use std::env;
use dotenv::dotenv;

pub struct ApiKey;

impl<S, B> Transform<S, ServiceRequest> for ApiKey
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Transform = ApiKeyMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiKeyMiddleware { service }))
    }
}

pub struct ApiKeyMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ApiKeyMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        dotenv().ok();
        let api_key = env::var("API_KEY").expect("API_KEY not set");

        let bearer_token = req
            .headers()
            .get("Authorization")
            .and_then(|header| header.to_str().ok())
            .and_then(|header| header.strip_prefix("Bearer "))
            .map(String::from);

        if let Some(token) = bearer_token {
            if token == api_key {
                let fut = self.service.call(req);
                Box::pin(async move {
                    let res: ServiceResponse<B> = fut.await?;
                    Ok(res.map_into_left_body())
                })
            } else {
                Box::pin(async move {
                    let (http_req, _payload) = req.into_parts();
                    let res = HttpResponse::new(StatusCode::UNAUTHORIZED);
                    Ok(ServiceResponse::new(http_req, res).map_into_right_body())
                })
            }
        } else {
            Box::pin(async move {
                let (http_req, _payload) = req.into_parts();
                let res = HttpResponse::new(StatusCode::UNAUTHORIZED);
                Ok(ServiceResponse::new(http_req, res).map_into_right_body())
            })
        }
    }
}

