use crate::{
    app::sidebar::SidebarLayout,
    error_template::{AppError, ErrorTemplate},
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod sidebar;

#[derive(Clone)]
struct GlobalClientState {
    font_awesome_kit_id: String,
}

impl GlobalClientState {
    fn new() -> Self {
        Self {
            font_awesome_kit_id: std::env!("FONT_AWESOME_KIT_ID").to_owned(),
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // Provides Query Client for entire app.
    leptos_query::provide_query_client();

    provide_context(GlobalClientState::new());
    let state = expect_context::<GlobalClientState>();

    view! {
        <Stylesheet id="leptos" href="/pkg/status-leptos.css"/>

        <Stylesheet
            id="font-awesome"
            href=format!("https://kit.fontawesome.com/{}.css", state.font_awesome_kit_id)
        />

        // sets the document title
        <Title text="Status - Uptime Monitoring by coreyja"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <SidebarLayout>
                <Routes>
                    <Route path="" view=HomePage/>
                </Routes>
            </SidebarLayout>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
    }
}
