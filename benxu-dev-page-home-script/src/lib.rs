#![feature(type_ascription)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use std::cmp::max;

fn cast_node_to_html_ele(node: web_sys::Node) -> Result<web_sys::HtmlElement, web_sys::Node> {
    node.dyn_into()
}
fn cast_ele_to_html_ele(node: web_sys::Element) -> Result<web_sys::HtmlElement, web_sys::Element> {
    node.dyn_into()
}
struct NodeListIter<'a> {
    nodes: &'a web_sys::NodeList,
    next_idx: u32,
}
impl<'a> NodeListIter<'a> {
    fn new(nodes: &'a web_sys::NodeList) -> Self {
        NodeListIter {
            nodes,
            next_idx: 0,
        }
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
fn get_slide_and_markers() -> Result<(web_sys::Element, web_sys::NodeList, web_sys::NodeList), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let slide_container = document.query_selector(".slides")?.expect("No slide container.");
    let slides = document.query_selector_all(".slide")?;
    let slide_markers = document.query_selector_all(".slide-marker")?;
    Ok((slide_container, slides, slide_markers))
}

fn set_height(e: &web_sys::HtmlElement, height: &str) -> Result<(), JsValue> {
    e.style().set_property("height", height)
}
fn unset_height(e: &web_sys::HtmlElement) -> Result<(), JsValue> {
    e.style().set_property("height", "")
}
fn to_px_string(num: i32) -> String {
    num.to_string() + "px"
}
fn set_sizing(e: &web_sys::HtmlElement, height: i32) -> Result<(), JsValue> {
    let needed_e_size = to_px_string(height); // need to set height to integral value
    let needed_margin_size = to_px_string(-height);
    e.style().set_property("margin-bottom", needed_margin_size.as_str())?;
    e.style().set_property("height",  needed_e_size.as_str())?;
    Ok(())
}

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

fn find_active_slides(slides: &[(web_sys::HtmlElement, web_sys::HtmlElement)]) -> Vec<usize> {
    slides.iter().enumerate()
        .filter(|(_, (ref slide, ref marker))| slide.class_list().contains("active-slide") || marker.class_list().contains("active-slide-marker"))
        .map(|(idx, _)| idx)
        .collect()
}
fn set_active(&(ref slide, ref marker): &(web_sys::HtmlElement, web_sys::HtmlElement)) -> Result<(), JsValue> {
    slide.class_list().add_1("active-slide")?;
    marker.class_list().add_1("active-slide-marker")?;
    Ok(())
}
fn unset_active(&(ref slide, ref marker): &(web_sys::HtmlElement, web_sys::HtmlElement)) -> Result<(), JsValue> {
    slide.class_list().remove_1("active-slide")?;
    marker.class_list().remove_1("active-slide-marker")?;
    Ok(())
}
fn shift_active_slide<
    F: for <'a> Fn(&'a [(web_sys::HtmlElement, web_sys::HtmlElement)], &[usize]) -> &'a (web_sys::HtmlElement, web_sys::HtmlElement)
>(
    determine_active_slide: F
) -> Result<(), JsValue> {
    let (_, slides, slide_markers) = get_slide_and_markers()?;

    let slides = NodeListIter::new(&slides)
        .map(cast_node_to_html_ele)
        .collect::<Result<Vec<_>, _>>()?;
    let slide_markers = NodeListIter::new(&slide_markers)
        .map(cast_node_to_html_ele)
        .collect::<Result<Vec<_>, _>>()?;

    let slides_and_markers = slides.into_iter().zip(slide_markers.into_iter()).collect::<Vec<_>>();

    let curr_active_slide_indices = find_active_slides(&slides_and_markers);
    let next_active_slide_and_marker = determine_active_slide(&slides_and_markers, &curr_active_slide_indices);

    (curr_active_slide_indices.iter().map(|&idx| &slides_and_markers[idx]).map(unset_active).collect(): Result<(), JsValue>)?;
    set_active(next_active_slide_and_marker)
}

fn find_next_slide<'a>(slides: &'a [(web_sys::HtmlElement, web_sys::HtmlElement)], active_slide_indices: &[usize]) -> &'a (web_sys::HtmlElement, web_sys::HtmlElement) {
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
fn next_slide() -> Result<(), JsValue> {
    shift_active_slide(find_next_slide)
}

fn find_prev_slide<'a>(slides: &'a [(web_sys::HtmlElement, web_sys::HtmlElement)], active_slide_indices: &[usize]) -> &'a (web_sys::HtmlElement, web_sys::HtmlElement) {
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
fn prev_slide() -> Result<(), JsValue> {
    shift_active_slide(find_prev_slide)
}

#[wasm_bindgen]
pub fn set_slide(idx: usize) -> Result<(), JsValue> {
    shift_active_slide(|slides: &[(web_sys::HtmlElement, web_sys::HtmlElement)], _| {
        if idx < slides.len() {
            &slides[idx]
        } else {
            &slides[0]
        }
    })
}

fn sec_to_ms(seconds: i32) -> i32 {
    seconds * 1_000
}

type OptionalClosure<T> = Option<Closure<T>>;
static mut OPT_SLIDE_COLLAPSE: OptionalClosure<dyn Fn(web_sys::UiEvent) -> ()> = None;
static mut OPT_SLIDE_TIMED_ADVANCE: OptionalClosure<dyn Fn(i8)> = None;

#[wasm_bindgen(start)]
pub fn init() -> Result<(), JsValue> {
    unsafe {
        OPT_SLIDE_COLLAPSE = Some(Closure::wrap(Box::new(|_: web_sys::UiEvent| { let _ = align_slides(); }) as Box<dyn Fn(_)>));
        OPT_SLIDE_TIMED_ADVANCE = Some(Closure::wrap(Box::new(|_: i8| { next_slide().expect("no issues"); }) as Box<dyn Fn(_)>));
    }

    let window = web_sys::window().expect("no global `window` exists");

    let resize_handler = unsafe { OPT_SLIDE_COLLAPSE.as_ref().unwrap() };

    if window.document().expect("Document present").ready_state() != "loading" {
        align_slides()?;
    } else {
        window.add_event_listener_with_callback("DOMContentLoaded", resize_handler.as_ref().unchecked_ref())?;
    }
    window.add_event_listener_with_callback("resize", resize_handler.as_ref().unchecked_ref())?;

    let interval_handler = unsafe { OPT_SLIDE_TIMED_ADVANCE.as_ref().unwrap() };
    window.set_interval_with_callback_and_timeout_and_arguments_0(interval_handler.as_ref().unchecked_ref(), sec_to_ms(5))?;

    Ok(())
}

