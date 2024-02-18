use leptos::*;

use leptos_query::{QueryResult, RefetchFn};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarSite {
    pub id: String,
    pub name: String,
    pub href: String,
}

#[server]
pub async fn get_my_sites(_args: ()) -> Result<Vec<SidebarSite>, ServerFnError> {
    use crate::extractors::*;
    use cja::app_state::AppState as _;

    let session = extract_session()?;
    let state = extract_state()?;

    let sites = if let Some(session) = &session {
        sqlx::query!(
            r#"
  SELECT Sites.site_id, Sites.name, Sites.domain
  FROM Sites
  WHERE user_id = $1
  LIMIT 5
  "#,
            session.user_id
        )
        .fetch_all(state.db())
        .await
        .unwrap()
    } else {
        vec![]
    };

    let sites = sites
        .into_iter()
        .map(|site| SidebarSite {
            id: site.site_id.to_string(),
            name: site.name,
            href: format!("/my/sites/{}", site.site_id),
        })
        .collect();

    Ok(sites)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    user_id: String,
    session_id: String,
}

#[server]
pub async fn get_current_user(_args: ()) -> Result<Option<CurrentUser>, ServerFnError> {
    use crate::extractors::*;

    let session = extract_session()?;

    match session {
        Some(session) => Ok(Some(CurrentUser {
            user_id: session.user_id.to_string(),
            session_id: session.session_id.to_string(),
        })),
        None => Ok(None),
    }
}

pub fn use_current_user() -> QueryResult<Result<Option<CurrentUser>, ServerFnError>, impl RefetchFn>
{
    leptos_query::use_query(|| (), get_current_user, Default::default())
}

#[component]
pub fn SidebarSiteList(sites: Signal<Vec<SidebarSite>>) -> impl IntoView {
    view! {
        <ul class="-mx-2 space-y-1" role="list">
            <For
                each=sites
                key=|site| site.id.to_owned()
                children=|site| {
                    let first_char = site.name.as_str().chars().next().unwrap_or('*');
                    let name_for_view = site.name.clone();
                    view! {
                        <li>
                            <a
                                class="text-indigo-200 hover:text-white hover:bg-indigo-700 group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold"
                                href=site.href.clone()
                            >
                                <span class="flex h-6 w-6 shrink-0 items-center justify-center rounded-lg border border-indigo-400 bg-indigo-500 text-[0.625rem] font-medium text-white">
                                    {first_char}
                                </span>
                                <span class="truncate">{name_for_view}</span>
                            </a>
                        </li>
                    }
                }
            />

        </ul>
    }
}

#[component]
pub fn MobileSidebar(sites: Signal<Vec<SidebarSite>>) -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);

    view! {
        <div
            class="relative z-50 lg:hidden"
            class:hidden=move || !is_open.get()
            role="dialog"
            aria-modal="true"
        >
            <div
                class="fixed inset-0 bg-gray-900/80"
                on:click=move |_| set_is_open.set(false)
            ></div>
            <div class="fixed inset-0 flex">
                <div class="relative mr-16 flex w-full max-w-xs flex-1">
                    <div class="absolute left-full top-0 flex w-16 justify-center pt-5">
                        <button
                            class="-m-2.5 p-2.5"
                            type="button"
                            on:click=move |_| {
                                set_is_open.set(false);
                            }
                        >

                            <span class="sr-only">Close sidebar</span>
                            <svg
                                class="h-6 w-6 text-white"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="1.5"
                                aria-hidden="true"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linejoin="round"
                                    d="M6 18L18 6M6 6l12 12"
                                    stroke-linecap="round"
                                ></path>
                            </svg>
                        </button>
                    </div>
                    <div class="flex grow flex-col gap-y-5 overflow-y-auto bg-indigo-600 px-6 pb-2">
                        <div class="flex h-16 shrink-0 items-center">
                            <img
                                class="h-8 w-auto"
                                src="https://tailwindui.com/img/logos/mark.svg?color=white"
                                alt="Status by Coreyja"
                            />
                        </div>
                        <nav class="flex flex-1 flex-col">
                            <ul class="flex flex-1 flex-col gap-y-7" role="list">
                                <li>
                                    <SidebarLinks/>
                                </li>
                                <Transition>
                                    <li>
                                        <div class="text-xs font-semibold leading-6 text-indigo-200">
                                            Your Sites
                                        </div>
                                        <SidebarSiteList sites=sites/>
                                    </li>
                                </Transition>
                            </ul>
                        </nav>
                    </div>
                </div>
            </div>
        </div>
        <div class="sticky top-0 z-40 flex items-center gap-x-6 bg-indigo-600 px-4 py-4 shadow-sm sm:px-6 lg:hidden">
            <button
                class="-m-2.5 p-2.5 text-indigo-200 lg:hidden"
                type="button"
                on:click=move |_| set_is_open.set(true)
            >
                <span class="sr-only">Open sidebar</span>
                <svg
                    class="h-6 w-6"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    aria-hidden="true"
                    stroke-width="1.5"
                >
                    <path
                        stroke-linecap="round"
                        d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5"
                        stroke-linejoin="round"
                    ></path>
                </svg>
            </button>
            <div class="flex-1 text-sm font-semibold leading-6 text-white">Dashboard</div>
            <a href="#">
                <span class="sr-only">Your profile</span>
                <img
                    class="h-8 w-8 rounded-full bg-indigo-700"
                    src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&amp;ixid=eyJhcHBfaWQiOjEyMDd9&amp;auto=format&amp;fit=facearea&amp;facepad=2&amp;w=256&amp;h=256&amp;q=80"
                    alt=""
                />
            </a>
        </div>
    }
}

#[component]
pub fn SidebarLink(title: String, href: String, icon_class: String) -> impl IntoView {
    view! {
        <li>
            <a
                class="text-indigo-200 hover:text-white hover:bg-indigo-700 group rounded-md p-2 flex gap-x-3"
                href=href
            >
                <i
                    class="self-center shrink-0 text-white fa-fw fa-solid"
                    class=icon_class
                    aria-hidden="true"
                ></i>
                <span class="text-sm leading-6 font-semibold">{title}</span>
            </a>
        </li>
    }
}

#[component]
pub fn ProfileFooter() -> impl IntoView {
    let QueryResult {
        data: current_user, ..
    } = use_current_user();

    view! {
        <Transition>
            <li class="-mx-6 mt-auto">
                <a
                    class="flex items-center gap-x-4 px-6 py-3 text-sm font-semibold leading-6 text-white hover:bg-indigo-700"
                    href="#"
                >
                    {move || {
                        if let Some(Ok(Some(current_user))) = current_user.get() {
                            view! {
                                <span>{format!("Logged in as {}", current_user.user_id)}</span>
                            }
                        } else {
                            view! { <span>{"Not logged in"}</span> }
                        }
                    }}

                </a>
            </li>
        </Transition>
    }
}

#[component]
pub fn SidebarLinks() -> impl IntoView {
    view! {
        <ul class="-mx-2 space-y-1" role="list">
            <SidebarLink
                title="Dashboard".to_owned()
                href="/".to_owned()
                icon_class="fa-house".to_owned()
            />
            <SidebarLink
                title="Sites".to_owned()
                href="/my/sites".to_owned()
                icon_class="fa-t-rex".to_owned()
            />
        </ul>
    }
}

#[component]
pub fn SidebarLayout(children: Children) -> impl IntoView {
    let QueryResult { data: sites, .. } =
        leptos_query::use_query(|| (), get_my_sites, Default::default());

    let sites: Signal<_> = Signal::derive(move || match sites.get() {
        Some(Ok(sites)) => sites,
        Some(Err(_)) => {
            vec![]
        }
        None => vec![],
    });

    view! {
        <body class="h-full">
            <div>
                <MobileSidebar sites=sites/>

                <div class="hidden lg:fixed lg:inset-y-0 lg:z-50 lg:flex lg:w-72 lg:flex-col">
                    <div class="flex grow flex-col gap-y-5 overflow-y-auto bg-indigo-600 px-6">
                        <div class="flex h-16 shrink-0 items-center">
                            <a href="/">
                                <h1 class="text-white text-2xl">Status</h1>
                            </a>
                        </div>
                        <nav class="flex flex-1 flex-col">
                            <ul class="flex flex-1 flex-col gap-y-7" role="list">
                                <li>
                                    <SidebarLinks/>
                                </li>
                                <Transition>
                                    <li>
                                        <div class="text-xs font-semibold leading-6 text-indigo-200">
                                            Your Sites
                                        </div>
                                        <SidebarSiteList sites=sites/>
                                    </li>
                                </Transition>

                                <ProfileFooter/>
                            </ul>
                        </nav>
                    </div>
                </div>

                <main class="py-10 lg:pl-72">{children()}</main>
            </div>
        </body>
    }
}
