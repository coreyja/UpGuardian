use crate::{
    app::{
        current_user::use_current_user, sidebar::SidebarLayout, sites::index::SitesIndex,
        utils::WaitForOk,
    },
    error_template::{AppError, ErrorTemplate},
};
use leptos::*;
use leptos_meta::*;
use leptos_query::QueryResult;
use leptos_router::*;

use self::current_user::CurrentUser;

mod sidebar;

pub mod current_user;
pub mod sites;
pub mod utils;

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
pub fn RequireAuth<C: Fn(CurrentUser) -> CIV + 'static, CIV: IntoView + 'static>(
    children: C,
) -> impl IntoView {
    let QueryResult {
        data: current_user, ..
    } = use_current_user();

    view! {
        <WaitForOk
            thing=current_user
            loading=|| view! { <p>"Loading..."</p> }.into_view()
            onError=|| view! { <p>"Error loading current user"</p> }.into_view()
            children=move |current_user| {
                match current_user.get() {
                    Some(current_user) => children(current_user).into_view(),
                    None => view! { <p>"You must be logged in to view this page"</p> }.into_view(),
                }
            }
        />
    }
}

#[component]
pub fn MyPages() -> impl IntoView {
    let current_user = use_current_user();

    let user_id = Signal::derive(move || {
        if let Some(Ok(Some(current_user))) = current_user.data.get() {
            current_user.user_id.clone()
        } else {
            "Unknown".to_owned()
        }
    });

    view! {
        <Suspense>
            <h1>"My Pages"</h1>
            <p>"Welcome, " {user_id} "!"</p>

            <Outlet/>
        </Suspense>
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
                    <Route path="my" view=MyPages>
                        <Route path="sites" view=SitesIndex/>
                    </Route>
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
