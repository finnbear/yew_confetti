use std::fmt::Write;
use std::ops::Deref;
use std::str::FromStr;
use web_sys::HtmlInputElement;
use yew::{
    function_component, html, html_nested, props, use_state_eq, Callback, Html, InputEvent,
    MouseEvent, TargetCast, UseStateHandle,
};
use yew_confetti::{Cannon, CannonProps, Confetti, ConfettiProps};

#[function_component(App)]
fn app() -> Html {
    let key = use_state_eq(|| 0i32);
    let on_reset = {
        let key = key.clone();
        Callback::from(move |_: MouseEvent| {
            key.set(key.wrapping_add(1));
        })
    };

    let show_defaults = use_state_eq(|| false);
    let props = use_state_eq(|| props!(ConfettiProps {}));

    let cannons_props = use_state_eq(|| vec![props!(CannonProps {})]);

    fn checkbox_factory<P: Clone + 'static>(
        name: &str,
        props: UseStateHandle<P>,
        load: impl Fn(&P) -> bool + 'static,
        store: impl Fn(&mut P, bool) + 'static,
    ) -> Html {
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
        html! {
            <tr>
                <td>{name}{":"}</td>
                <td>
                    <input
                        type="checkbox"
                        checked={value}
                        {oninput}
                    />
                </td>
                <td>{value}</td>
            </tr>
        }
    }

    fn slider_factory<P: Clone + 'static>(
        name: &str,
        min: f32,
        max: f32,
        props: UseStateHandle<P>,
        load: impl Fn(&P) -> f32 + 'static,
        store: impl Fn(&mut P, f32) + 'static,
    ) -> Html {
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
        html! {
            <tr>
                <td>{name}{":"}</td>
                <td>
                    <input
                        type="range"
                        min={min.to_string()}
                        max={max.to_string()}
                        step="0.01"
                        value={value.clone()}
                        {oninput}
                    />
                </td>
                <td>{value}</td>
            </tr>
        }
    }

    let style = format!(
        "background-color: black; width: {}px; height: {}px;",
        props.width, props.height
    );

    let default_props = props!(ConfettiProps {});
    let mut code = String::new();
    write!(&mut code, "<Confetti\n").unwrap();
    macro_rules! prop {
        ($code: ident, $props: ident, $defaults: ident, $prop: ident, $ident: literal, $show_defaults: ident) => {
            if *$show_defaults || $props.$prop != $defaults.$prop {
                write!(
                    &mut $code,
                    "{}    {}={{{}}}\n",
                    $ident,
                    stringify!($prop),
                    $props.$prop
                )
                .unwrap();
            }
        };
    }
    prop!(code, props, default_props, width, "", show_defaults);
    prop!(code, props, default_props, height, "", show_defaults);
    prop!(code, props, default_props, count, "", show_defaults);
    prop!(code, props, default_props, decay, "", show_defaults);
    prop!(code, props, default_props, drift, "", show_defaults);
    prop!(code, props, default_props, gravity, "", show_defaults);
    prop!(code, props, default_props, lifespan, "", show_defaults);
    prop!(code, props, default_props, scalar, "", show_defaults);
    prop!(code, props, default_props, continuous, "", show_defaults);
    write!(&mut code, "    style={{{style:?}}}\n").unwrap();
    write!(&mut code, ">\n").unwrap();
    for props in cannons_props.iter() {
        let default_props = props!(CannonProps {});
        write!(&mut code, "    <Cannon\n").unwrap();
        prop!(code, props, default_props, x, "    ", show_defaults);
        prop!(code, props, default_props, y, "    ", show_defaults);
        prop!(code, props, default_props, angle, "    ", show_defaults);
        prop!(code, props, default_props, spread, "    ", show_defaults);
        prop!(code, props, default_props, velocity, "    ", show_defaults);
        write!(&mut code, "    />\n").unwrap();
    }
    write!(&mut code, "</Confetti>\n").unwrap();

    html! {<>
        <h2 style="margin-top: 0;">{"yew_confetti"}</h2>
        <div style="display: flex; flex-direction: column; gap: 0.5rem; width: min-content;">
            <div style="display: flex; flex-direction: row; gap: 0.5rem;">
                <a style="color: white;" href="https://github.com/finnbear/yew_confetti">{"GitHub"}</a>
                <a style="color: white;" href="https://crates.io/crates/yew_confetti">{"crates.io"}</a>
                <a style="color: white;" href="https://docs.rs/yew_confetti/latest/yew_confetti">{"docs.rs"}</a>
            </div>
            <div style="display: flex; flex-direction: row; gap: 0.5rem;">
                <Confetti
                    key={*key}
                    {style}
                    ..props.deref().clone()
                >
                    {for cannons_props.deref().clone().into_iter().map(|props| html_nested!{
                        <Cannon ..props/>
                    })}
                </Confetti>
                <pre style="min-width: 30rem;">
                    {code}
                </pre>
            </div>
            <table style="border-spacing: 0.25rem; table-layout: fixed; border-collapse: separate;">
                {slider_factory("width", 64.0, 512.0, props.clone(), |props| props.width as f32, |props, width| {
                    props.width = width as u32;
                })}
                {slider_factory("height", 64.0, 512.0, props.clone(), |props| props.height as f32, |props, height| {
                    props.height = height as u32;
                })}
                {slider_factory("count", 1.0, 500.0, props.clone(), |props| props.count as f32, |props, count| {
                    props.count = count as usize;
                })}
                {slider_factory("cannons", 0.0, 3.0, cannons_props.clone(), move |props| props.len() as f32, move |props, x| {
                    let x = x as usize;
                    props.truncate(x);
                    while props.len() < x {
                        props.push(props!(CannonProps{}));
                    }
                })}
                {cannons_props.iter().enumerate().map(|(i, _)| html!{<>
                    {slider_factory(&format!("x{i}"), -0.1, 1.1, cannons_props.clone(), move |props| props[i].x, move |props, x| {
                        props[i].x = x;
                    })}
                    {slider_factory(&format!("y{i}"), -0.1, 1.1, cannons_props.clone(), move |props| props[i].y, move |props, y| {
                        props[i].y = y;
                    })}
                    {slider_factory(&format!("angle{i}"), 0.0, std::f32::consts::TAU, cannons_props.clone(), move |props| props[i].angle, move |props, angle| {
                        props[i].angle = angle;
                    })}
                    {slider_factory(&format!("spread{i}"), 0.0, std::f32::consts::PI, cannons_props.clone(), move |props| props[i].spread, move |props, spread| {
                        props[i].spread = spread;
                    })}
                    {slider_factory(&format!("velocity{i}"), 0.1, 3.0, cannons_props.clone(), move |props| props[i].velocity, move |props, velocity| {
                        props[i].velocity = velocity;
                    })}
                </>}).collect::<Html>()}
                {slider_factory("decay", 0.01, 1.0, props.clone(), |props| props.decay, |props, decay| {
                    props.decay = decay;
                })}
                {slider_factory("drift", -1.0, 1.0, props.clone(), |props| props.drift, |props, drift| {
                    props.drift = drift;
                })}
                {slider_factory("gravity", 0.0, 2.0, props.clone(), |props| props.gravity, |props, gravity| {
                    props.gravity = gravity;
                })}
                {slider_factory("lifespan", 1.0, 5.0, props.clone(), |props| props.lifespan, |props, lifespan| {
                    props.lifespan = lifespan;
                })}
                {slider_factory("scalar", 0.1, 10.0, props.clone(), |props| props.scalar, |props, scalar| {
                    props.scalar = scalar;
                })}
                {checkbox_factory("continuous", props.clone(), |props| props.continuous, |props, continuous| {
                    props.continuous = continuous;
                })}
                {checkbox_factory("show_defaults", show_defaults.clone(), |props| *props, |props, continuous| {
                    *props = continuous;
                })}
                <tr>
                    <td colspan="3"><button
                        onclick={on_reset}
                        style="color: black;"
                    >{"Reset"}</button></td>
                </tr>
            </table>
        </div>
    </>}
}

fn main() {
    yew::Renderer::<App>::new().render();
}
