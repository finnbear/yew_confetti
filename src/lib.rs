use js_sys::wasm_bindgen::{prelude::Closure, JsCast};
use std::ops::Range;
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
    last_raw_time: Option<f64>,
    last_time: u64,
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
    /// How to emit particles.
    #[prop_or_default]
    pub mode: Mode,
}

/// How to emit particles. Times are precise to the nearest millisecond.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mode(ModeImpl);

impl Default for Mode {
    fn default() -> Self {
        Self::continuous(100)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[doc(hidden)]
pub enum ModeImpl {
    /// Emit all particles at a certain time.
    Burst {
        /// How many particles to emit.
        count: usize,
        /// Time, in seconds since first render.
        delay: u64,
    },
    /// Constant stream of particles.
    Continuous {
        /// How many particles are emitted per second. Max is 1000.
        rate: u16,
        /// When the particles start being emitted, in seconds since first render.
        start: u64,
        /// When the particles stop being emitted, in seconds since first render.
        end: u64,
    },
}

fn round_time(seconds: f32) -> u64 {
    (seconds * 1000.0).round() as u64
}

impl Mode {
    /// Emit `count` particles upon first render.
    pub fn burst(count: usize) -> Self {
        Self(ModeImpl::Burst { count, delay: 0 })
    }

    /// Emit `count` particles after `delay` seconds after first render.
    pub fn delayed_burst(count: usize, delay: f32) -> Self {
        assert!(delay >= 0.0);
        Self(ModeImpl::Burst {
            count,
            delay: round_time(delay),
        })
    }

    pub fn is_burst(&self) -> bool {
        matches!(self.0, ModeImpl::Burst { .. })
    }

    /// Constantly emit `rate` particles per second.
    ///
    /// # Panics
    /// - If `rate` > 1000.
    pub fn continuous(rate: usize) -> Self {
        assert!(rate <= 1000);
        Self(ModeImpl::Continuous {
            rate: rate as u16,
            start: 0,
            end: u64::MAX,
        })
    }

    /// Constantly emit `rate` particles per second, starting `delay` seconds after first render.
    ///
    /// # Panics
    /// - If `rate` > 1000.
    /// - If `delay` isn't positive.
    pub fn delayed_continuous(rate: usize, delay: f32) -> Self {
        assert!(rate <= 1000);
        assert!(delay >= 0.0);
        Self(ModeImpl::Continuous {
            rate: rate as u16,
            start: round_time(delay) as u64,
            end: u64::MAX,
        })
    }

    /// Constantly emit `rate` particles per second, for the first `duration` seconds after first render.
    ///
    /// # Panics
    /// - If `rate` > 1000.
    /// - If `duration` isn't positive.
    pub fn finite_continuous(rate: usize, duration: f32) -> Self {
        assert!(rate <= 1000);
        assert!(duration >= 0.0);
        Self(ModeImpl::Continuous {
            rate: rate as u16,
            start: 0,
            end: round_time(duration),
        })
    }

    /// Constantly emit `rate` particles per second, starting `delay` seconds after first render
    /// and for `duration` seconds thereafter.
    ///
    /// # Panics
    /// - If `rate` > 1000.
    /// - If `delay` isn't positive.
    /// - If `duration` isn't positive.
    pub fn delayed_finite_continuous(rate: usize, delay: f32, duration: f32) -> Self {
        assert!(rate <= 1000);
        assert!(delay >= 0.0);
        assert!(duration >= 0.0);
        Self(ModeImpl::Continuous {
            rate: rate as u16,
            start: round_time(delay),
            end: round_time(delay + duration),
        })
    }

    pub fn is_continuous(&self) -> bool {
        matches!(self.0, ModeImpl::Continuous { .. })
    }

    #[doc(hidden)]
    pub fn impl_ref(&self) -> &ModeImpl {
        &self.0
    }

    #[doc(hidden)]
    pub fn impl_mut(&mut self) -> &mut ModeImpl {
        &mut self.0
    }
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
        state_2.borrow_mut().callback = Some(Closure::new(move |raw_time: f64| {
            let mut state = state.borrow_mut();

            let mut total_delta_time = (raw_time - state.last_raw_time.unwrap_or(raw_time)) as u64;
            // TODO: use lifespan instead of constant?
            if total_delta_time > 500 {
                // Skip some time.
                state.last_time += total_delta_time - 500;
                total_delta_time = 500;
            }
            state.last_raw_time = Some(raw_time);
            let substeps = (total_delta_time / 100).max(1);
            let delta_time = total_delta_time / substeps;
            let raw_delta = delta_time as f32 * 0.001;
            for _ in 0..substeps {
                // Inclusive.
                let start_time = state.last_time;
                // Exclusive.
                let end_time = start_time + delta_time;
                for cannon in props.children.iter() {
                    let count = match cannon.props.mode.0 {
                        ModeImpl::Burst { count, delay } => {
                            if (start_time..end_time).contains(&delay) {
                                count
                            } else {
                                0
                            }
                        }
                        ModeImpl::Continuous { rate, start, end } => {
                            /*
                            fn main() {
                                let mut sus: [u16; 1000] = std::array::from_fn(|i| i as u16);
                                fn key(n: u16) -> u16 {
                                    (1..100).filter(|k| n % ((1000 + k - 1) / k) == 0).min().unwrap_or(u16::MAX)
                                }
                                sus.sort_by_key(|n| key(*n));
                                let mut hash = 0u16;
                                for i in 0..1000 {
                                    hash = hash.wrapping_mul(37).wrapping_add(i);
                                    let i0 = (hash % 1000) as usize;
                                    hash = hash.wrapping_mul(79).wrapping_add(i);
                                    let i1 = (hash % 1000) as usize;
                                    if key(sus[i0]) == key(sus[i1]) {
                                        sus.swap(i0 as usize, i1 as usize);
                                    }
                                }
                                // println!("{sus:?}");
                                let ret: [u16; 1000] = std::array::from_fn(|i| sus.iter().position(|k| *k == i as u16).unwrap() as u16);
                                println!("{ret:?}");
                            }
                            */
                            #[rustfmt::skip]
                            static ORDER: [u16; 1000] = [0, 603, 549, 550, 551, 813, 553, 758, 751, 556, 557, 534, 524, 513, 504, 497, 491, 474, 469, 452, 447, 439, 433, 418, 415, 406, 397, 383, 381, 352, 346, 315, 306, 288, 268, 255, 248, 991, 228, 210, 201, 552, 196, 773, 180, 498, 159, 561, 147, 562, 139, 475, 398, 121, 384, 535, 117, 453, 353, 101, 347, 563, 316, 91, 307, 514, 289, 77, 269, 419, 256, 564, 65, 565, 968, 407, 229, 54, 211, 567, 202, 385, 690, 569, 45, 476, 695, 368, 181, 571, 348, 35, 160, 317, 881, 454, 148, 894, 505, 290, 31, 574, 270, 575, 399, 257, 122, 932, 249, 577, 434, 866, 23, 649, 230, 420, 355, 212, 102, 477, 203, 536, 580, 581, 318, 19, 92, 582, 308, 996, 400, 579, 182, 455, 78, 386, 271, 585, 161, 586, 258, 587, 588, 13, 66, 356, 589, 440, 570, 591, 140, 861, 231, 478, 55, 319, 213, 790, 627, 123, 204, 421, 387, 656, 596, 291, 744, 10, 46, 515, 272, 456, 808, 599, 357, 259, 183, 103, 705, 583, 250, 757, 36, 904, 162, 604, 320, 479, 605, 93, 232, 606, 149, 607, 804, 214, 382, 767, 292, 610, 6, 79, 611, 358, 273, 612, 613, 422, 401, 457, 197, 614, 124, 615, 953, 617, 67, 321, 618, 619, 184, 481, 960, 840, 24, 408, 622, 623, 233, 595, 163, 56, 359, 718, 215, 584, 107, 896, 274, 628, 150, 629, 435, 392, 630, 260, 631, 458, 322, 708, 4, 633, 47, 423, 634, 480, 309, 635, 636, 816, 402, 367, 638, 639, 185, 125, 234, 712, 88, 641, 349, 642, 275, 37, 643, 409, 164, 644, 645, 323, 118, 646, 797, 672, 985, 459, 14, 829, 68, 482, 361, 872, 652, 974, 198, 105, 654, 293, 869, 424, 32, 755, 657, 568, 235, 659, 276, 651, 57, 661, 324, 662, 216, 779, 664, 94, 715, 887, 126, 362, 205, 736, 165, 460, 251, 410, 668, 669, 725, 671, 294, 650, 673, 600, 2, 81, 25, 675, 403, 676, 277, 325, 236, 677, 624, 425, 679, 924, 363, 681, 141, 217, 186, 682, 106, 683, 684, 441, 685, 849, 69, 461, 558, 295, 38, 990, 689, 728, 166, 721, 692, 127, 326, 832, 278, 20, 694, 364, 95, 720, 237, 863, 697, 698, 151, 58, 699, 862, 701, 702, 218, 426, 119, 703, 954, 660, 187, 984, 707, 442, 7, 696, 82, 327, 709, 389, 365, 537, 279, 710, 711, 839, 560, 104, 167, 714, 310, 772, 238, 716, 48, 939, 992, 693, 128, 411, 590, 999, 722, 15, 723, 724, 70, 845, 328, 366, 726, 427, 727, 936, 188, 96, 280, 548, 525, 730, 731, 982, 26, 733, 142, 538, 949, 735, 640, 39, 239, 737, 738, 390, 168, 739, 59, 740, 376, 329, 741, 742, 219, 83, 743, 780, 108, 539, 745, 412, 281, 129, 746, 747, 152, 516, 748, 428, 189, 749, 391, 750, 880, 625, 261, 827, 526, 354, 240, 296, 330, 754, 688, 756, 1, 11, 885, 598, 49, 759, 169, 220, 602, 761, 282, 762, 311, 388, 763, 764, 527, 540, 506, 765, 206, 620, 369, 976, 691, 262, 912, 331, 153, 429, 130, 109, 241, 517, 770, 573, 84, 846, 962, 60, 252, 774, 753, 566, 283, 777, 40, 778, 678, 554, 143, 370, 170, 781, 782, 499, 783, 826, 332, 518, 27, 297, 785, 786, 528, 787, 788, 97, 831, 937, 242, 791, 16, 864, 507, 413, 71, 793, 284, 794, 371, 795, 796, 131, 784, 221, 798, 621, 50, 333, 110, 952, 492, 801, 298, 263, 802, 830, 171, 680, 8, 805, 508, 85, 806, 541, 807, 593, 243, 372, 809, 519, 253, 632, 811, 500, 61, 812, 594, 814, 334, 393, 815, 686, 154, 21, 817, 299, 888, 483, 98, 819, 820, 821, 822, 823, 132, 41, 373, 578, 207, 825, 898, 944, 172, 501, 244, 828, 72, 111, 144, 335, 647, 958, 766, 666, 493, 833, 509, 834, 190, 835, 836, 222, 799, 264, 470, 374, 3, 838, 86, 542, 28, 559, 837, 394, 404, 964, 842, 843, 208, 844, 336, 717, 245, 928, 510, 847, 496, 133, 173, 929, 957, 62, 850, 851, 375, 484, 789, 853, 33, 597, 223, 462, 191, 502, 855, 994, 112, 857, 616, 859, 971, 337, 199, 17, 592, 841, 626, 608, 73, 865, 246, 824, 867, 360, 300, 868, 42, 395, 776, 485, 529, 775, 871, 265, 174, 87, 472, 810, 448, 224, 134, 873, 338, 920, 875, 576, 192, 877, 5, 878, 495, 879, 377, 555, 51, 909, 882, 301, 209, 883, 884, 856, 886, 486, 891, 113, 155, 670, 63, 889, 890, 752, 471, 339, 892, 443, 893, 463, 225, 543, 175, 378, 29, 874, 895, 818, 897, 732, 899, 900, 74, 520, 901, 135, 902, 903, 200, 487, 9, 959, 905, 544, 80, 266, 340, 906, 907, 908, 350, 983, 379, 910, 436, 911, 156, 464, 769, 43, 449, 913, 914, 915, 916, 302, 114, 771, 176, 918, 919, 870, 312, 488, 921, 12, 193, 341, 922, 923, 52, 380, 601, 925, 926, 521, 473, 64, 136, 927, 145, 430, 530, 860, 511, 465, 760, 719, 18, 931, 450, 444, 876, 933, 75, 665, 935, 489, 342, 545, 351, 89, 713, 700, 177, 22, 531, 938, 800, 940, 194, 941, 99, 942, 285, 115, 943, 663, 416, 945, 946, 303, 947, 466, 948, 667, 30, 226, 950, 343, 34, 137, 437, 445, 951, 854, 858, 706, 955, 956, 44, 637, 157, 546, 609, 503, 729, 648, 286, 961, 178, 930, 963, 522, 53, 414, 917, 965, 313, 966, 344, 467, 967, 655, 969, 490, 76, 970, 90, 653, 451, 972, 973, 431, 116, 100, 438, 852, 532, 523, 146, 975, 120, 658, 138, 803, 978, 304, 979, 980, 158, 345, 405, 981, 934, 768, 179, 687, 195, 468, 792, 986, 254, 987, 988, 227, 494, 989, 704, 547, 267, 572, 734, 993, 417, 674, 287, 446, 247, 432, 305, 995, 314, 848, 512, 997, 533, 998, 977, 396];

                            let effective_start_time = start_time.max(start);
                            let effective_end_time = end_time.min(end);
                            if rate > 0 && effective_end_time > effective_start_time {
                                //let relative_start_time = effective_start_time % 1000;
                                //let effective_delta_time = effective_end_time - effective_start_time;
                                (effective_start_time..effective_end_time)
                                    .filter(|effective_time| {
                                        rate > ORDER[(effective_time % 1000) as usize]
                                    })
                                    .count()
                            } else {
                                0
                            }
                        }
                    };
                    for _ in 0..count {
                        state.confetti.push(Fetti::new(&props, &cannon.props));
                    }
                }
                state.last_time = end_time;

                state
                    .confetti
                    .retain_mut(|fetti| fetti.update(raw_delta, &props));
            }

            context.reset();
            for fetti in &state.confetti {
                fetti.draw(&props, &context);
            }

            let done = state.confetti.is_empty()
                && props.children.iter().all(|c| match c.props.mode.0 {
                    ModeImpl::Burst { delay, .. } => state.last_time > delay,
                    ModeImpl::Continuous { end, .. } => state.last_time > end,
                });
            if done {
                state.last_raw_time = None;
                state.animation_frame = None;
            } else {
                state.animation_frame =
                    Some(request_animation_frame(state.callback.as_ref().unwrap()));
            }
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

    fn update(&mut self, delta: f32, props: &ConfettiProps) -> bool {
        self.x += (self.angle_2d.cos() * self.velocity + props.drift) * delta;
        self.y += (self.angle_2d.sin() * self.velocity - props.gravity) * delta;
        self.velocity *= props.decay.powf(delta);
        self.wobble += self.wobble_speed * delta;
        self.tilt_angle += 0.1 * delta;
        self.life_remaining -= delta;
        if self.life_remaining > 0.0 {
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
