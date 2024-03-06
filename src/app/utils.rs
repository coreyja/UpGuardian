use std::rc::Rc;

use leptos::*;

#[component]
pub fn WaitForOk<T: Clone + 'static, IV: IntoView + 'static, C: Fn(Signal<T>) -> IV + 'static>(
    thing: Signal<Option<Result<T, leptos::ServerFnError>>>,
    #[prop(optional, into)] loading: ViewFn,
    #[prop(optional, into)] onError: ViewFn,
    children: C,
) -> impl IntoView {
    view! {
        <WaitFor
            thing=thing
            fallback=loading
            children=Box::new(move |thing| {
                match thing.get() {
                    Ok(thing) => children(Signal::derive(move || thing.clone())).into_view().into(),
                    Err(_) => onError.run().into(),
                }
            })
        />
    }
}

#[component]
pub fn WaitFor<T: Clone + 'static>(
    thing: Signal<Option<T>>,
    #[prop(optional, into)] fallback: ViewFn,
    children: Box<dyn Fn(Signal<T>) -> Fragment>,
) -> impl IntoView {
    view! {
        <Suspense
            fallback=fallback.clone()
            children=Rc::new(move || {
                match thing.get() {
                    Some(thing) => children(Signal::derive(move || thing.clone())),
                    None => fallback.run().into_view().into(),
                }
            })
        />
    }
}
