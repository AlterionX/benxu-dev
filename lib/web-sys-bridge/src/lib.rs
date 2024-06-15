#![no_std]

mod iter {
    use web_sys::{Node, NodeList};

    /// Implements the [`Iterator`] interface for a [`NodeList`](web_sys::NodeList).
    pub struct NodeListIter<'a> {
        /// A reference to the [`NodeList`](web_sys::NodeList) being iterated over.
        nodes: &'a web_sys::NodeList,
        /// Cached length.
        len: u32,
        /// The index of the next element for the iterator to go over.
        next_idx: u32,
    }
    impl<'a> NodeListIter<'a> {
        /// Creates a new [`NodeListIter`] from a [`NodeList`](web_sys::NodeList).
        fn new(nodes: &'a NodeList) -> Self {
            NodeListIter { nodes, len: nodes.length(), next_idx: 0 }
        }
    }
    impl<'a> Iterator for NodeListIter<'a> {
        type Item = Node;
        fn next(&mut self) -> Option<Self::Item> {
            if self.next_idx >= self.len {
                return None;
            }

            let curr_idx = self.next_idx;
            self.next_idx = curr_idx + 1;
            self.nodes.item(curr_idx)
        }
    }

    pub trait NodeListIntoIterator {
        fn nl_iter(&self) -> NodeListIter<'_>;
    }

    impl NodeListIntoIterator for NodeList {
        fn nl_iter(&self) -> NodeListIter<'_> {
            NodeListIter::new(self)
        }
    }
}

mod conv {
    use wasm_bindgen::JsCast;

    use web_sys::{Node, Element, HtmlElement};

    pub trait ElConv: Sized {
        fn el(self) -> Result<Element, Self>;
        fn el_opt(self) -> Option<Element> {
            self.el().ok()
        }
    }

    pub trait HtmlElConv: Sized {
        fn html_el(self) -> Result<HtmlElement, Self>;
        fn html_el_opt(self) -> Option<HtmlElement> {
            self.html_el().ok()
        }
    }

    impl ElConv for Node {
        fn el(self) -> Result<Element, Self> {
            self.dyn_into()
        }
    }

    impl HtmlElConv for Node {
        fn html_el(self) -> Result<HtmlElement, Self> {
            self.dyn_into()
        }
    }

    impl HtmlElConv for Element {
        fn html_el(self) -> Result<HtmlElement, Self> {
            self.dyn_into()
        }
    }
}

mod css {
    use wasm_bindgen::JsValue;

    use web_sys::HtmlElement;

    fn px_str(val: i32, buffer: &mut [u8]) -> &str {
        // We're reversing later, set up px.
        buffer[0] = b'x';
        buffer[1] = b'p';

        // skip 0 & 1
        let mut idx = 2;
        let mut working_val = val.abs();
        while working_val != 0 {
            let offset = (working_val % 10) as u8;
            working_val /= 10;
            buffer[idx] = b'0' + offset;
            idx += 1;
        }
        if val < 0 {
            buffer[idx] = b'-';
            idx += 1;
        }
        let len = idx;

        // Reverse relevant potion.
        let h = &mut buffer[0..len];
        h.reverse();

        core::str::from_utf8(h).expect("ascii is valid")
    }

    pub trait SetMarginBottom {
        #[must_use]
        fn set_margin_bottom(&self, val: &str) -> Result<(), JsValue>;

        #[must_use]
        fn set_margin_bottom_px(&self, val: i32) -> Result<(), JsValue> {
            let mut buffer = [0; 50];
            let h = px_str(val, &mut buffer);

            self.set_margin_bottom(h)
        }

        #[must_use]
        fn unset_margin_bottom(&self) -> Result<(), JsValue> {
            self.set_margin_bottom("")
        }
    }

    impl SetMarginBottom for HtmlElement {
        fn set_margin_bottom(&self, val: &str) -> Result<(), JsValue> {
            self.style().set_property("margin-bottom", val)
        }
    }

    pub trait SetHeight {
        #[must_use]
        fn set_height(&self, val: &str) -> Result<(), JsValue>;

        #[must_use]
        fn set_height_px(&self, val: i32) -> Result<(), JsValue> {
            let mut buffer = [0; 50];
            let h = px_str(val, &mut buffer);

            self.set_height(h)
        }

        #[must_use]
        fn unset_height(&self) -> Result<(), JsValue> {
            self.set_height("")
        }
    }

    impl SetHeight for HtmlElement {
        fn set_height(&self, val: &str) -> Result<(), JsValue> {
            self.style().set_property("height", val)
        }
    }
}

mod closures {
    use js_sys::Function;
    use wasm_bindgen::{prelude::Closure, JsCast, convert::FromWasmAbi};

    pub trait FnConv {
        fn as_closure(self) -> Closure<dyn Fn()>;
    }

    impl<F: Fn() + 'static> FnConv for F {
        fn as_closure(self) -> Closure<dyn Fn()> {
            Closure::new(self)
        }
    }

    pub trait Fn1Conv<T> {
        fn as_closure(self) -> Closure<dyn Fn(T)>;
    }

    impl<T: FromWasmAbi + 'static, F: Fn(T) + 'static> Fn1Conv<T> for F {
        fn as_closure(self) -> Closure<dyn Fn(T)> {
            Closure::new(self)
        }
    }

    pub trait AsFunction {
        fn as_js_fn(&self) -> &Function;
    }

    impl AsFunction for Closure<dyn Fn()> {
        fn as_js_fn(&self) -> &Function {
            self.as_ref().unchecked_ref()
        }
    }

    impl<T: FromWasmAbi + 'static> AsFunction for Closure<dyn Fn(T)> {
        fn as_js_fn(&self) -> &Function {
            self.as_ref().unchecked_ref()
        }
    }
}

pub use conv::{ElConv, HtmlElConv};
pub use iter::NodeListIntoIterator;
pub use css::{SetHeight, SetMarginBottom};
pub use closures::{FnConv, Fn1Conv, AsFunction};
