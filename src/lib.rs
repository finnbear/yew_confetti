use std::ops::Range;

use js_sys::wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};
use yew::{
    function_component, html, use_effect_with, use_mut_ref, use_node_ref, AttrValue, Classes, Html,
    Properties,
};

#[derive(Clone, PartialEq, Properties)]
pub struct ConfettiProps {
    #[prop_or(256)]
    pub width: u32,
    #[prop_or(256)]
    pub height: u32,
    /// If continuous, controls max alive particles. Otherwise, controls how many spawn at beginning.
    #[prop_or(50)]
    pub count: usize,
    /// Emitter horizontal position. 0.0 means left edge, 1.0 means right edge.
    #[prop_or(0.5)]
    pub x: f32,
    /// Emitter vertical position. 0.0 means bottom edge, 1.0 means top edge.
    #[prop_or(0.5)]
    pub y: f32,
    /// Launch angle (0 = right, PI/2 = up, etc.)
    #[prop_or(90f32.to_radians())]
    pub angle: f32,
    /// Random variation in launch angle (PI/2 = PI/4 on each side)
    #[prop_or(45f32.to_radians())]
    pub spread: f32,
    #[prop_or(2.0)]
    pub velocity: f32,
    /// Velocity decay per second.
    #[prop_or(0.25)]
    pub decay: f32,
    /// Downward acceleration.
    #[prop_or(1.0)]
    pub gravity: f32,
    /// Rightward acceleration.
    #[prop_or(0.0)]
    pub drift: f32,
    /// Number of seconds each particle lasts.
    #[prop_or(4.0)]
    pub lifespan: f32,
    #[prop_or(true)]
    pub continuous: bool,
    /// Shape probability distribution. Repeated shapes are more likely.
    #[prop_or(&[Shape::Circle, Shape::Square])]
    pub shapes: &'static [Shape],
    /// CSS color probability distribution. Repeated colors are more likely.
    #[prop_or(&["#26ccff", "#a25afd", "#ff5e7e", "#88ff5a", "#fcff42", "#ffa62d", "#ff36ff"])]
    pub colors: &'static [&'static str],
    /// Don't show any confetti if user prefers reduced motion, according to a CSS media query.
    #[prop_or(true)]
    pub disable_for_reduced_motion: bool,
    #[prop_or(5.0)]
    pub scalar: f32,
    /// Classes to apply to the canvas.
    #[prop_or_default]
    pub class: Classes,
    /// Inline style to apply to the canvas.
    #[prop_or(None)]
    pub style: Option<AttrValue>,
    /// Id of the canvas.
    #[prop_or(None)]
    pub id: Option<AttrValue>,
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame`");
}

#[derive(Default)]
struct State {
    confetti: Vec<Fetti>,
    callback: Option<Closure<dyn FnMut(f64)>>,
    last_time: f64,
}

#[function_component(Confetti)]
pub fn confetti(props: &ConfettiProps) -> Html {
    let canvas = use_node_ref();
    let state = use_mut_ref(|| State::default());

    use_effect_with((canvas.clone(), props.clone()), move |(canvas, props)| {
        let disable_for_reduced_motion = props.disable_for_reduced_motion;
        let context = canvas
            .cast::<HtmlCanvasElement>()
            .unwrap()
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        let props = props.clone();
        let state_2 = state.clone();
        let spawn_period = props.lifespan / props.count as f32;
        let mut spawn_credits = 0.0;
        state_2.borrow_mut().callback = Some(Closure::new(move |time: f64| {
            {
                context.reset();
                let mut state = state.borrow_mut();

                let delta = ((time - state.last_time) * 0.001).clamp(0.0, 0.5) as f32;
                spawn_credits += delta;

                while if props.continuous {
                    spawn_credits > spawn_period
                } else {
                    state.last_time == 0.0
                } && state.confetti.len() < props.count
                {
                    spawn_credits -= spawn_period;
                    state.confetti.push(Fetti::new(&props));
                }
                state
                    .confetti
                    .retain_mut(|fetti| fetti.update(delta, &props, &context));
                //gloo_console::log!("{}", state.confetti.len());
                state.last_time = time;
            }

            request_animation_frame(state.borrow().callback.as_ref().unwrap());
        }));

        if !disable_for_reduced_motion
            || !window()
                .unwrap()
                .match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
                .map(|m| m.matches())
                .unwrap_or(false)
        {
            request_animation_frame(state_2.borrow().callback.as_ref().unwrap());
        }

        move || drop(state_2.borrow_mut().callback.take())
    });

    html! {
        <canvas
            ref={canvas}
            id={props.id.clone()}
            width={props.width.to_string()}
            height={props.height.to_string()}
            style={format!("pointer-events: none;{}", props.style.as_ref().map(|s| s.as_str()).unwrap_or(""))}
            class={props.class.clone()}
        />
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Shape {
    Circle,
    Square,
}

struct Fetti {
    x: f32,
    y: f32,
    wobble: f32,
    wobble_speed: f32,
    velocity: f32,
    angle_2d: f32,
    tilt_angle: f32,
    color: &'static str,
    shape: Shape,
    life_remaining: f32,
}

fn rand_unit() -> f32 {
    js_sys::Math::random() as f32
}

fn rand_max(max: f32) -> f32 {
    rand_unit() * max
}

fn rand_range(min: f32, max: f32) -> f32 {
    min + (max - min) * rand_unit()
}

impl Fetti {
    fn new(props: &ConfettiProps) -> Self {
        Self {
            x: props.x,
            y: props.y,
            wobble: rand_unit(),
            wobble_speed: rand_range(0.01, 0.015),
            velocity: props.velocity + (0.5 + 0.5 * js_sys::Math::random() as f32),
            angle_2d: props.angle + rand_range(-props.spread * 0.5, props.spread * 0.5),
            tilt_angle: rand_max(std::f32::consts::TAU),
            color: props.colors[rand_max(props.colors.len() as f32) as usize],
            shape: props.shapes[rand_max(props.shapes.len() as f32) as usize],
            life_remaining: props.lifespan,
        }
    }

    fn update(
        &mut self,
        delta: f32,
        props: &ConfettiProps,
        context: &CanvasRenderingContext2d,
    ) -> bool {
        self.x += (self.angle_2d.cos() * self.velocity + props.drift) * delta;
        self.y += (self.angle_2d.sin() * self.velocity - props.gravity) * delta;
        self.velocity *= props.decay.powf(delta);
        self.wobble += self.wobble_speed * delta;
        self.tilt_angle += 0.1 * delta;
        self.life_remaining -= delta;
        if self.life_remaining > 0.0 {
            self.draw(props, context);
            true
        } else {
            false
        }
    }

    fn draw(&self, props: &ConfettiProps, context: &CanvasRenderingContext2d) {
        let center_x = map_ranges(self.x, 0.0..1.0, 0.0..props.width as f32);
        let center_y = map_ranges(self.y, 0.0..1.0, props.height as f32..0.0);

        let wobble_x = center_x + self.wobble.cos() * props.scalar;
        let wobble_y = center_y + self.wobble.sin() * props.scalar;
        let tilt_sin = self.tilt_angle.sin();
        let tilt_cos = self.tilt_angle.cos();

        let random = rand_range(2.0, 3.0);
        let x1 = center_x + tilt_cos * random;
        let y1 = center_y + tilt_sin * random;
        let x2 = wobble_x + tilt_cos * random;
        let y2 = wobble_y + tilt_sin * random;

        context.set_fill_style_str(&self.color);
        // TODO: Dirty state.
        context.set_global_alpha((self.life_remaining / props.lifespan) as f64);

        context.begin_path();
        match self.shape {
            Shape::Circle => {
                let _ = context.ellipse(
                    center_x as f64,
                    center_y as f64,
                    ((x2 - x1).abs() * 0.5) as f64,
                    ((y2 - y1).abs() * 0.5) as f64,
                    self.wobble as f64,
                    0.0,
                    std::f64::consts::TAU,
                );
            }
            Shape::Square => {
                context.move_to(center_x.floor() as f64, center_y.floor() as f64);
                context.line_to(wobble_x.floor() as f64, y1 as f64);
                context.line_to(x2.floor() as f64, y2.floor() as f64);
                context.line_to(x1.floor() as f64, wobble_y.floor() as f64);
            }
        }

        context.close_path();
        context.fill();
    }
}

#[inline]
pub fn map_ranges(number: f32, old: Range<f32>, new: Range<f32>) -> f32 {
    let old_range = old.end - old.start;
    let new_range = new.end - new.start;
    let mul: f32 = new_range / old_range;
    let add: f32 = -old.start * mul + new.start;

    if cfg!(target_feature = "fma") {
        number.mul_add(mul, add)
    } else {
        number * mul + add
    }
}
