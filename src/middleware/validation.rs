use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web::dev::{ServiceResponse, Transform};
use futures_util::future::{ok, Ready};
use std::task::{Context, Poll};

pub struct PhoneValidation;

impl<S, B> Transform<S, ServiceRequest> for PhoneValidation
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = PhoneValidationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(PhoneValidationMiddleware { service })
    }
}

pub struct PhoneValidationMiddleware<S> {
    service: S,
}

impl<S, B> actix_web::dev::Service<ServiceRequest> for PhoneValidationMiddleware<S>
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = S::Future;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Add phone number validation logic here if needed
        self.service.call(req)
    }
}
