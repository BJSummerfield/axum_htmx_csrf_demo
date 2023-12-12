use crate::models::{AuthenticityToken, Counter, CounterAction, CounterForm, RequestCounter, User};
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
        csrf_token: CsrfToken,
        session: Session,
        request_counter: RequestCounter,
        counter: Counter,
        Form(username_struct): Form<User>,
    ) -> impl IntoResponse {
        let username = username_struct.username;
        let authenticity_token = csrf_token.authenticity_token().unwrap();

        session
            .insert(&User::KEY, &username)
            .expect("Could not serialize.");
        Html(Render::root_body(
            authenticity_token,
            &username,
            request_counter,
            counter,
        ))
        .into_response()
    }

    pub async fn root(
        csrf_token: CsrfToken,
        session: Session,
        request_counter: RequestCounter,
        counter: Counter,
    ) -> impl IntoResponse {
        let authenticity_token = csrf_token.authenticity_token().unwrap();

        session
            .insert(&AuthenticityToken::KEY, &authenticity_token)
            .expect("Could not serialize.");

        match session.get::<String>(&User::KEY) {
            Ok(Some(username)) => (
                csrf_token.clone(),
                Render::root(authenticity_token, &username, request_counter, counter),
            )
                .into_response(),
            Ok(None) => (
                csrf_token.clone(),
                Render::root_no_username(authenticity_token, request_counter),
            )
                .into_response(),
            Err(e) => {
                eprintln!("Error: {}", e);
                (
                    csrf_token,
                    Render::root_no_username(authenticity_token, request_counter),
                )
                    .into_response()
            }
        }
    }

    pub async fn counter(
        csrf_token: CsrfToken,
        session: Session,
        request_counter: RequestCounter,
        mut counter: Counter,
        Form(counter_form): Form<CounterForm>,
    ) -> impl IntoResponse {
        match counter_form.action {
            CounterAction::Increment => {
                counter.value += 1; // Increment the counter value
            }
            CounterAction::Decrement => {
                counter.value -= 1; // Decrement the counter value
            }
        }

        //Generate new keys
        let authenticity_token = csrf_token.authenticity_token().unwrap();
        let username = session
            .get::<String>(&User::KEY)
            .expect("Could not deserialize.")
            .unwrap_or_default();

        session
            .insert(&AuthenticityToken::KEY, &authenticity_token)
            .expect("Could not serialize.");
        // Insert the updated counter into the session
        session
            .insert(Counter::KEY, &counter)
            .expect("Could not serialize.");

        Render::root(authenticity_token, &username, request_counter, counter)
    }
}
