use crate::models::{AuthenticityToken, User};
use crate::render::Render;
use axum::{
    extract::Form,
    response::{Html, IntoResponse},
};
use axum_csrf::CsrfToken;
use tower_sessions::Session;

pub struct Handle {}

impl Handle {
    pub async fn username(
        session: Session,
        Form(username_struct): Form<User>,
    ) -> impl IntoResponse {
        let username = username_struct.username;
        session
            .insert(&User::key(), &username)
            .expect("Could not serialize.");
        Html(Render::root_body(&username)).into_response()
    }

    pub async fn root(csrf_token: CsrfToken, session: Session) -> impl IntoResponse {
        let authenticity_token = csrf_token.authenticity_token().unwrap();
        let _ = session
            .insert(&AuthenticityToken::key(), &authenticity_token)
            .expect("Could not serialize.");
        match session.get::<String>(&User::key()) {
            Ok(Some(username)) => (csrf_token.clone(), Render::root(&username)).into_response(),
            Ok(None) => (
                csrf_token.clone(),
                Render::root_no_username(authenticity_token),
            )
                .into_response(),
            Err(e) => {
                eprintln!("Error: {}", e);
                (csrf_token, Render::root_no_username(authenticity_token)).into_response()
            }
        }
    }
}
