use std::ops::Range;

use js_sys::wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};
use yew::{
    function_component, html, use_effect_with, use_mut_ref, use_node_ref, AttrValue,
    ChildrenWithProps, Classes, Component, Html, Properties,
};

/// Confetti animation options.
#[derive(Clone, PartialEq, Properties)]
pub struct ConfettiProps {
    /// Horizontal resolution of canvas.
    #[prop_or(256)]
    pub width: u32,
    /// Vertical resolution of canvas.
    #[prop_or(256)]
    pub height: u32,
    /// If continuous, controls max alive particles. Otherwise, controls how many spawn at beginning.
    #[prop_or(150)]
    pub count: usize,
    /// Velocity decay per second (0.5 means lose 50% of velocity per second).
    #[prop_or(0.3)]
    pub decay: f32,
    /// Downward acceleration.
    #[prop_or(1.0)]
    pub gravity: f32,
    /// Rightward acceleration.
    #[prop_or(0.0)]
    pub drift: f32,
    /// Number of seconds each particle lasts.
    #[prop_or(2.5)]
    pub lifespan: f32,
    /// Don't show any confetti if user prefers reduced motion, according to a CSS media query.
    #[prop_or(true)]
    pub disable_for_reduced_motion: bool,
    /// Particle size.
    #[prop_or(5.0)]
    pub scalar: f32,
    /// Whether to continuously spawn particles (or just once at the beginning).
    #[prop_or(true)]
    pub continuous: bool,
    /// Classes to apply to the canvas.
    #[prop_or_default]
    pub class: Classes,
    /// Inline style to apply to the canvas.
    #[prop_or(None)]
    pub style: Option<AttrValue>,
    /// Id of the canvas.
    #[prop_or(None)]
    pub id: Option<AttrValue>,
    /// `<Cannon/>`'s
    #[prop_or_default]
    pub children: ChildrenWithProps<Cannon>,
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) -> i32 {
    window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame`")
}

#[derive(Default)]
struct State {
    confetti: Vec<Fetti>,
    callback: Option<Closure<dyn FnMut(f64)>>,
    animation_frame: Option<i32>,
    last_time: Option<f64>,
}

/// Confetti emitter options.
#[derive(Clone, PartialEq, Properties)]
pub struct CannonProps {
    /// Emitter horizontal position. 0.0 means left edge, 1.0 means right edge.
    #[prop_or(0.5)]
    pub x: f32,
    /// Emitter vertical position. 0.0 means bottom edge, 1.0 means top edge.
    #[prop_or(0.5)]
    pub y: f32,
    /// Launch angle (0 = right, PI/2 = up, etc.).
    #[prop_or(90f32.to_radians())]
    pub angle: f32,
    /// Random variation in launch angle (PI/2 = PI/4 on each side).
    #[prop_or(45f32.to_radians())]
    pub spread: f32,
    /// Initial velocity.
    #[prop_or(2.0)]
    pub velocity: f32,
    /// Shape probability distribution. Repeated shapes are more likely.
    #[prop_or(&[Shape::Circle, Shape::Square])]
    pub shapes: &'static [Shape],
    /// CSS color probability distribution. Repeated colors are more likely.
    #[prop_or(&["#26ccff", "#a25afd", "#ff5e7e", "#88ff5a", "#fcff42", "#ffa62d", "#ff36ff"])]
    pub colors: &'static [&'static str],
}

/// Confetti emitter component.
pub struct Cannon;
impl Component for Cannon {
    type Properties = CannonProps;
    type Message = ();
    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self
    }
    fn view(&self, _ctx: &yew::Context<Self>) -> Html {
        panic!("<Cannon> must be inside <Confetti>");
    }
}

/// Confetti animation component.
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
        let spawn_period = props.children.len() as f32 * props.lifespan / props.count as f32;
        let mut spawn_credits = 0.0;
        state_2.borrow_mut().callback = Some(Closure::new(move |time: f64| {
            context.reset();
            let mut state = state.borrow_mut();

            let delta = ((time - state.last_time.unwrap_or(time)) * 0.001).clamp(0.0, 0.5) as f32;
            if !props.children.is_empty() {
                spawn_credits += delta;

                while if props.continuous {
                    spawn_credits > spawn_period
                } else {
                    state.last_time.is_none() && state.confetti.len() < props.count
                } {
                    spawn_credits -= spawn_period;
                    for cannon in props.children.iter() {
                        state.confetti.push(Fetti::new(&props, &cannon.props));
                    }
                }
            }

            state
                .confetti
                .retain_mut(|fetti| fetti.update(delta, &props, &context));
            state.last_time = Some(time);

            state.animation_frame = Some(request_animation_frame(state.callback.as_ref().unwrap()));
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
            let mut state = state_2.borrow_mut();
            state.animation_frame = Some(request_animation_frame(state.callback.as_ref().unwrap()));
        }

        move || {
            let mut state = state_2.borrow_mut();
            if let Some(animation_frame) = state.animation_frame.take() {
                let _ = window().unwrap().cancel_animation_frame(animation_frame);
            }
            drop(state.callback.take());
        }
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

/// Particle shape.
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
    min + rand_max(max - min)
}

impl Fetti {
    fn new(props: &ConfettiProps, cannon: &CannonProps) -> Self {
        let (sin, cos) = rand_max(std::f32::consts::TAU).sin_cos();
        let mag = rand_unit().sqrt();
        Self {
            x: cannon.x,
            y: cannon.y,
            wobble: rand_unit(),
            wobble_speed: rand_range(0.01, 0.015),
            velocity: cannon.velocity * (0.9 + 0.1 * sin * mag),
            angle_2d: cannon.angle + cos * cannon.spread * 0.5 * mag,
            tilt_angle: rand_max(std::f32::consts::TAU),
            color: cannon.colors[rand_max(cannon.colors.len() as f32) as usize],
            shape: cannon.shapes[rand_max(cannon.shapes.len() as f32) as usize],
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
fn map_ranges(number: f32, old: Range<f32>, new: Range<f32>) -> f32 {
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
