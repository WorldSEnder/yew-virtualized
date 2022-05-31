use wasm_bindgen::prelude::Closure;
use web_sys::Element;

mod raw {
    use wasm_bindgen::{
        prelude::{wasm_bindgen, Closure},
        JsValue,
    };
    use web_sys::{DomRectReadOnly, Element};

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(extends = ::js_sys::Object)]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub type ResizeObserver;
        #[wasm_bindgen(constructor)]
        pub fn new(callback: &ResizeCallback) -> ResizeObserver;
        #[wasm_bindgen(method, catch)]
        pub fn disconnect(this: &ResizeObserver) -> Result<(), JsValue>;
        #[wasm_bindgen(method, catch)]
        pub fn observe(
            this: &ResizeObserver,
            element: Element, /* , options? */
        ) -> Result<(), JsValue>;
        #[wasm_bindgen(method, catch)]
        pub fn unobserve(this: &ResizeObserver, element: Element) -> Result<(), JsValue>;

        #[wasm_bindgen(extends = ::js_sys::Object)]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub type ResizeObserverEntry;
        #[wasm_bindgen(structural, method, getter)]
        pub fn target(this: &ResizeObserverEntry) -> Element;
        #[wasm_bindgen(structural, method,  getter, js_name = contentRect)]
        pub fn content_rect(this: &ResizeObserverEntry) -> DomRectReadOnly;
    }
    pub type ResizeFn = dyn FnMut(Box<[ResizeObserverEntry]>, ResizeObserver);
    pub type ResizeCallback = Closure<ResizeFn>;
}

pub struct ResizeObserver {
    closure: Option<raw::ResizeCallback>,
    observer: raw::ResizeObserver,
}

pub struct ObservedElement {
    observer: Option<raw::ResizeObserver>,
    element: Element,
}

impl ResizeObserver {
    pub fn new<F>(mut callback: F) -> ResizeObserver
    where
        F: 'static + FnMut(&[raw::ResizeObserverEntry]),
    {
        let closure = Closure::wrap(Box::new(
            move |entries: Box<[raw::ResizeObserverEntry]>, _this: raw::ResizeObserver| {
                callback(&entries)
            },
        ) as Box<raw::ResizeFn>);
        let observer = raw::ResizeObserver::new(&closure);
        Self {
            closure: Some(closure),
            observer,
        }
    }

    pub fn observe(&self, element: Element) -> ObservedElement {
        self.observer
            .observe(element.clone())
            .expect("failed js call");
        ObservedElement {
            observer: Some(self.observer.clone()),
            element,
        }
    }
}

impl ObservedElement {
    pub fn element(&self) -> &Element {
        &self.element
    }
}

impl Drop for ResizeObserver {
    fn drop(&mut self) {
        if let Some(_cb) = self.closure.take() {
            self.observer.disconnect().expect("can disconnect");
        }
    }
}

impl Drop for ObservedElement {
    fn drop(&mut self) {
        if let Some(this) = self.observer.take() {
            this.unobserve(self.element.clone())
                .expect("failed js call");
        }
    }
}
