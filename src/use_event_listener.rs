use crate::core::ElementMaybeSignal;
use leptos::ev::EventDescriptor;
use leptos::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

/// Use EventListener with ease.
/// Register using [addEventListener](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener) on mounted,
/// and [removeEventListener](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/removeEventListener) automatically on cleanup.
///
/// ## Usage
///
/// ```
/// # use leptos::*;
/// # use leptos::ev::visibilitychange;
/// # use leptos_use::use_event_listener;
/// #
/// # #[component]
/// # fn Demo(cx: Scope) -> impl IntoView {
/// use_event_listener(cx, document(), visibilitychange, |evt| {
///     log!("{:?}", evt);
/// });
/// #    view! { cx, }
/// # }
/// ```
///
/// You can also pass a [`NodeRef`](https://docs.rs/leptos/latest/leptos/struct.NodeRef.html) as the event target, [`use_event_listener`] will unregister the previous event and register
/// the new one when you change the target.
///
/// ```
/// # use leptos::*;
/// # use leptos::ev::click;
/// # use leptos_use::use_event_listener;
/// #
/// # #[component]
/// # fn Demo(cx: Scope) -> impl IntoView {
/// let element = create_node_ref(cx);
///
/// use_event_listener(cx, element, click, |evt| {
///     log!("click from element {:?}", event_target::<web_sys::HtmlDivElement>(&evt));
/// });
///
/// let (cond, set_cond) = create_signal(cx, true);
///
/// view! { cx,
///     <Show
///         when=move || cond()
///         fallback=move |cx| view! { cx, <div node_ref=element>"Condition false"</div> }
///     >
///         <div node_ref=element>"Condition true"</div>
///     </Show>
/// }
/// # }
/// ```
///
/// You can also call the returned to unregister the listener.
///
/// ```
/// # use leptos::*;
/// # use leptos::ev::keydown;
/// # use web_sys::KeyboardEvent;
/// # use leptos_use::use_event_listener;
/// #
/// # #[component]
/// # fn Demo(cx: Scope) -> impl IntoView {
/// let cleanup = use_event_listener(cx, document().body(), keydown, |evt: KeyboardEvent| {
///     log!("{}", &evt.key());
/// });
///
/// cleanup();
/// #
/// #    view! { cx, }
/// # }
/// ```
///
/// Note if your components also run in SSR (Server Side Rendering), you might get errors
/// because DOM APIs like document and window are not available outside of the browser.
/// To avoid that you can put the logic inside a
/// [`create_effect`](https://docs.rs/leptos/latest/leptos/fn.create_effect.html) hook
/// which only runs client side.
pub fn use_event_listener<Ev, El, T, F>(
    cx: Scope,
    target: El,
    event: Ev,
    handler: F,
) -> Box<dyn Fn()>
where
    Ev: EventDescriptor + 'static,
    (Scope, El): Into<ElementMaybeSignal<T, web_sys::EventTarget>>,
    T: Into<web_sys::EventTarget> + Clone + 'static,
    F: FnMut(<Ev as EventDescriptor>::EventType) + 'static,
{
    use_event_listener_with_options(
        cx,
        target,
        event,
        handler,
        web_sys::AddEventListenerOptions::new(),
    )
}

/// Version of [`use_event_listener`] that takes `web_sys::AddEventListenerOptions`. See the docs for [`use_event_listener`] for how to use.
pub fn use_event_listener_with_options<Ev, El, T, F>(
    cx: Scope,
    target: El,
    event: Ev,
    handler: F,
    options: web_sys::AddEventListenerOptions,
) -> Box<dyn Fn()>
where
    Ev: EventDescriptor + 'static,
    (Scope, El): Into<ElementMaybeSignal<T, web_sys::EventTarget>>,
    T: Into<web_sys::EventTarget> + Clone + 'static,
    F: FnMut(<Ev as EventDescriptor>::EventType) + 'static,
{
    let event_name = event.name();
    let closure_js = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>).into_js_value();

    let closure = closure_js.clone();
    let cleanup_fn = move |element: &web_sys::EventTarget| {
        let _ = element
            .remove_event_listener_with_callback(&event_name, closure.as_ref().unchecked_ref());
    };
    let cleanup = cleanup_fn.clone();

    let event_name = event.name();

    let signal = (cx, target).into();

    let element = signal.get_untracked();

    let cleanup_prev_element = if let Some(element) = element {
        let element = element.into();

        _ = element.add_event_listener_with_callback_and_add_event_listener_options(
            &event_name,
            closure_js.as_ref().unchecked_ref(),
            &options,
        );

        let clean = cleanup.clone();
        Rc::new(RefCell::new(Box::new(move || {
            clean(&element);
        }) as Box<dyn Fn()>))
    } else {
        Rc::new(RefCell::new(Box::new(move || {}) as Box<dyn Fn()>))
    };

    let cleanup_prev_el = Rc::clone(&cleanup_prev_element);
    let closure = closure_js;
    create_effect(cx, move |_| {
        cleanup_prev_el.borrow()();

        let element = signal.get();

        if let Some(element) = element {
            let element = element.into();

            _ = element.add_event_listener_with_callback_and_add_event_listener_options(
                &event_name,
                closure.as_ref().unchecked_ref(),
                &options,
            );

            let clean = cleanup.clone();
            let _ = cleanup_prev_el.replace(Box::new(move || {
                clean(&element);
            }) as Box<dyn Fn()>);
        } else {
            let _ = cleanup_prev_el.replace(Box::new(move || {}) as Box<dyn Fn()>);
        }
    });

    let cleanup_fn = move || cleanup_prev_element.borrow()();
    on_cleanup(cx, cleanup_fn.clone());

    Box::new(cleanup_fn)
}
