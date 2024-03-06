use std::rc::Rc;

use leptos::*;

#[component]
pub fn WaitForOk<T: Clone + 'static, IV: IntoView + 'static, C: Fn(Signal<T>) -> IV + 'static>(
    thing: Signal<Option<Result<T, leptos::ServerFnError>>>,
    #[prop(optional, into)] loading: ViewFn,
    #[prop(optional, into)] onError: ViewFn,
    children: C,
) -> impl IntoView {
    let children = Rc::new(children);
    view! {
        <WaitFor
            thing=thing
            fallback=loading
            children=move |thing| {
                let onError = onError.clone();
                let children = Rc::clone(&children);
                view! {
                    {move || {
                        match thing.get() {
                            Ok(thing) => children(Signal::derive(move || thing.clone())).into_view(),
                            Err(_) => onError.run(),
                        }
                    }}
                }
            }
        />
    }
}

#[component]
pub fn WaitFor<T: Clone + 'static, C: Fn(Signal<T>) -> CIV + 'static, CIV: IntoView + 'static>(
    thing: Signal<Option<T>>,
    #[prop(optional, into)] fallback: ViewFn,
    children: C,
) -> impl IntoView {
    let children = Rc::new(children);

    let children_for_view = Rc::clone(&children);
    // let rendered_fallback = fallback.run().into_view();
    view! {
        <Suspense children=Rc::new(move || {
            let fallback = fallback.clone();
            let children_for_view = Rc::clone(&children_for_view);
            view! {
                {move || match thing.get() {
                    Some(thing) => {
                        children_for_view(Signal::derive(move || thing.clone())).into_view()
                    }
                    None => fallback.clone().run().into_view(),
                }}
            }
        })/>
    }
}
