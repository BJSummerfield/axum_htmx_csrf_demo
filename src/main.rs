use axum::{
    body::Body,
    error_handling::HandleErrorLayer,
    extract::{Form, Request},
    http::{Method, StatusCode},
    middleware::Next,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    BoxError, Router,
};
use axum_csrf::{CsrfConfig, CsrfLayer, CsrfToken};
use http_body_util::BodyExt;
use maud::{html, PreEscaped};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr};
use tower::ServiceBuilder;
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer};

const USERNAME_KEY: &str = "username";

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
struct Username {
    username: String,
}

#[tokio::main]
async fn main() {
    let session_store = MemoryStore::default();
    let csrf_config = CsrfConfig::default();
    let session_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            StatusCode::BAD_REQUEST
        }))
        .layer(
            SessionManagerLayer::new(session_store)
                .with_secure(true)
                .with_expiry(Expiry::OnSessionEnd),
        )
        .layer(CsrfLayer::new(csrf_config));

    let app = Router::new()
        .route("/submit_username", post(handle_submit_username))
        .layer(axum::middleware::from_fn(auth_middleware))
        .route("/", get(handler))
        .layer(session_service);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

pub async fn auth_middleware(
    token: CsrfToken,
    session: Session,
    method: Method,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    println!("{:?}", session);
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
                        eprintln!("Modification of both Cookie/token OR a replay attack occured");
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

async fn handle_submit_username(
    session: Session,
    Form(username_struct): Form<Username>,
) -> impl IntoResponse {
    let username = username_struct.username;

    session
        .insert(USERNAME_KEY, &username)
        .expect("Could not serialize.");

    Html(username_body(&username)).into_response()
}

async fn handler(csrf_token: CsrfToken, session: Session) -> impl IntoResponse {
    let authenticity_token = csrf_token.authenticity_token().unwrap();
    let _ = session
        .insert("authenticity_token", &authenticity_token)
        .expect("Could not serialize.");

    match session.get::<String>(USERNAME_KEY) {
        Ok(Some(username)) => (csrf_token.clone(), render_base(&username)).into_response(),
        Ok(None) => (
            csrf_token.clone(),
            render_base_no_username(authenticity_token).await,
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
fn username_body(username: &str) -> String {
    let markup = html! {
        h1 { "Hello, " (username) "!" }
    };

    markup.into_string()
}

async fn render_base_no_username(authenticity_token: String) -> Html<String> {
    let markup = html! {
        (maud::DOCTYPE)
        html {
        head {(header())}
            body {(PreEscaped(username_form_body(authenticity_token).await.into_string()))}
        }
    };
    Html(markup.into_string())
}

fn render_base(username: &str) -> Html<String> {
    let markup = html! {
        (maud::DOCTYPE)
        html {
        head {
            (header())
        }
            body {(PreEscaped(username_body(username)))}
        }
    };
    Html(markup.into_string())
}

fn header() -> PreEscaped<String> {
    let markup = html! {
        head {
            title { "Server Test" }
            script
                src="https://unpkg.com/htmx.org@1.9.9"
                integrity="sha384-QFjmbokDn2DjBjq+fM+8LUIVrAgqcNW2s0PjAxHETgRn9l4fvX31ZxDxvwQnyMOX"
                crossorigin="anonymous" {}
        }
    };
    PreEscaped(markup.into_string())
}
async fn username_form_body(authenticity_token: String) -> PreEscaped<String> {
    let markup = html! {
        h1 { "Enter Your Username" }
        form
            hx-post="/submit_username"
            hx-target="body"
        {
            input
                type="text"
                name="username"
                placeholder="Username"
                required="true"
            {}
            input
                type="hidden"
                name="authenticity_token"
                value=(authenticity_token)
            {}
            button
                type="submit"
            {
                "Submit"
            }
        }
    };
    PreEscaped(markup.into_string())
}
