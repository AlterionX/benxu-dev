#![feature(type_ascription)]

//! The wasm functions for managing my slides. This crate should only function in a browser.
//! Static globals are used to ensure that closures are kept in memory, while not simply forgetting
//! them due to need for reuse at times.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use std::cmp::max;

#[wasm_bindgen]
extern "C" {
    /// Binding to javascript's `console.log()`.
    #[wasm_bindgen(js_namespace = console)]
    fn log(a: &str);
}

/// Casts a [`Node`](web_sys::Node) to an [`HtmlElement`](web_sys::HtmlElement).
fn cast_node_to_html_ele(node: web_sys::Node) -> Result<web_sys::HtmlElement, web_sys::Node> {
    node.dyn_into()
}
/// Casts an [`Element`](web_sys::Element) to an [`HtmlElement`](web_sys::HtmlElement).
fn cast_ele_to_html_ele(node: web_sys::Element) -> Result<web_sys::HtmlElement, web_sys::Element> {
    node.dyn_into()
}
/// Implements the [`Iterator`] interface for a [`NodeList`](web_sys::NodeList).
struct NodeListIter<'a> {
    /// A reference to the [`NodeList`](web_sys::NodeList) being iterated over.
    nodes: &'a web_sys::NodeList,
    /// The index of the next element for the iterator to go over.
    next_idx: u32,
}
impl<'a> NodeListIter<'a> {
    /// Creates a new [`NodeListIter`] from a [`NodeList`](web_sys::NodeList).
    fn new(nodes: &'a web_sys::NodeList) -> Self {
        NodeListIter { nodes, next_idx: 0 }
    }
}
impl<'a> Iterator for NodeListIter<'a> {
    type Item = web_sys::Node;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next_idx >= self.nodes.length() {
            None
        } else {
            let curr_idx = self.next_idx;
            self.next_idx = curr_idx + 1;
            self.nodes.item(curr_idx)
        }
    }
}

/// Extracts the nodes for the slides' container, slides, and slide markers, in that order.
fn get_slide_and_markers(
) -> Result<(web_sys::Element, web_sys::NodeList, web_sys::NodeList), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let slide_container = document
        .query_selector(".slides")?
        .expect("No slide container.");
    let slides = document.query_selector_all(".slide")?;
    let slide_markers = document.query_selector_all(".slide-marker")?;
    Ok((slide_container, slides, slide_markers))
}

/// Sets the height of an [`HtmlElement`](web_sys::HtmlElement).
fn set_height(e: &web_sys::HtmlElement, height: &str) -> Result<(), JsValue> {
    e.style().set_property("height", height)
}
/// Unsets the height of an [`HtmlElement`](web_sys::HtmlElement).
fn unset_height(e: &web_sys::HtmlElement) -> Result<(), JsValue> {
    e.style().set_property("height", "")
}
/// Formats a number into a pixel string.
fn to_px_string(num: i32) -> String {
    format!("{}{}", num, "px")
}
/// Set the sizing so that the next element will appear to sit directly behind the current element,
/// flush to the top of the element.
fn set_sizing(e: &web_sys::HtmlElement, height: i32) -> Result<(), JsValue> {
    let needed_e_size = to_px_string(height); // need to set height to integral value
    let needed_margin_size = to_px_string(-height);
    e.style()
        .set_property("margin-bottom", needed_margin_size.as_str())?;
    e.style().set_property("height", needed_e_size.as_str())?;
    Ok(())
}

/// Aligns slides so that all slides match the top of the slide deck. This way, they appear to be
/// all in the same location.
fn align_slides() -> Result<(), JsValue> {
    let (slide_container, slides, _) = get_slide_and_markers()?;

    let slide_container = cast_ele_to_html_ele(slide_container)?;
    let slides = NodeListIter::new(&slides)
        .map(cast_node_to_html_ele)
        .collect::<Result<Vec<_>, _>>()?;

    let mut max_height = 0;
    for slide in slides.iter() {
        unset_height(slide)?;
        let height = slide.offset_height();
        set_sizing(slide, height)?;
        max_height = max(height, max_height);
    }

    let container_height = to_px_string(max_height);
    set_height(&slide_container, &container_height)?;

    Ok(())
}

/// Get the slide indices of all slides with the `active-slide` css class.
fn find_active_slides(slides: &[(web_sys::HtmlElement, web_sys::HtmlElement)]) -> Vec<usize> {
    slides
        .iter()
        .enumerate()
        .filter(|(_, (ref slide, ref marker))| {
            slide.class_list().contains("active-slide")
                || marker.class_list().contains("active-slide-marker")
        })
        .map(|(idx, _)| idx)
        .collect()
}
/// Sets a slide and marker to the active state.
fn set_active(
    &(ref slide, ref marker): &(web_sys::HtmlElement, web_sys::HtmlElement),
) -> Result<(), JsValue> {
    slide.class_list().add_1("active-slide")?;
    marker.class_list().add_1("active-slide-marker")?;
    Ok(())
}
/// Sets a slide and marker to the inactive state.
fn unset_active(
    &(ref slide, ref marker): &(web_sys::HtmlElement, web_sys::HtmlElement),
) -> Result<(), JsValue> {
    slide.class_list().remove_1("active-slide")?;
    marker.class_list().remove_1("active-slide-marker")?;
    Ok(())
}
/// Change the active slides from one to another. `determine_active_slide` returns a single (slide,
/// slide_marker) pair denoting the next active slides while consuming a list of all slide and
/// marker pairs as well as the indices of all active slides.
fn shift_active_slide<F>(determine_active_slide: F) -> Result<(), JsValue>
where
    F: for<'a> Fn(
        &'a [(web_sys::HtmlElement, web_sys::HtmlElement)],
        &[usize],
    ) -> &'a (web_sys::HtmlElement, web_sys::HtmlElement),
{
    let (_, slides, slide_markers) = get_slide_and_markers()?;

    let slides = NodeListIter::new(&slides)
        .map(cast_node_to_html_ele)
        .collect::<Result<Vec<_>, _>>()?;
    let slide_markers = NodeListIter::new(&slide_markers)
        .map(cast_node_to_html_ele)
        .collect::<Result<Vec<_>, _>>()?;

    let slides_and_markers = slides
        .into_iter()
        .zip(slide_markers.into_iter())
        .collect::<Vec<_>>();

    let curr_active_slide_indices = find_active_slides(&slides_and_markers);
    let next_active_slide_and_marker =
        determine_active_slide(&slides_and_markers, &curr_active_slide_indices);

    (curr_active_slide_indices
        .iter()
        .map(|&idx| &slides_and_markers[idx])
        .map(unset_active)
        .collect(): Result<(), JsValue>)?;
    set_active(next_active_slide_and_marker)
}

/// A function to find the slide after the current active slides.
fn find_next_slide<'a>(
    slides: &'a [(web_sys::HtmlElement, web_sys::HtmlElement)],
    active_slide_indices: &[usize],
) -> &'a (web_sys::HtmlElement, web_sys::HtmlElement) {
    if active_slide_indices.len() == 0 {
        &slides[0]
    } else if active_slide_indices.len() == 1 {
        let curr_active_slide_idx = active_slide_indices[0];
        let next_active_slide_idx = (curr_active_slide_idx + 1) % slides.len();
        &slides[next_active_slide_idx]
    } else {
        unreachable!("Should never reach here. {:?}", active_slide_indices);
    }
}
/// A function to change the active slide to the next slide, wrapping around once it hits the end
/// of the slide deck.
fn next_slide() -> Result<(), JsValue> {
    shift_active_slide(find_next_slide)
}

/// A function to find the slide before the current active slides.
fn find_prev_slide<'a>(
    slides: &'a [(web_sys::HtmlElement, web_sys::HtmlElement)],
    active_slide_indices: &[usize],
) -> &'a (web_sys::HtmlElement, web_sys::HtmlElement) {
    if active_slide_indices.len() == 0 {
        &slides[0]
    } else if active_slide_indices.len() == 1 {
        let curr_active_slide_idx = active_slide_indices[0];
        let next_active_slide_idx = (curr_active_slide_idx + slides.len() - 1) % slides.len();
        &slides[next_active_slide_idx]
    } else {
        unreachable!("Should never reach here. {:?}", active_slide_indices);
    }
}
/// A function to change the active slide to the pervious slide, wrapping around once it hits the
/// beginning of the slide deck.
fn prev_slide() -> Result<(), JsValue> {
    shift_active_slide(find_prev_slide)
}

/// Sets a slide to active, deactivating all other slides.
#[wasm_bindgen]
pub fn set_slide(idx: usize) -> Result<(), JsValue> {
    shift_active_slide(
        |slides: &[(web_sys::HtmlElement, web_sys::HtmlElement)], _| {
            if idx < slides.len() {
                &slides[idx]
            } else {
                &slides[0]
            }
        },
    )
}

/// Converts seconds to milliseconds. Does not handle overflow.
fn sec_to_ms(seconds: i32) -> i32 {
    seconds * 1_000
}

/// A type for late-init globals that are also closures.
type OptionalClosure<T> = Option<Closure<T>>;

/// Static global to keep memory around for the `collapse` closure. Collapses slides into a sane
/// slide-deck like structure.
static mut OPT_SLIDE_COLLAPSE: OptionalClosure<dyn Fn(web_sys::UiEvent) -> ()> = None;

/// Static global to keep memory around for the timed slide auto-progression closure.
static mut OPT_SLIDE_TIMED_ADVANCE: OptionalClosure<dyn Fn(i8)> = None;
/// Static global to keep the timer id for debouncing the auto-progression.
static mut OPT_SLIDE_TIMED_ADVANCE_ID: Option<i32> = None;
/// Sets the timer for auto-progression of slides. 50 seconds are given to read each slide. This
/// also sets the globals [`OPT_SLIDE_TIMED_ADVANCE`] and [`OPT_SLIDE_TIMED_ADVANCE_ID`].
fn set_slide_progression_timer(window: &web_sys::Window) -> Result<(), JsValue> {
    let interval_handler = unsafe { OPT_SLIDE_TIMED_ADVANCE.as_ref().unwrap() };
    let timer_id = window.set_interval_with_callback_and_timeout_and_arguments_0(
        interval_handler.as_ref().unchecked_ref(),
        sec_to_ms(50),
    )?;

    unsafe {
        OPT_SLIDE_TIMED_ADVANCE_ID = Some(timer_id);
    }

    Ok(())
}
/// Debounces the slide auto-progression after the active slide changes. Calls
/// [`set_slide_progression_timer`] in the process.
fn reset_slide_progression_timer() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let handle = unsafe { OPT_SLIDE_TIMED_ADVANCE_ID };
    match handle {
        Some(timer_id) => window.clear_interval_with_handle(timer_id),
        _ => (),
    }
    set_slide_progression_timer(&window)?;

    Ok(())
}
/// Static global to keep closures that jump to specific slides. There are seven slides, so there
/// are seven of these closures.
static mut OPT_SLIDE_SELECT: [OptionalClosure<dyn Fn(i8)>; 7] =
    [None, None, None, None, None, None, None]; // TODO fix when enum variants become types
/// Initializes and binds a member of [`OPT_SLIDE_SELECT`] for a specific slide to the slide's
/// marker (round circle things at the bottom of the slides.
fn bind_per_marker_listener(document: &web_sys::Document, slide_idx: usize) -> Result<(), JsValue> {
    let listener = unsafe {
        OPT_SLIDE_SELECT[slide_idx] = Some(Closure::wrap(Box::new(move |_: i8| {
            set_slide(slide_idx).expect("no issues");
            reset_slide_progression_timer().expect("no issues");
        }) as Box<dyn Fn(_)>));
        OPT_SLIDE_SELECT[slide_idx].as_ref().unwrap()
    };
    let selector = format!("#slide-marker-{}", slide_idx);
    let marker = document
        .query_selector(selector.as_str())?
        .expect("Has a marker.");
    let marker = cast_ele_to_html_ele(marker)?;
    marker.add_event_listener_with_callback("click", listener.as_ref().unchecked_ref())?;
    Ok(())
}
/// Static global to keep the closure responsible for the "next" button.
static mut OPT_SLIDE_NEXT: OptionalClosure<dyn Fn(i8)> = None;
/// Initializes and binds the [`OPT_SLIDE_NEXT`] to the "next" button.
fn bind_next_slide_button(document: &web_sys::Document) -> Result<(), JsValue> {
    let listener = unsafe {
        OPT_SLIDE_NEXT = Some(Closure::wrap(Box::new(|_: i8| {
            next_slide().expect("no issues");
        }) as Box<dyn Fn(_)>));
        OPT_SLIDE_NEXT.as_ref().unwrap()
    };
    let button = document
        .query_selector("#slide-next")?
        .expect("Has a marker.");
    let button = cast_ele_to_html_ele(button)?;
    button.add_event_listener_with_callback("click", listener.as_ref().unchecked_ref())?;
    Ok(())
}
/// Static global to keep the closure responsible for the "prev" button.
static mut OPT_SLIDE_PREV: OptionalClosure<dyn Fn(i8)> = None;
/// Initializes and binds the [`OPT_SLIDE_NEXT`] to the "prev" button.
fn bind_prev_slide_button(document: &web_sys::Document) -> Result<(), JsValue> {
    let listener = unsafe {
        OPT_SLIDE_PREV = Some(Closure::wrap(Box::new(|_: i8| {
            prev_slide().expect("no issues");
        }) as Box<dyn Fn(_)>));
        OPT_SLIDE_PREV.as_ref().unwrap()
    };
    let button = document
        .query_selector("#slide-prev")?
        .expect("Has a marker.");
    let button = cast_ele_to_html_ele(button)?;
    button.add_event_listener_with_callback("click", listener.as_ref().unchecked_ref())?;
    Ok(())
}

/// Binds all relevant callbacks and listeners in the home page. Triggered when the wasm module is
/// loaded.
#[wasm_bindgen(start)]
pub fn init() -> Result<(), JsValue> {
    unsafe {
        OPT_SLIDE_COLLAPSE = Some(Closure::wrap(Box::new(|_: web_sys::UiEvent| {
            let _ = align_slides();
        }) as Box<dyn Fn(_)>));
        OPT_SLIDE_TIMED_ADVANCE = Some(Closure::wrap(Box::new(|_: i8| {
            next_slide().expect("no issues");
        }) as Box<dyn Fn(_)>));
    }

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let resize_handler = unsafe { OPT_SLIDE_COLLAPSE.as_ref().unwrap() };

    if window.document().expect("Document present").ready_state() != "loading" {
        align_slides()?;
    } else {
        window.add_event_listener_with_callback(
            "DOMContentLoaded",
            resize_handler.as_ref().unchecked_ref(),
        )?;
    }
    window.add_event_listener_with_callback("resize", resize_handler.as_ref().unchecked_ref())?;

    set_slide_progression_timer(&window)?;

    for slide_idx in 0..7 {
        bind_per_marker_listener(&document, slide_idx)?;
    }
    bind_next_slide_button(&document)?;
    bind_prev_slide_button(&document)?;

    Ok(())
}
