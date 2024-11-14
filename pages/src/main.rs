use yew::{function_component, html, Html};
use yew_confetti::Confetti;

#[function_component(App)]
fn app() -> Html {
    html! {
        <Confetti
            scalar={20.0}
            x={100.0}
            y={50.0}
            gravity={-9.81}
            start_velocity={20.0}
        />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
