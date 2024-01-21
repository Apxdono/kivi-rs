use ureq::{Error, Middleware, Request, Response};

/// Token header authentication [`Middleware`] for [`Request`].
pub struct TokenAuthHeaderMiddleware {
    header: String,
    token: Option<String>,
}

/// Format a Basic auth string
pub fn basic_auth(token: &String) -> String {
    format!("Basic {}", token)
}

impl TokenAuthHeaderMiddleware {
    pub fn new(header: String, token: Option<String>) -> Self {
        Self { header, token }
    }
}

impl Middleware for TokenAuthHeaderMiddleware {
    /// [`Middleware`] implementation adds header only when token value exists.
    fn handle(&self, request: Request, next: ureq::MiddlewareNext) -> Result<Response, Error> {
        let req: Request = match &self.token {
            Some(token) => request.set(self.header.as_str(), token.as_str()),
            _ => request,
        };
        next.handle(req)
    }
}
