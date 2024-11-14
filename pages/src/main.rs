use yew::{function_component, html, Html};
use yew_confetti::Confetti;

#[function_component(App)]
fn app() -> Html {
    html! {
        <Confetti scalar={15.0} x={100.0} y={50.0} gravity={0.2} spread={45.0} start_velocity={0.003}/>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
