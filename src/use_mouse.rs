use crate::core::{ElementMaybeSignal, Position};
use crate::use_event_listener_with_options;
use default_struct_builder::DefaultBuilder;
use leptos::ev::{dragover, mousemove, touchend, touchmove, touchstart};
use leptos::*;
use std::marker::PhantomData;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::AddEventListenerOptions;

/// Reactive mouse position
///
/// ## Demo
///
/// [Link to Demo](https://github.com/Synphonyte/leptos-use/tree/main/examples/use_mouse)
///
/// ## Basic Usage
///
/// ```
/// # use leptos::*;
/// # use leptos_use::{use_mouse, UseMouseReturn};
/// #
/// # #[component]
/// # fn Demo(cx: Scope) -> impl IntoView {
/// let UseMouseReturn {
///     x, y, source_type, ..
/// } = use_mouse(cx);
/// # view! { cx, }
/// # }
/// ```
///
/// Touch is enabled by default. To only detect mouse changes, set `touch` to `false`.
/// The `dragover` event is used to track mouse position while dragging.
///
/// ```
/// # use leptos::*;
/// # use leptos_use::{use_mouse_with_options, UseMouseOptions, UseMouseReturn};
/// #
/// # #[component]
/// # fn Demo(cx: Scope) -> impl IntoView {
/// let UseMouseReturn {
///     x, y, ..
/// } = use_mouse_with_options(
///     cx,
///     UseMouseOptions::default().touch(false)
/// );
/// # view! { cx, }
/// # }
/// ```
///
/// ## Custom Extractor
///
/// It's also possible to provide a custom extractor to get the position from the events.
///
/// ```
/// # use leptos::*;
/// use web_sys::MouseEvent;
/// use leptos_use::{use_mouse_with_options, UseMouseOptions, UseMouseReturn, UseMouseEventExtractor, UseMouseCoordType};
///
/// #[derive(Clone)]
/// struct MyExtractor;
///
/// impl UseMouseEventExtractor for MyExtractor {
///     fn extract_mouse_coords(&self, event: &MouseEvent) -> Option<(f64, f64)> {
///         Some((event.offset_x() as f64, event.offset_y() as f64))
///     }
///
///     // don't implement fn extract_touch_coords to ignore touch events
/// }
///
/// #[component]
/// fn Demo(cx: Scope) -> impl IntoView {
///     let element = create_node_ref(cx);
///
///     let UseMouseReturn {
///         x, y, source_type, ..
///     } = use_mouse_with_options(
///         cx,
///         UseMouseOptions::default()
///             .target(element)
///             .coord_type(UseMouseCoordType::Custom(MyExtractor))
///     );
///     view! { cx, <div node_ref=element></div> }
/// }
/// ```
pub fn use_mouse(cx: Scope) -> UseMouseReturn {
    use_mouse_with_options(cx, Default::default())
}

/// Variant of [`use_mouse`] that accepts options. Please see [`use_mouse`] for how to use.
pub fn use_mouse_with_options<El, T, Ex>(
    cx: Scope,
    options: UseMouseOptions<El, T, Ex>,
) -> UseMouseReturn
where
    El: Clone,
    (Scope, El): Into<ElementMaybeSignal<T, web_sys::EventTarget>>,
    T: Into<web_sys::EventTarget> + Clone + 'static,
    Ex: UseMouseEventExtractor + Clone + 'static,
{
    let (x, set_x) = create_signal(cx, options.initial_value.x);
    let (y, set_y) = create_signal(cx, options.initial_value.y);
    let (source_type, set_source_type) = create_signal(cx, UseMouseSourceType::Unset);

    let coord_type = options.coord_type.clone();
    let mouse_handler = move |event: web_sys::MouseEvent| {
        let result = coord_type.extract_mouse_coords(&event);

        if let Some((x, y)) = result {
            set_x(x);
            set_y(y);
            set_source_type(UseMouseSourceType::Mouse);
        }
    };

    let handler = mouse_handler.clone();
    let drag_handler = move |event: web_sys::DragEvent| {
        let js_value: &JsValue = event.as_ref();
        handler(js_value.clone().unchecked_into::<web_sys::MouseEvent>());
    };

    let coord_type = options.coord_type.clone();
    let touch_handler = move |event: web_sys::TouchEvent| {
        let touches = event.touches();
        if touches.length() > 0 {
            let result = coord_type.extract_touch_coords(
                &touches
                    .get(0)
                    .expect("Just checked that there's at least on touch"),
            );

            if let Some((x, y)) = result {
                set_x(x);
                set_y(y);
                set_source_type(UseMouseSourceType::Touch);
            }
        }
    };

    let initial_value = options.initial_value;
    let reset = move || {
        set_x(initial_value.x);
        set_y(initial_value.y);
    };

    // TODO : event filters?

    let target = options.target;
    let mut event_listener_options = AddEventListenerOptions::new();
    event_listener_options.passive(true);

    let _ = use_event_listener_with_options(
        cx,
        target.clone(),
        mousemove,
        mouse_handler,
        event_listener_options.clone(),
    );
    let _ = use_event_listener_with_options(
        cx,
        target.clone(),
        dragover,
        drag_handler,
        event_listener_options.clone(),
    );
    if options.touch && !matches!(options.coord_type, UseMouseCoordType::Movement) {
        let _ = use_event_listener_with_options(
            cx,
            target.clone(),
            touchstart,
            touch_handler.clone(),
            event_listener_options.clone(),
        );
        let _ = use_event_listener_with_options(
            cx,
            target.clone(),
            touchmove,
            touch_handler,
            event_listener_options.clone(),
        );
        if options.reset_on_touch_ends {
            let _ = use_event_listener_with_options(
                cx,
                target,
                touchend,
                move |_| reset(),
                event_listener_options.clone(),
            );
        }
    }

    UseMouseReturn {
        x,
        y,
        set_x,
        set_y,
        source_type,
    }
}

#[derive(DefaultBuilder)]
/// Options for [`use_mouse_with_options`].
pub struct UseMouseOptions<El, T, Ex>
where
    El: Clone,
    (Scope, El): Into<ElementMaybeSignal<T, web_sys::EventTarget>>,
    T: Into<web_sys::EventTarget> + Clone + 'static,
    Ex: UseMouseEventExtractor + Clone,
{
    /// How to extract the x, y coordinates from mouse events or touches
    coord_type: UseMouseCoordType<Ex>,

    /// Listen events on `target` element. Defaults to `window`
    target: El,

    /// Listen to `touchmove` events. Defaults to `true`.
    touch: bool,

    /// Reset to initial value when `touchend` event fired. Defaults to `false`
    reset_on_touch_ends: bool,

    /// Initial values. Defaults to `{x: 0.0, y: 0.0}`.
    initial_value: Position,

    #[builder(skip)]
    _marker: PhantomData<T>,
}

impl Default for UseMouseOptions<web_sys::Window, web_sys::Window, UseMouseEventExtractorDefault> {
    fn default() -> Self {
        Self {
            coord_type: UseMouseCoordType::<UseMouseEventExtractorDefault>::default(),
            target: window(),
            touch: true,
            reset_on_touch_ends: false,
            initial_value: Position { x: 0.0, y: 0.0 },
            _marker: Default::default(),
        }
    }
}

/// Defines how to get the coordinates from the event.
#[derive(Clone)]
pub enum UseMouseCoordType<E: UseMouseEventExtractor + Clone> {
    Page,
    Client,
    Screen,
    Movement,
    Custom(E),
}

impl Default for UseMouseCoordType<UseMouseEventExtractorDefault> {
    fn default() -> Self {
        Self::Page
    }
}

/// Trait to implement if you want to specify a custom extractor
#[allow(unused_variables)]
pub trait UseMouseEventExtractor {
    /// Return the coordinates from mouse events (`Some(x, y)`) or `None`
    fn extract_mouse_coords(&self, event: &web_sys::MouseEvent) -> Option<(f64, f64)> {
        None
    }

    /// Return the coordinates from touches (`Some(x, y)`) or `None`
    fn extract_touch_coords(&self, touch: &web_sys::Touch) -> Option<(f64, f64)> {
        None
    }
}

impl<E: UseMouseEventExtractor + Clone> UseMouseEventExtractor for UseMouseCoordType<E> {
    fn extract_mouse_coords(&self, event: &web_sys::MouseEvent) -> Option<(f64, f64)> {
        match self {
            UseMouseCoordType::Page => Some((event.page_x() as f64, event.page_y() as f64)),
            UseMouseCoordType::Client => Some((event.client_x() as f64, event.client_y() as f64)),
            UseMouseCoordType::Screen => Some((event.screen_x() as f64, event.client_y() as f64)),
            UseMouseCoordType::Movement => {
                Some((event.movement_x() as f64, event.movement_y() as f64))
            }
            UseMouseCoordType::Custom(ref extractor) => extractor.extract_mouse_coords(event),
        }
    }

    fn extract_touch_coords(&self, touch: &web_sys::Touch) -> Option<(f64, f64)> {
        match self {
            UseMouseCoordType::Page => Some((touch.page_x() as f64, touch.page_y() as f64)),
            UseMouseCoordType::Client => Some((touch.client_x() as f64, touch.client_y() as f64)),
            UseMouseCoordType::Screen => Some((touch.screen_x() as f64, touch.client_y() as f64)),
            UseMouseCoordType::Movement => None,
            UseMouseCoordType::Custom(ref extractor) => extractor.extract_touch_coords(touch),
        }
    }
}

#[derive(Clone)]
pub struct UseMouseEventExtractorDefault;

impl UseMouseEventExtractor for UseMouseEventExtractorDefault {}

/// Return type of [`use_mouse`].
pub struct UseMouseReturn {
    /// X coordinate of the mouse pointer / touch
    pub x: ReadSignal<f64>,
    /// Y coordinate of the mouse pointer / touch
    pub y: ReadSignal<f64>,
    /// Sets the value of `x`. This does not actually move the mouse cursor.
    pub set_x: WriteSignal<f64>,
    /// Sets the value of `y`. This does not actually move the mouse cursor.
    pub set_y: WriteSignal<f64>,
    /// Identifies the source of the reported coordinates
    pub source_type: ReadSignal<UseMouseSourceType>,
}

/// Identifies the source of the reported coordinates
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UseMouseSourceType {
    /// coordinates come from mouse movement
    Mouse,
    /// coordinates come from touch
    Touch,
    /// Initially before any event has been recorded the source type is unset
    Unset,
}
