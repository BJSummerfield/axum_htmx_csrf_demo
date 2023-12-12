use axum::response::Html;
use maud::{html, PreEscaped};

use crate::models::{AuthenticityToken, Counter, CounterAction, RequestCounter, User};

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

    pub fn root(
        authenticity_token: String,
        username: &str,
        request_counter: RequestCounter,
        counter: Counter,
    ) -> Html<String> {
        let markup = html! {
            (maud::DOCTYPE)
            html {
                head {
                    (PreEscaped(Self::header()))
                }
                body {
                    (PreEscaped(Self::root_body(authenticity_token, username, request_counter, counter)))
                }
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
                body {
                    h1 id="banner-message" { "Enter Your Username" }
                    div id="form"   {(PreEscaped(Self::username_form(authenticity_token)))}
                    div id="request-counter"{(PreEscaped(Self::request_counter(request_counter)))}
                }

            }
        };
        Html(markup.into_string())
    }

    pub fn root_body(
        authenticity_token: String,
        username: &str,
        request_counter: RequestCounter,
        counter: Counter,
    ) -> String {
        let markup = html! {
            h1 { "Hello, " (username)"!" }
            {(PreEscaped(Self::request_counter(request_counter)))}
            {(PreEscaped(Self::counter_form(authenticity_token, counter)))}
        };
        markup.into_string()
    }

    fn request_counter(request_counter: RequestCounter) -> String {
        let markup = html! {
            {"Total requests made: " (request_counter.value)}
        };
        markup.into_string()
    }

    fn counter_form(authenticity_token: String, counter: Counter) -> String {
        let markup = html! {
            form
            hx-post="/counter"
            hx-target="body"
            {
                button
                    type="submit"
                    name="action"
                    value=(CounterAction::Decrement)
                { "-" } // Text for the decrement button

                span{(counter.value)}
                input
                    type="hidden"
                    name=(AuthenticityToken::KEY)
                    value=(authenticity_token)
                {}
                button
                    type="submit"
                    name="action"
                    value=(CounterAction::Increment)
                { "+" }  // Text for the increment button
            }
        };
        markup.into_string()
    }

    fn username_form(authenticity_token: String) -> String {
        let markup = html! {
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
                { "Submit" }
            }
        };
        markup.into_string()
    }
}
