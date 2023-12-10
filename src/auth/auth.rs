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

pub struct Auth {}

impl Auth {
    pub async fn csrf_middleware(
        token: CsrfToken,
        session: Session,
        method: Method,
        mut request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        if method == Method::POST {
            let (parts, body) = request.into_parts();

            let bytes = body
                .collect()
                .await
                .map_err(|_| {
                    eprintln!("Internal server error while collecting body");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
                .to_bytes()
                .to_vec();

            let form_data: HashMap<String, String> =
                serde_urlencoded::from_bytes(&bytes).map_err(|_| {
                    eprintln!("Error parsing form data");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            match form_data.get("authenticity_token") {
                Some(form_token) => match session.get::<String>("authenticity_token") {
                    Ok(Some(session_token)) => {
                        if form_token != &session_token {
                            eprintln!("Form token and session token mismatch");
                            return Err(StatusCode::UNAUTHORIZED);
                        }
                        if token.verify(form_token).is_err() {
                            eprintln!("Form Token verification failed");
                            return Err(StatusCode::UNAUTHORIZED);
                        }
                        if token.verify(&session_token).is_err() {
                            eprintln!(
                                "Modification of both Cookie/token OR a replay attack occured"
                            );
                            return Err(StatusCode::UNAUTHORIZED);
                        }
                    }
                    _ => {
                        eprintln!("Session token not found");
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                },
                None => {
                    eprintln!("Authenticity token missing in form data");
                    return Err(StatusCode::BAD_REQUEST);
                }
            }

            request = Request::from_parts(parts, Body::from(bytes));
        }

        Ok(next.run(request).await)
    }
}
