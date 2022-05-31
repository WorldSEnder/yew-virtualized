//! An infinite scroll component for Yew.

mod resize_observer;

use std::{cell::RefCell, fmt::Display, rc::Rc};

use gloo_timers::callback::Timeout;
use resize_observer::{ObservedElement, ResizeObserver};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast};
use web_sys::Element;
use yew::{html::IntoPropValue, prelude::*};

type ItemGenerator = Callback<usize, Html>;

#[derive(PartialEq)]
pub enum ItemSize {
    Pixels(usize),
}

impl ItemSize {
    fn as_scroll_size(&self) -> i32 {
        match self {
            Self::Pixels(pxs) => (*pxs).try_into().unwrap(),
        }
    }
}

impl IntoPropValue<ItemSize> for usize {
    fn into_prop_value(self) -> ItemSize {
        ItemSize::Pixels(self)
    }
}

impl Display for ItemSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pixels(pxs) => write!(f, "{pxs}px"),
        }
    }
}

impl std::ops::Mul<&'_ ItemSize> for usize {
    type Output = ItemSize;

    fn mul(self, rhs: &ItemSize) -> Self::Output {
        match rhs {
            ItemSize::Pixels(pxs) => ItemSize::Pixels(self * pxs),
        }
    }
}

#[wasm_bindgen]
extern "C" {
    type PositionedElementDuck;
    #[wasm_bindgen(method, getter, structural, js_name = __yew_resize_obs_pos)]
    fn pos(this: &PositionedElementDuck) -> usize;
    #[wasm_bindgen(method, setter, structural, js_name = __yew_resize_obs_pos)]
    fn set_pos(this: &PositionedElementDuck, pos: usize);
}

#[derive(Properties)]
struct ScrollWrapperProps {
    observer: Rc<ResizeObserver>,
    pos: usize,
    children: Children,
    classes: Classes,
}

impl PartialEq for ScrollWrapperProps {
    fn eq(&self, other: &Self) -> bool {
        self.children == other.children
    }
}

#[function_component]
fn ScrollItemWrapper(props: &ScrollWrapperProps) -> Html {
    let wrapped_ref = use_node_ref();
    let observed = use_mut_ref(|| Option::<ObservedElement>::None);
    {
        let wrapped_ref = wrapped_ref.clone();
        let observer = props.observer.clone();
        let pos = props.pos;
        use_effect(move || {
            let el = wrapped_ref.cast::<Element>().unwrap();
            let positioned_el = el.unchecked_ref::<PositionedElementDuck>();
            positioned_el.set_pos(pos);
            let mut observed = observed.borrow_mut();
            if matches!(&*observed, Some(observed) if observed.element() != &el) {
                *observed = None;
            }
            if observed.is_none() {
                *observed = Some(observer.observe(el));
            }
            || {}
        })
    }
    html! {
        <div ref={&wrapped_ref} class={props.classes.clone()}>
            {props.children.clone()}
        </div>
    }
}

struct SharedScrollState {
    element_sizes: RefCell<Vec<f64>>,
    trigger_update: UseForceUpdate,
}

struct ScrollManager {
    host_height: i32,
    scroll_top: i32,
    observer: Rc<ResizeObserver>,
    shared: Rc<SharedScrollState>,
}

impl ScrollManager {
    fn new(trigger_update: UseForceUpdate) -> Self {
        let shared = {
            let trigger_update = trigger_update.clone();
            Rc::new(SharedScrollState {
                element_sizes: RefCell::default(),
                trigger_update,
            })
        };
        let observer = {
            let shared = shared.clone();
            Rc::new(ResizeObserver::new(move |change_entries| {
                let mut element_sizes = shared.element_sizes.borrow_mut();
                for change in change_entries {
                    let pos = change
                        .target()
                        .unchecked_ref::<PositionedElementDuck>()
                        .pos();
                    element_sizes[pos] = change.content_rect().height();
                }
                trigger_update.force_update();
            }))
        };
        ScrollManager {
            host_height: 0,
            scroll_top: 0,
            observer,
            shared,
        }
    }

    fn mounted(&mut self, host: Element) {
        let height = host.client_height();
        self.host_height = height;
        self.shared.trigger_update.force_update();
    }

    fn unmount(&mut self) {}

    fn update(&mut self, scroll_top: i32) {
        if self.scroll_top != scroll_top {
            self.scroll_top = scroll_top;
            self.shared.trigger_update.force_update();
        }
    }

    fn generate_contents(&self, props: &VirtualListProps) -> Html {
        let item_height = props.height_prior.as_scroll_size();
        // take care of some state change
        {
            let mut element_sizes = self.shared.element_sizes.borrow_mut();
            element_sizes.resize(props.item_count, item_height.into());
        }

        let element_sizes = self.shared.element_sizes.borrow();
        // TODO: depend on item_height and scroll speed?
        const EXTRA_BUFFER: usize = 5;
        // TODO: rework to range-query datastructure
        let mut before_ring_buffered: [f64; EXTRA_BUFFER] = [0.0; EXTRA_BUFFER];
        let mut before_ring_buff_idx = 0usize;
        let mut first_idx = props.item_count;

        let mut passed_height = 0f64;
        for (i, i_size) in element_sizes.iter().enumerate() {
            let height_before = passed_height;
            passed_height += i_size;
            if passed_height >= self.scroll_top.into() {
                first_idx = i;
                break;
            }

            before_ring_buffered[before_ring_buff_idx as usize] = height_before;
            before_ring_buff_idx += 1;
            before_ring_buff_idx %= before_ring_buffered.len();
        }
        let first_idx = first_idx.saturating_sub(EXTRA_BUFFER).min(props.item_count);
        let hidden_before = before_ring_buffered[first_idx % EXTRA_BUFFER];

        let mut past_last_idx = props.item_count;
        let mut passed_height = hidden_before;
        for (i, i_size) in element_sizes.iter().enumerate().skip(first_idx) {
            passed_height += i_size;
            if passed_height >= (self.scroll_top + self.host_height).into() {
                past_last_idx = i.saturating_add(1 + EXTRA_BUFFER);
                break;
            }
        }
        let past_last_idx = past_last_idx.min(props.item_count);
        let hidden_after: f64 = element_sizes[past_last_idx..].iter().sum();

        //let item_height = &props.item_height;
        let items = (first_idx..past_last_idx).map(|i| {
            let item = props.items.emit(i);
            html! {
                <ScrollItemWrapper key={i} pos={i} observer={&self.observer} classes={props.item_classes.clone()}>
                    {item}
                </ScrollItemWrapper>
            }
        });

        html! {
            <>
            <div key="pre" style={format!("height: {hidden_before}px;")}>
            </div>
            <div key="wrap" style={"display: contents;"}>
            {for items}
            </div>
            <div key="post" style={format!("height: {hidden_after}px;")}>
            </div>
            </>
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct VirtualListProps {
    pub items: ItemGenerator,
    pub item_count: usize,
    pub height_prior: ItemSize,
    #[prop_or_default]
    pub classes: Classes,
    #[prop_or_default]
    pub item_classes: Classes,
}

#[hook]
fn use_debounce<E: 'static>(millis: u32, cb: Callback<E>) -> Callback<E> {
    (*use_memo(
        |cb| {
            let cb = cb.clone();
            let debounced = Rc::new(RefCell::new(None));
            Callback::from(move |scroll| {
                let mut debounced_ref = debounced.borrow_mut();
                if (*debounced_ref).is_some() {
                    return;
                }
                let cb = cb.clone();
                let debounced = debounced.clone();
                *debounced_ref = Some(Timeout::new(millis, move || {
                    cb.emit(scroll);
                    *debounced.borrow_mut() = None;
                }))
            })
        },
        cb,
    ))
    .clone()
}

#[function_component]
pub fn VirtualList(props: &VirtualListProps) -> Html {
    let update = use_force_update();
    let manager = use_mut_ref(|| ScrollManager::new(update));

    let host_ref = use_node_ref();
    {
        let scroll_manager = manager.clone();
        let host_ref = host_ref.clone();
        use_effect_with_deps(
            move |_| {
                let host = host_ref.cast::<Element>().unwrap();
                scroll_manager.borrow_mut().mounted(host);
                move || {
                    scroll_manager.borrow_mut().unmount();
                }
            },
            (),
        );
    }

    let onscroll = {
        let scroll_manager = manager.clone();
        let cb = use_callback(
            move |scroll: Event, _| {
                let el = scroll.target_dyn_into::<web_sys::Element>().unwrap();
                let scroll_top = el.scroll_top();
                scroll_manager.borrow_mut().update(scroll_top);
            },
            (),
        );
        use_debounce(50, cb)
    };
    let contents = (*manager).borrow_mut().generate_contents(props);

    html! {
        <div ref={host_ref} class={props.classes.clone()} {onscroll}>
            {contents}
        </div>
    }
}
