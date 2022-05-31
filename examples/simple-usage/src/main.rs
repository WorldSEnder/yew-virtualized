use yew::prelude::*;
use yew_virtualized::VirtualList;

#[derive(PartialEq, Properties)]
struct ItemProps {
    idx: usize,
}

#[function_component]
fn Item(ItemProps { idx }: &ItemProps) -> Html {
    html! {
        <div class={"item"}>
            {format!("Item {idx}")}
        </div>
    }
}

fn items(idx: usize) -> Html {
    html! { <Item {idx} /> }
}

#[function_component]
fn App() -> Html {
    html! {
        <VirtualList
            item_count={100}
            height_prior={30}
            {items}
            classes={"scrollbar"} />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
