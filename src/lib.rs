use js_sys::wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};
use yew::{function_component, html, use_effect_with, use_mut_ref, use_node_ref, Html, Properties};

#[derive(Clone, PartialEq, Properties)]
pub struct ConfettiProps {
    #[prop_or(50)]
    pub particle_count: u32,
    #[prop_or(90.0)]
    pub angle: f32,
    #[prop_or(45.0)]
    pub spread: f32,
    #[prop_or(45.0)]
    pub start_velocity: f32,
    #[prop_or(0.9)]
    pub decay: f32,
    #[prop_or(1.0)]
    pub gravity: f32,
    #[prop_or(0.0)]
    pub drift: f32,
    #[prop_or(200)]
    pub ticks: u32,
    #[prop_or(0.5)]
    pub x: f32,
    #[prop_or(0.5)]
    pub y: f32,
    #[prop_or(&[Shape::Circle, Shape::Square])]
    pub shapes: &'static [Shape],
    //#[prop_or(100)]
    //pub z_index: i32,
    #[prop_or(&["#26ccff", "#a25afd", "#ff5e7e", "#88ff5a", "#fcff42", "#ffa62d", "#ff36ff"])]
    pub colors: &'static [&'static str],
    #[prop_or(true)]
    pub disable_for_reduced_motion: bool,
    #[prop_or(1.0)]
    pub scalar: f32,
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame`");
}

#[derive(Default)]
struct State {
    confetti: Vec<Fetti>,
    callback: Option<Closure<dyn FnMut()>>,
}

#[function_component(Confetti)]
pub fn confetti(props: &ConfettiProps) -> Html {
    let canvas = use_node_ref();
    let state = use_mut_ref(|| State::default());

    use_effect_with((canvas.clone(), props.clone()), move |(canvas, props)| {
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
        state_2.borrow_mut().callback = Some(Closure::new(move || {
            {
                context.reset();
                let mut state = state.borrow_mut();
                state.confetti.push(Fetti::new(&props));
                state
                    .confetti
                    .retain_mut(|fetti| fetti.update(&props, &context));
                //gloo_console::log!("{}", state.confetti.len());
            }

            request_animation_frame(state.borrow().callback.as_ref().unwrap());
        }));

        request_animation_frame(state_2.borrow().callback.as_ref().unwrap());

        move || drop(state_2.borrow_mut().callback.take())
    });

    html! {
        <canvas ref={canvas} width={512} height={512}/>
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
    ticks_remaining: u32,
    random: f32,
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
            angle_2d: props.angle,
            tilt_angle: rand_max(std::f32::consts::TAU),
            color: props.colors[rand_max(props.colors.len() as f32) as usize],
            shape: props.shapes[rand_max(props.shapes.len() as f32) as usize],
            ticks_remaining: props.ticks,
            random: rand_range(2.0, 3.0),
        }
    }

    fn update(&mut self, props: &ConfettiProps, context: &CanvasRenderingContext2d) -> bool {
        self.x += self.angle_2d.cos() * self.velocity + props.drift;
        self.y += self.angle_2d.sin() * self.velocity + props.gravity;
        self.velocity *= props.decay;
        self.wobble += self.wobble_speed;
        self.tilt_angle += 0.1;

        if let Some(ticks_remaining) = self.ticks_remaining.checked_sub(1) {
            self.ticks_remaining = ticks_remaining;
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

        let x1 = self.x;
        let y1 = self.y;
        let x2 = wobble_x;
        let y2 = wobble_y;

        context.set_fill_style_str(&self.color);
        // TODO: Dirty state.
        context.set_global_alpha((self.ticks_remaining as f32 / props.ticks as f32) as f64);

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
