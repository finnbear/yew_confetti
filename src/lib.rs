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
    #[prop_or(50)]
    pub particle_count: u32,
    #[prop_or((-90f32).to_radians())]
    pub angle: f32,
    #[prop_or(45f32.to_radians())]
    pub spread: f32,
    #[prop_or(45.0)]
    pub start_velocity: f32,
    #[prop_or(0.9)]
    pub decay: f32,
    #[prop_or(1.0)]
    pub gravity: f32,
    #[prop_or(0.0)]
    pub drift: f32,
    /// Number of seconds each particle lasts.
    #[prop_or(4.0)]
    pub lifespan: f32,
    #[prop_or(0.5)]
    pub x: f32,
    #[prop_or(0.5)]
    pub y: f32,
    /// Shape probability distribution. Repeated shapes are more likely.
    #[prop_or(&[Shape::Circle, Shape::Square])]
    pub shapes: &'static [Shape],
    /// CSS color probability distribution. Repeated colors are more likely.
    #[prop_or(&["#26ccff", "#a25afd", "#ff5e7e", "#88ff5a", "#fcff42", "#ffa62d", "#ff36ff"])]
    pub colors: &'static [&'static str],
    #[prop_or(true)]
    pub disable_for_reduced_motion: bool,
    #[prop_or(1.0)]
    pub scalar: f32,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or(None)]
    pub style: Option<AttrValue>,
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
        let mut last_time = 0.0;
        let spawn_period = props.lifespan / props.particle_count as f32;
        let mut spawn_credits = 0.0;
        state_2.borrow_mut().callback = Some(Closure::new(move |time: f64| {
            let delta = ((time - last_time) * 0.001).clamp(0.0, 1.0) as f32;
            last_time = time;
            spawn_credits += delta;
            {
                context.reset();
                let mut state = state.borrow_mut();

                while spawn_credits > spawn_period {
                    spawn_credits -= spawn_period;
                    state.confetti.push(Fetti::new(&props));
                }
                state
                    .confetti
                    .retain_mut(|fetti| fetti.update(delta, &props, &context));
                //gloo_console::log!("{}", state.confetti.len());
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
            velocity: props.start_velocity + (0.5 + 0.5 * js_sys::Math::random() as f32),
            angle_2d: props.angle + rand_range(-props.spread, props.spread),
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
        self.y += (self.angle_2d.sin() * self.velocity + props.gravity) * delta;
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
        let wobble_x = self.x + self.wobble.cos() * props.scalar;
        let wobble_y = self.y + self.wobble.sin() * props.scalar;
        let tilt_sin = self.tilt_angle.sin();
        let tilt_cos = self.tilt_angle.cos();

        let random = rand_range(2.0, 3.0);
        let x1 = self.x + tilt_cos * random;
        let y1 = self.y + tilt_sin * random;
        let x2 = wobble_x + tilt_cos * random;
        let y2 = wobble_y + tilt_sin * random;

        context.set_fill_style_str(&self.color);
        // TODO: Dirty state.
        context.set_global_alpha((self.life_remaining / props.lifespan) as f64);

        context.begin_path();
        match self.shape {
            Shape::Circle => {
                let _ = context.ellipse(
                    self.x as f64,
                    self.y as f64,
                    ((x2 - x1).abs() * 0.5) as f64,
                    ((y2 - y1).abs() * 0.5) as f64,
                    self.wobble as f64,
                    0.0,
                    std::f64::consts::TAU,
                );
            }
            Shape::Square => {
                context.move_to(self.x.floor() as f64, self.y.floor() as f64);
                context.line_to(wobble_x.floor() as f64, y1 as f64);
                context.line_to(x2.floor() as f64, y2.floor() as f64);
                context.line_to(x1.floor() as f64, wobble_y.floor() as f64);
            }
        }

        context.close_path();
        context.fill();
    }
}
