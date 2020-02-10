use async_trait::async_trait;
use futures::stream::Stream;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

mod proto;

pub struct Error;

pub trait Service<Request> {
    /// Responses given by the service
    type Response;

    /// Returns `Ready` when the service is able to process requests.
    ///
    /// If the service is at capacity, then `NotReady` is returned and the task
    /// is notified when the service becomes ready again. This function is
    /// expected to be called while on a task.
    ///
    /// This is a **best effort** implementation. False positives are permitted.
    /// It is permitted for the service to return `Ready` from a `poll_ready`
    /// call and the next invocation of `call` results in an error.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>>;

    /// Process the request and return the response asynchronously.
    ///
    /// This function is expected to be callable off task. As such,
    /// implementations should take care to not call `poll_ready`. If the
    /// service is at capacity and the request is unable to be handled, the
    /// returned `Future` should resolve to an error.
    ///
    /// Calling `call` without calling `poll_ready` is permitted. The
    /// implementation must be resilient to this fact.
    fn call(
        &mut self,
        req: Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Response, Error>> + Send + Sync + 'static>>;
}

impl<Request, T> Service<Request> for T
where
    T: tower::Service<Request, Error = Error>,
    T::Future: Future<Output = Result<T::Response, T::Error>> + Send + Sync + 'static,
{
    type Response = T::Response;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        self.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Pin<Box<dyn Future<Output = Result<Self::Response, Error>> + Send + Sync + 'static>>
    {
        Box::pin(self.call(req))
    }
}

#[async_trait]
trait MethodPerEndpoint {
    async fn unary_unary(
        &mut self,
        req: impl tonic::IntoRequest<unary_unary::Request>,
    ) -> Result<tonic::Response<unary_unary::Response>, tonic::Status>;

    async fn unary_streaming(
        &mut self,
        req: impl tonic::IntoRequest<unary_streaming::Request>,
    ) -> Result<tonic::Response<tonic::codec::Streaming<unary_streaming::Response>>, tonic::Status>;

    async fn streaming_unary(
        &mut self,
        req: impl tonic::IntoStreamingRequest<Message = streaming_unary::Request>,
    ) -> Result<tonic::Response<streaming_unary::Response>, tonic::Status>;

    async fn streaming_streaming(
        &mut self,
        req: impl tonic::IntoStreamingRequest<Message = streaming_streaming::Request>,
    ) -> Result<tonic::Response<tonic::codec::Streaming<streaming_streaming::Response>>, tonic::Status>;
}

trait ServicePerEndpoint {
    fn unary_unary(
        &mut self,
    ) -> Box<
        dyn Service<
            tonic::Request<unary_unary::Request>,
            Response = Result<tonic::Response<unary_unary::Response>, tonic::Status>,
        >,
    >;
    fn unary_streaming(
        &mut self,
    ) -> Box<
        dyn Service<
            tonic::Request<unary_streaming::Request>,
            Response = Result<tonic::Response<tonic::codec::Streaming<unary_streaming::Response>>, tonic::Status>,
        >,
    >;
    fn streaming_unary(
        &mut self,
    ) -> Box<
        dyn Service<
            tonic::Request<
                Box<dyn Stream<Item = streaming_unary::Request> + Send + Sync + 'static>,
            >,
            Response = Result<tonic::Response<streaming_unary::Response>, tonic::Status>,
        >,
    >;
    fn streaming_streaming(
        &mut self,
    ) -> Box<
        dyn Service<
            tonic::Request<
                Box<dyn Stream<Item = streaming_streaming::Request> + Send + Sync + 'static>,
            >,
            Response = Result<tonic::Response<tonic::codec::Streaming<streaming_streaming::Response>>, tonic::Status>,
        >,
    >;
}

trait ServicePerEndpointToMethodPerEndpoint<S, M>
where
    S: ServicePerEndpoint,
    M: MethodPerEndpoint,
{
    fn to_method_per_endpoint(
        &self,
        service_per_endpoint: S,
    ) -> M;
}

pub mod unary_unary {
    use crate::proto::tonic_playground::*;

    pub struct Request(Foo);
    pub struct Response(Bar);
}

pub mod unary_streaming {
    use crate::proto::tonic_playground::*;

    pub struct Request(Foo);
    pub struct Response(Bar);
}

pub mod streaming_unary {
    use crate::proto::tonic_playground::*;

    pub struct Request(Foo);
    pub struct Response(Bar);
}

pub mod streaming_streaming {
    use crate::proto::tonic_playground::*;

    pub struct Request(Foo);
    pub struct Response(Bar);
}

#[cfg(test)]
mod tests {}
