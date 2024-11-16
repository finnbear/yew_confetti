# yew_confetti

[![Documentation](https://docs.rs/yew_confetti/badge.svg)](https://docs.rs/yew_confetti)
[![crates.io](https://img.shields.io/crates/v/yew_confetti.svg)](https://crates.io/crates/yew_confetti)
[![Build](https://github.com/finnbear/yew_confetti/actions/workflows/build.yml/badge.svg)](https://github.com/finnbear/yew_confetti/actions/workflows/build.yml) 
[![Test Page](https://img.shields.io/badge/Test-page-green)](https://finnbear.github.io/yew_confetti/)

Confetti animation for Yew websites, inspired by [canvas-confetti](https://github.com/catdad/canvas-confetti).

## Usage

```rust
use yew::html;
use yew_confetti::{Confetti, Cannon};

// Defaults, except style prop.
// Shape and color props omitted.
html!{
    <Confetti
        width={256}
        height={256}
        count={150}
        decay={0.3}
        drift={0}
        gravity={1}
        lifespan={2.5}
        scalar={5}
        style={"background-color: black; width: 256px; height: 256px;"}
    >
        <Cannon
            x={0.5}
            y={0.5}
            angle={1.5707964}
            spread={0.7853982}
            velocity={2}
            continuous={true}
        />
    </Confetti>
}
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
