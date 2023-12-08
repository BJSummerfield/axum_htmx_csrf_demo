use std::net::SocketAddr;

use axum::{
    error_handling::HandleErrorLayer,
    extract::Form,
    response::{Html, IntoResponse},
    routing::{get, post},
    BoxError, Router,
};
use http::StatusCode;
use maud::html;
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer};

const USERNAME_KEY: &str = "username";

#[derive(Default, Deserialize, Serialize, Debug)]
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
    Form(username): Form<Username>,
) -> impl IntoResponse {
    println!("{:?}", session);
    session
        .insert(USERNAME_KEY, &username)
        .expect("Could not serialize.");
    render_username(username).into_response()
}

async fn handler(session: Session) -> impl IntoResponse {
    println!("{:?}", session);
    match session.get::<Username>(USERNAME_KEY) {
        Ok(Some(username)) => render_username(username).into_response(), // Handle the case where a username is present
        Ok(None) => render_enter_username().await.into_response(), // Handle the case where a username is not present
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(), // Simplified error handling
    }
}

fn render_username(username: Username) -> String {
    format!("Hello, {}!", username.username)
}

async fn render_enter_username() -> Html<String> {
    let markup = html! {
        (maud::DOCTYPE)
        html {
            head {
                title { "Server Test" }
                script
                    src="https://unpkg.com/htmx.org@1.9.9"
                    integrity="sha384-QFjmbokDn2DjBjq+fM+8LUIVrAgqcNW2s0PjAxHETgRn9l4fvX31ZxDxvwQnyMOX"
                    crossorigin="anonymous" {}
            }
            body {
                h1 { "Enter Your Username" }
                form
                    hx-post="/submit_username"

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

            }
        }
    };
    Html(markup.into_string())
}
