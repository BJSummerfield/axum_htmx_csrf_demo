use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_csrf::CsrfToken;
use http_body_util::BodyExt;
use std::collections::HashMap;
use tower_sessions::Session;

// Define the Auth struct, currently empty as it only contains static methods.
pub struct Auth {}

// Custom error type to encapsulate various error scenarios in the middleware.
#[derive(Debug)]
enum AuthError {
    InternalServerError,
    Unauthorized,
    BadRequest,
}

// Implement conversion from AuthError to StatusCode.
// This allows our middleware to return a result with StatusCode directly from AuthError.
impl From<AuthError> for StatusCode {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::Unauthorized => StatusCode::UNAUTHORIZED,
            AuthError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }
}

// Implement the Auth struct.
impl Auth {
    // CSRF middleware function to validate CSRF tokens in POST requests.
    pub async fn csrf_middleware(
        token: CsrfToken,
        session: Session,
        method: Method,
        request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        // Process only for POST requests.
        let new_request = if method == Method::POST {
            // Destructure the request into parts and body.
            let (parts, body) = request.into_parts();
            // Collect the bytes from the body.
            let bytes = Auth::collect_body_bytes(body).await?;
            // Parse the form data from the bytes.
            let form_data = Auth::parse_form_data(&bytes)?;
            // Validate the CSRF tokens.
            Auth::validate_tokens(&token, &session, &form_data)?;

            // Reconstruct the request with the collected body.
            Request::from_parts(parts, Body::from(bytes))
        } else {
            // If not a POST request, pass the original request.
            request
        };

        // Pass the (possibly modified) request to the next middleware and return its response.
        Ok(next.run(new_request).await)
    }

    // Asynchronously collect bytes from the request body.
    async fn collect_body_bytes(body: Body) -> Result<Vec<u8>, AuthError> {
        body.collect()
            .await
            .map_err(|_| AuthError::InternalServerError)
            .and_then(|b| Ok(b.to_bytes().to_vec()))
    }

    // Parse URL-encoded form data from the request body bytes.
    fn parse_form_data(bytes: &[u8]) -> Result<HashMap<String, String>, AuthError> {
        serde_urlencoded::from_bytes(bytes).map_err(|_| AuthError::BadRequest)
    }

    // Validate the CSRF tokens from the form and the session.
    fn validate_tokens(
        token: &CsrfToken,
        session: &Session,
        form_data: &HashMap<String, String>,
    ) -> Result<(), AuthError> {
        // Retrieve the authenticity token from the form data.
        let form_token = form_data
            .get("authenticity_token")
            .ok_or(AuthError::BadRequest)?;
        // Retrieve the authenticity token from the session.
        let session_token = session
            .get::<String>("authenticity_token")
            .map_err(|_| AuthError::InternalServerError)?
            .ok_or(AuthError::Unauthorized)?;

        // Compare the tokens and validate their authenticity.
        if form_token != &session_token
            || token.verify(form_token).is_err()
            || token.verify(&session_token).is_err()
        {
            // If tokens don't match or are invalid, return an unauthorized error.
            return Err(AuthError::Unauthorized);
        }

        // Return Ok if all validations pass.
        Ok(())
    }
}
