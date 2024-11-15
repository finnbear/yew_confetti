use std::ops::Deref;
use std::str::FromStr;
use web_sys::HtmlInputElement;
use yew::{
    function_component, html, use_state_eq, Callback, Html, InputEvent, MouseEvent, Properties,
    TargetCast,
};
use yew_confetti::{Confetti, ConfettiProps};

#[function_component(App)]
fn app() -> Html {
    let key = use_state_eq(|| 0i32);
    let on_reset = {
        let key = key.clone();
        Callback::from(move |_: MouseEvent| {
            key.set(key.wrapping_add(1));
        })
    };

    let props = use_state_eq(|| {
        let __yew_props = ConfettiProps::builder();
        let __yew_required_props_token = ::yew::html::AssertAllProps;
        ::yew::html::Buildable::prepare_build(__yew_props, &__yew_required_props_token).build()
    });

    let checkbox_factory = {
        let props = props.clone();
        move |name: &'static str,
              load: fn(&ConfettiProps) -> bool,
              store: fn(&mut ConfettiProps, bool)|
              -> Html {
            let props = props.clone();
            let value = load(&*props);
            let new_props = props.deref().clone();
            let oninput = Callback::from(move |event: InputEvent| {
                let input = event.target_dyn_into::<HtmlInputElement>().unwrap();
                let value = input.checked();
                let mut new_props = new_props.clone();
                store(&mut new_props, value);
                props.set(new_props);
            });
            html! {<tr>
                <td>{name}{":"}</td>
                <td>
                    <input
                        type="checkbox"
                        checked={value}
                        {oninput}
                    />
                </td>
                <td>{value}</td>
            </tr>}
        }
    };

    let slider_factory = {
        let props = props.clone();
        move |name,
              min: f32,
              max: f32,
              load: fn(&ConfettiProps) -> f32,
              store: fn(&mut ConfettiProps, f32)|
              -> Html {
            let props = props.clone();
            let value = load(&*props).to_string();
            let new_props = props.deref().clone();
            let oninput = Callback::from(move |event: InputEvent| {
                let input = event.target_dyn_into::<HtmlInputElement>().unwrap();
                let value = input.value();
                let mut new_props = new_props.clone();
                store(&mut new_props, f32::from_str(&value).unwrap());
                props.set(new_props);
            });
            html! {<tr>
                <td>{name}{":"}</td>
                <td>
                    <input
                        type="range"
                        min={min.to_string()}
                        max={max.to_string()}
                        step="0.001"
                        value={value.clone()}
                        {oninput}
                    />
                </td>
                <td>{value}</td>
            </tr>}
        }
    };

    html! {<>
        <h2>{"yew_confetti"}</h2>
        <Confetti
            key={*key}
            style={"background-color: black;"}
            ..props.deref().clone()
        />
        <table style="border-spacing: 0.25rem; table-layout: fixed; border-collapse: separate;">
            {slider_factory("width", 64.0, 512.0, |props| props.width as f32, |props, width| {
                props.width = width as u32;
            })}
            {slider_factory("height", 64.0, 512.0, |props| props.height as f32, |props, height| {
                props.height = height as u32;
            })}
            {slider_factory("count", 1.0, 500.0, |props| props.count as f32, |props, count| {
                props.count = count as usize;
            })}
            {slider_factory("x", -0.1, 1.1, |props| props.x, |props, x| {
                props.x = x;
            })}
            {slider_factory("y", -0.1, 1.1, |props| props.y, |props, y| {
                props.y = y;
            })}
            {slider_factory("angle", 0.0, std::f32::consts::TAU, |props| props.angle, |props, angle| {
                props.angle = angle;
            })}
            {slider_factory("spread", 0.0, std::f32::consts::PI, |props| props.spread, |props, spread| {
                props.spread = spread;
            })}
            {slider_factory("velocity", 0.1, 3.0, |props| props.velocity, |props, velocity| {
                props.velocity = velocity;
            })}
            {slider_factory("decay", 0.001, 1.0, |props| props.decay, |props, decay| {
                props.decay = decay;
            })}
            {slider_factory("drift", -1.0, 1.0, |props| props.drift, |props, drift| {
                props.drift = drift;
            })}
            {slider_factory("gravity", 0.0, 2.0, |props| props.gravity, |props, gravity| {
                props.gravity = gravity;
            })}
            {slider_factory("lifespan", 1.0, 5.0, |props| props.lifespan, |props, lifespan| {
                props.lifespan = lifespan;
            })}
            {slider_factory("scalar", 0.1, 10.0, |props| props.scalar, |props, scalar| {
                props.scalar = scalar;
            })}
            {checkbox_factory("continuous", |props| props.continuous, |props, continuous| {
                props.continuous = continuous;
            })}
            <tr>
                <td colspan="3"><button
                    onclick={on_reset}
                    style="color: black;"
                >{"Reset"}</button></td>
            </tr>
        </table>
    </>}
}

fn main() {
    yew::Renderer::<App>::new().render();
}
