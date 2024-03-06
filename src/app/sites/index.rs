use leptos::{server_fn::request::Req, *};
use leptos_query::QueryResult;
use leptos_router::*;

use crate::app::{
    current_user::{self, use_current_user, CurrentUser},
    sites::get_my_sites,
    utils::WaitForOk,
};

#[component]
pub fn SitesIndex() -> impl IntoView {
    let QueryResult { data: sites, .. } =
        leptos_query::use_query(|| None, get_my_sites, Default::default());

    view! {
        <h1>My Sites</h1>

        <A href="/my/sites/new">"Create a new site"</A>

        <WaitForOk thing=sites let:sites>
            <ul class="divide-y divide-gray-100" role="list">
                <For each=sites key=|site| site.site_id let:site>
                    <li class="flex items-center justify-between gap-x-6 py-5">
                        <SiteTableRow site=site/>
                    </li>
                </For>
            </ul>
        </WaitForOk>
    }
}

fn signal_for<'t, T, InnerReturn: 't, OuterReturn: Clone>(
    value: &'t T,
    func: impl Fn(&'t T) -> &'t InnerReturn,
) -> Signal<OuterReturn>
where
    OuterReturn: From<&'t InnerReturn>,
{
    let inner = func(value);
    let outer = OuterReturn::from(inner);

    Signal::derive(move || outer.clone())
}

trait SignalFor {
    fn signal_for<'s, InnerReturn: 's, OuterReturn: Clone>(
        &'s self,
        func: impl Fn(&'s Self) -> &'s InnerReturn,
    ) -> Signal<OuterReturn>
    where
        OuterReturn: From<&'s InnerReturn>;
}

impl<T> SignalFor for T {
    fn signal_for<'s, InnerReturn: 's, OuterReturn: Clone>(
        &'s self,
        func: impl Fn(&'s Self) -> &'s InnerReturn,
    ) -> Signal<OuterReturn>
    where
        OuterReturn: From<&'s InnerReturn>,
    {
        signal_for(self, &func)
    }
}

#[component]
pub fn SiteTableRow(site: crate::app::sites::Site) -> impl IntoView {
    let name_signal: Signal<String> = site.signal_for(|s| &s.name);
    let domain_signal: Signal<String> = site.signal_for(|s| &s.domain);

    let (options_open, set_options_open) = create_signal(false);

    view! {
        <div class="min-w-0">
            <div class="flex items-start gap-x-3">
                <p class="text-sm font-semibold leading-6 text-gray-900">
                    <A href=site.href()>{name_signal}</A>
                </p>
                <p class="rounded-md whitespace-nowrap mt-0.5 px-1.5 py-0.5 text-xs font-medium ring-1 ring-inset text-green-700 bg-green-50 ring-green-600/20">
                    Complete
                </p>
            </div>

            {site
                .description
                .as_ref()
                .map(|description| {
                    view! {
                        <div class="mt-1 flex items-center gap-x-2 text-xs leading-5 text-gray-500">
                            <p class="truncate">{description}</p>
                        </div>
                    }
                })}

            <div class="mt-1 flex items-center gap-x-2 text-xs leading-5 text-gray-500">
                <p class="whitespace-nowrap">
                    Last Checked at <time datetime="2023-03-17T00:00Z">March 17, 2023</time>
                </p>
                <svg class="h-0.5 w-0.5 fill-current" viewBox="0 0 2 2">
                    <circle cy="1" cx="1" r="1"></circle>
                </svg>
                <p class="truncate">{domain_signal}</p>
            </div>
        </div>
        <div class="flex flex-none items-center gap-x-4">
            <A
                class="hidden rounded-md bg-white px-2.5 py-1.5 text-sm font-semibold text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50 sm:block"
                href=site.href()
            >
                View project
                <span class="sr-only">", " {name_signal}</span>
            </A>
            <div class="relative flex-none">
                <button
                    class="-m-2.5 block p-2.5 text-gray-500 hover:text-gray-900"
                    id="options-menu-0-button"
                    aria-expanded="false"
                    type="button"
                    aria-haspopup="true"
                    on:click=move |_| set_options_open.update(|new_value| *new_value = !*new_value)
                >
                    <span class="sr-only">Open options</span>
                    <svg class="h-5 w-5" viewBox="0 0 20 20" aria-hidden="true" fill="currentColor">
                        <path d="M10 3a1.5 1.5 0 110 3 1.5 1.5 0 010-3zM10 8.5a1.5 1.5 0 110 3 1.5 1.5 0 010-3zM11.5 15.5a1.5 1.5 0 10-3 0 1.5 1.5 0 003 0z"></path>
                    </svg>
                </button>
                <div
                    class="absolute right-0 z-10 mt-2 w-32 origin-top-right rounded-md bg-white py-2 shadow-lg ring-1 ring-gray-900/5 focus:outline-none"
                    class:hidden=move || !options_open.get()
                    tabindex="-1"
                    aria-orientation="vertical"
                    aria-labelledby="options-menu-0-button"
                    role="menu"
                >
                    <a
                        class="block px-3 py-1 text-sm leading-6 text-gray-900"
                        id="options-menu-0-item-0"
                        role="menuitem"
                        tabindex="-1"
                        href="#"
                    >
                        Edit
                        <span class="sr-only">", " {name_signal}</span>
                    </a>
                    <a
                        class="block px-3 py-1 text-sm leading-6 text-gray-900"
                        id="options-menu-0-item-2"
                        role="menuitem"
                        tabindex="-1"
                        href="#"
                    >
                        Delete
                        <span class="sr-only">", " {name_signal}</span>
                    </a>
                </div>
            </div>
        </div>
    }
}
