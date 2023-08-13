#![feature(unboxed_closures, lazy_cell, maybe_uninit_uninit_array, maybe_uninit_array_assume_init)]
#![no_std]

//! The wasm functions for managing my slides. This crate should only function in a browser.
//! Static globals are used to ensure that closures are kept in memory, while not simply forgetting
//! them due to need for reuse at times.
//!
//! An associated script, `compile.sh` will call wasm-pack with the arguments used for building the
//! server's static resources.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use web_sys::UiEvent;
use web_sys::Window;
use web_sys_bridge::AsFunction;
use web_sys_bridge::Fn1Conv;
use web_sys_bridge::FnConv;
use web_sys_bridge::HtmlElConv;
use web_sys_bridge::NodeListIntoIterator;
use web_sys_bridge::SetHeight;
use web_sys_bridge::SetMarginBottom;

fn sec_to_ms(s: i32) -> i32 {
    s * 1000
}

#[wasm_bindgen]
extern "C" {
    /// Binding to javascript's `console.log()`.
    #[wasm_bindgen(js_namespace = console)]
    fn log(a: &str);
}

trait SlideOp {
    /// Set the sizing so that the next element will appear to sit directly behind the current element,
    /// flush to the top of the element.
    ///
    /// Returns height of element.
    fn align_next(&self) -> i32;
}

impl SlideOp for HtmlElement {
    fn align_next(&self) -> i32 {
        self.unset_height().expect("no issue unsetting height");
        let px = self.offset_height();
        self.set_height_px(px).expect("no issue setting height");
        self.set_margin_bottom_px(-px).expect("no issue setting margin bottom");
        px
    }
}

struct LazyOpt<T, F: FnOnce() -> T = fn() -> T> {
    store: Option<T>,
    maker: Option<F>,
}
impl<T, F: FnOnce() -> T> LazyOpt<T, F> {
    const fn new(f: F) -> Self {
        Self {
            store: None,
            maker: Some(f),
        }
    }

    fn get(&mut self) -> &mut T {
        if !self.store.is_some() {
            let maker = self.maker.take().expect("absent store means present maker");
            self.store = Some(maker());
        }

        self.store.as_mut().expect("store should have been created if it wasn't")
    }
}

/// Tracks specific html elements and UI state.
struct SlidesUI {
    active_slide: usize,

    window: Window,

    container: HtmlElement,

    slide_count: usize,
    slides: [Option<HtmlElement>; 20],
    markers: [Option<HtmlElement>; 20],

    next_button: HtmlElement,
    prev_button: HtmlElement,
}
impl SlidesUI {
    /// Extracts the nodes for the slides' container, slides, and slide markers, in that order.
    fn new() -> Result<Self, JsValue> {
        let window = web_sys::window().expect("`window` exists.");
        let document = window.document().expect("`document` exists in `window`.");

        let container = document.query_selector(".slides").expect("lookup to work.").expect("a slide container exists.").html_el().expect("container is an html element.");

        let mut slide_count = 0;
        let mut slides = [None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None];
        for (slide_idx, slide) in document.query_selector_all(".slide").expect("lookup to work.").nl_iter().enumerate() {
            slide_count += 1;
            slides[slide_idx] = Some(slide.html_el().expect("list of html elements"));
        }
        let mut markers = [None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None];
        for (marker_idx, marker) in document.query_selector_all(".slide-marker").expect("lookup to work.").nl_iter().enumerate() {
            markers[marker_idx] = Some(marker.html_el().expect("list of html elements"));
        }

        let next_button = document.query_selector("#slide-next").expect("lookup to work.").expect("has a next button.").html_el().expect("next is an html element.");
        let prev_button = document.query_selector("#slide-prev").expect("lookup to work.").expect("has a prev button.").html_el().expect("prev is an html element.");

        Ok(Self {
            active_slide: 0,

            window,

            container,

            slide_count,
            slides,
            markers,

            next_button,
            prev_button,
        })
    }

    /// Deactivate slides. Does not update active_slide to match.
    unsafe fn raw_deactivate_all_slides(&self) {
        for slide in self.slides.iter().filter_map(|o| o.as_ref()) {
            slide.class_list().remove_1("active-slide").expect("class removal worked");
        }
        for marker in self.markers.iter().filter_map(|o| o.as_ref()) {
            marker.class_list().remove_1("active-slide-marker").expect("class removal worked");
        }
    }

    /// Activate slides. Does not update active_slide to match.
    unsafe fn raw_activate_slide(&self, idx: usize) {
        self.slides[idx].as_ref().expect("slide exists").class_list().add_1("active-slide").expect("");
        self.markers[idx].as_ref().expect("slide exists").class_list().add_1("active-slide-marker").expect("");
    }

    /// Set the active slide to a specific slide.
    fn set_active_slide(&mut self, idx: usize) {
        self.active_slide = idx;
        unsafe {
            self.raw_deactivate_all_slides();
            self.raw_activate_slide(self.active_slide);
        }
    }

    /// Advance to the "next" slide.
    fn shift_active_slide_forward(&mut self) {
        self.set_active_slide((self.active_slide + 1) % self.slide_count);
    }

    /// Advance to the "previous" slide.
    fn shift_active_slide_backward(&mut self) {
        self.set_active_slide((self.active_slide + self.slide_count - 1) % self.slide_count);
    }

    /// Aligns slides so that all slides match the top of the slide deck. This way, they appear to be
    /// all in the same location.
    fn align(&self) {
        let mut max_height = 0;
        for slide in self.slides.iter().filter_map(|o| o.as_ref()) {
            let slide_height = slide.align_next();
            max_height = max_height.max(slide_height);
        }

        self.container.set_height_px(max_height).expect("no problem setting height");
    }
}

static mut UI: LazyOpt<SlidesUI> = LazyOpt::new(|| SlidesUI::new().expect("it's fine"));

/// Tracks an interval, letting it be stopped and restarted easily.
struct Interval {
    handle: Option<i32>,
    closure: Closure<dyn Fn()>,
}

impl Interval {
    fn new() -> Self {
        Self {
            handle: None,
            closure: (|| unsafe {
                UI.get().shift_active_slide_forward();
            }).as_closure()
        }
    }

    unsafe fn start(&mut self) {
        self.handle = Some(UI.get().window.set_interval_with_callback_and_timeout_and_arguments_0(
            self.closure.as_ref().unchecked_ref(),
            sec_to_ms(50),
        ).expect("no issue starting interval"));
    }

    unsafe fn stop(&mut self) {
        if let Some(h) = self.handle {
            unsafe {UI.get()}.window.clear_interval_with_handle(h);
        }
    }

    fn stop_if_present_then_start(&mut self) {
        unsafe {
            self.stop();
            self.start();
        }
    }
}

static mut AUTO_ADVANCE_INTERVAL: LazyOpt<Interval> = LazyOpt::new(|| Interval::new());

/// Stores and tracks closures that are in flight.
struct SlideClosures {
    marker_functions: [Closure<dyn Fn()>; 20],

    next_function: Closure<dyn Fn(UiEvent)>,
    prev_function: Closure<dyn Fn(UiEvent)>,

    realign_function: Closure<dyn Fn()>,
}

impl SlideClosures {
    fn new() -> Self {
        let mut a = core::mem::MaybeUninit::uninit_array();
        for slide_idx in 0..20 {
            a[slide_idx] = core::mem::MaybeUninit::new(Closure::new(move || unsafe {
                UI.get().set_active_slide(slide_idx);
            }));
        }

        let marker_functions = unsafe { core::mem::MaybeUninit::array_assume_init(a) };

        Self {
            marker_functions,
            next_function: (|_: UiEvent| {
                unsafe { AUTO_ADVANCE_INTERVAL.get() }.stop_if_present_then_start();
                unsafe { UI.get() }.shift_active_slide_forward();
            }).as_closure(),
            prev_function: (|_: UiEvent| {
                unsafe { AUTO_ADVANCE_INTERVAL.get() }.stop_if_present_then_start();
                unsafe { UI.get() }.shift_active_slide_backward();
            }).as_closure(),

            realign_function: (|| {
                unsafe { UI.get() }.align();
            }).as_closure()
        }
    }

    fn attach_to_ui(&self, ui: &mut SlidesUI) {
        if ui.window.document().expect("Document present").ready_state() != "loading" {
            ui.align();
        } else {
            ui.window.add_event_listener_with_callback(
                "DOMContentLoaded",
                self.realign_function.as_js_fn(),
            ).expect("no issue setting listener");
        }
        ui.window.add_event_listener_with_callback("resize", self.realign_function.as_js_fn()).expect("no issue");

        for (idx, slide) in ui.markers.iter().filter_map(|a| a.as_ref()).enumerate() {
            slide.set_onclick(Some(self.marker_functions[idx].as_js_fn()));
        }
        ui.next_button.set_onclick(Some(self.next_function.as_js_fn()));
        ui.prev_button.set_onclick(Some(self.prev_function.as_js_fn()));
    }
}

static mut CLOSURES: LazyOpt<SlideClosures> = LazyOpt::new(|| SlideClosures::new());

/// Binds all relevant callbacks and listeners in the home page. Triggered when the wasm module is
/// loaded.
#[wasm_bindgen(start)]
pub fn init() -> Result<(), JsValue> {
    unsafe {AUTO_ADVANCE_INTERVAL.get()}.stop_if_present_then_start();
    unsafe {CLOSURES.get()}.attach_to_ui(unsafe {UI.get()});

    Ok(())
}
