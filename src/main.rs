use std::net::SocketAddr;

use axum::{
    error_handling::HandleErrorLayer,
    extract::Form,
    response::{Html, IntoResponse},
    routing::{get, post},
    BoxError, Router,
};
use http::StatusCode;
use maud::{html, PreEscaped};
use serde::{Deserialize, Serialize};
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
    let session_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            StatusCode::BAD_REQUEST
        }))
        .layer(
            SessionManagerLayer::new(session_store)
                .with_secure(true)
                .with_expiry(Expiry::OnSessionEnd),
        );

    let app = Router::new()
        .route("/", get(handler))
        .route("/submit_username", post(handle_submit_username))
        .layer(session_service);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn handle_submit_username(
    session: Session,
    Form(username_struct): Form<Username>,
) -> impl IntoResponse {
    let username = username_struct.username;
    println!("{:?}", session);
    session
        .insert(USERNAME_KEY, &username)
        .expect("Could not serialize.");

    Html(username_body(&username)).into_response() // Pass the username string directly
}

async fn handler(session: Session) -> impl IntoResponse {
    println!("{:?}", session);
    match session.get::<String>(USERNAME_KEY) {
        Ok(Some(username)) => render_base(&username).into_response(),
        Ok(None) => render_base_no_username().into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

fn username_body(username: &str) -> String {
    let markup = html! {
        h1 { "Hello, " (username) "!" }
    };

    markup.into_string()
}

fn render_base_no_username() -> Html<String> {
    let markup = html! {
        (maud::DOCTYPE)
        html {
        head {(header())}
            body {(PreEscaped(username_form_body().into_string()))}
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
fn username_form_body() -> PreEscaped<String> {
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
            button
                type="submit"
            {
                "Submit"
            }
        }
    };
    PreEscaped(markup.into_string())
}
