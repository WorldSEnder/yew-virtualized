use yew::prelude::*;
use yew_virtualized::VirtualList;

#[derive(PartialEq, Properties)]
struct ItemProps {
    idx: usize,
}

#[function_component(Item)]
fn item(ItemProps { idx }: &ItemProps) -> Html {
    html! {
        <div class={"item"}>
            {format!("Item {idx}")}
        </div>
    }
}

fn items(idx: usize) -> Html {
    html! { <Item {idx} /> }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <VirtualList
            item_count={100}
            height_prior={30}
            items={VirtualList::item_gen(items)}
            classes={"scrollbar"} />
    }
}

fn main() { yew::start_app::<App>(); }
