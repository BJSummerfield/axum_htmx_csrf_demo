use axum::response::Html;
use maud::{html, PreEscaped};

use crate::models::{AuthenticityToken, RequestCounter, User};

pub struct Render {}

impl Render {
    fn header() -> String {
        let markup = html! {
        (maud::DOCTYPE)
            head {
                title { "Server Test" }
                script
                    src="https://unpkg.com/htmx.org@1.9.9"
                    integrity="sha384-QFjmbokDn2DjBjq+fM+8LUIVrAgqcNW2s0PjAxHETgRn9l4fvX31ZxDxvwQnyMOX"
                    crossorigin="anonymous" {}
            }
        };
        markup.into_string()
    }

    pub fn root(username: &str, request_counter: RequestCounter) -> Html<String> {
        let markup = html! {
            (maud::DOCTYPE)
            html {
            head {
                (PreEscaped(Self::header()))
            }
                body {(PreEscaped(Self::root_body(username, request_counter)))}
            }
        };
        Html(markup.into_string())
    }

    pub fn root_no_username(
        authenticity_token: String,
        request_counter: RequestCounter,
    ) -> Html<String> {
        let markup = html! {
            (maud::DOCTYPE)
            html {
            head {(PreEscaped(Self::header()))}
                body {(PreEscaped(Self::username_form(authenticity_token,request_counter)))}
            }
        };
        Html(markup.into_string())
    }

    pub fn root_body(username: &str, request_counter: RequestCounter) -> String {
        let markup = html! {
            h1 { "Hello, " (username) "!" }
            {(PreEscaped(Self::request_counter(request_counter)))}
        };
        markup.into_string()
    }

    fn request_counter(request_counter: RequestCounter) -> String {
        let markup = html! {
            div{"Total requests made: " (request_counter.value)}
        };
        markup.into_string()
    }

    fn username_form(authenticity_token: String, request_counter: RequestCounter) -> String {
        let markup = html! {
            h1 { "Enter Your Username" }
            form
                hx-post="/username"
                hx-target="body"
            {
                input
                    type="text"
                    name=(User::KEY)
                    placeholder="Username"
                    required="true"
                {}
                input
                    type="hidden"
                    name=(AuthenticityToken::KEY)
                    value=(authenticity_token)
                {}
                button
                    type="submit"
                {
                    "Submit"
                }
            }
            {(PreEscaped(Self::request_counter(request_counter)))}
        };
        markup.into_string()
    }
}
