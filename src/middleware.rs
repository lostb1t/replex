use hyper::header::HeaderValue;
use hyper::{Body, Method, Request, Response};

use simple_proxy::proxy::error::MiddlewareError;
use simple_proxy::proxy::middleware::MiddlewareResult::Next;
use simple_proxy::proxy::middleware::MiddlewareResult::RespondWith;
use simple_proxy::proxy::middleware::{Middleware, MiddlewareResult};
use simple_proxy::proxy::service::{ServiceContext, State};

pub struct Cors {
}

impl Cors {
    pub fn new(
    ) -> Self {
        Cors {
        }
    }

}

impl Middleware for Cors {
    fn name() -> String {
        String::from("Cors")
    }

    fn before_request(
        &mut self,
        req: &mut Request<Body>,
        _context: &ServiceContext,
        _state: &State,
    ) -> Result<MiddlewareResult, MiddlewareError> {
        // println!("{:#?}", req);
        if req.method() == Method::GET {
            let mut response: Response<Body> = Response::new(Body::from(""));
            // self.set_cors_headers(&mut response);

            return Ok(RespondWith(response));
        }
        
        Ok(Next)
    }

    fn after_request(
        &mut self,
        response: Option<&mut Response<Body>>,
        _context: &ServiceContext,
        _state: &State,
    ) -> Result<MiddlewareResult, MiddlewareError> {
        // if response.method() == Method::GET {
        //     let mut response: Response<Body> = Response::new(Body::from(""));
        //     // self.set_cors_headers(&mut response);

        //     return Ok(RespondWith(response));
        // }

        // if let Some(res) = response {
        //     self.set_cors_headers(res);
        // }
        Ok(Next)
    }
}