## yew-virtualized

A `VirtualList` yew component that renders only the visible part of a scrollable list.

This component uses the [`ResizeObserver`] API to allow dynamically sized items in the list.

### Quick Example

```rust
fn items(idx: usize) -> Html {
    html! { format!("Item #{idx}") }
}

#[function_component]
fn App() -> Html {
    html! {
        <VirtualList
            // An approximate item height that will be used to guess
            // space usage before the first render of an item. Subsequent renders
            // use the exact item height
            height_prior={30}
            // How many items to render, in total.
            item_count={100}
            // Callback function to render individual items by index in 0..item_count
            {items}
            // Additional classes to apply to the root node
            classes={"scrollbar"} />
    }
}
```

[`ResizeObserver`]: https://developer.mozilla.org/en-US/docs/Web/API/ResizeObserver
