# `wgsl_plus`
A simple preprocessor for the WebGpu Shading Language.

- [`wgsl_plus`](#wgsl_plus)
  - [Why?](#why)
  - [Usage](#usage)
  - [Syntax](#syntax)
    - [Conditional code](#conditional-code)
        - [Example:](#example)
    - [Constants](#constants)
        - [Example:](#example-1)
    - [Include](#include)
        - [Example:](#example-2)

## Why?

I needed a preprocessor for some libraries I'm making but found that
`wgsl` will not get an official preprocessor, at least not until 1.0. There are
other options like `wgsl_preprocessor` and `bevy`'s preprocessor, but 
`wgsl_preprocessor` does not have support for conditional code, and `bevy`'s
preprocessor isn't available as a seperate package.

## Usage

To use `wgsl_plus`'s features, you need to create a `WgslWorkspace` and load
the source code of your shaders into it, this can be done by pointing it to a
folder containing your shaders, or by manually specifying shaders and their
paths:

Automatically:

```rs
let mut workspace = WgslWorkspace::scan("shaders").unwrap();
```

Manually:

```rs
let mut workspace = WgslWorkspace::from_memory("shaders", &[
    ("my-shader.wgsl", include_str!("shaders/my-shader.wgsl")),
    ("vertex.wgsl", include_str!("shaders/vertex.wgsl")),
]).unwrap();
```

Then just request the shader like this:

```rs
let shader = workspace.get_shader("my-shader.wgsl").unwrap();
```

This will give you a string containing the source code of your preprocessed
shader, that's it.

Now you can set variables to use in your shaders like this:

```rs
workspace.set_global_i64("SAMPLE_SIZE", 64);
workspace.set_global_f64("QUALITY", 5.0);
workspace.set_global_bool("DO_STUFF", false);
```

## Syntax

### Conditional code

> WGSL Syntax:
> 
> ```rs
> //:if <condition>
> ...
> //:else
> ...
> //:end
> ```

Include or exclude a piece of code if a certain condition is true.

##### Example:

Shader code

```rs
fn calculate_lighting(idx: u32) -> vec3<f32> {
    let light = lights[idx];

    //:if quality >= 4.0
    let result = do_some_fancy_stuff(light);
    //:else
    let result = do_the_cheaper_version(light);
    //:end

    return result;
}
```

Rust code:

```rs
workspace.set_global_f64("quality", 5.0);
```

Resulting shader:

```rs
fn calculate_lighting(idx: u32) -> vec3<f32> {
    let light = lights[idx];

    let result = do_some_fancy_stuff(light);

    return result;
}
```

### Constants

> WGSL Syntax:
>
> ```rs
> //:const <name>
> ```

Insert a variable into the shader as a constant.

##### Example:

Shader code:

```rs
//:const SAMPLE_SIZE
```

Rust code

```rs
workspace.set_global_i64("SAMPLE_SIZE", 64);
```

Resulting shader:

```rs
const SAMPLE_SIZE = 64;
```

### Include

> WGSL Syntax:
>
> ```rs
> //:include <path>
> ```

Include a file into this shader (path is relative to the
workspace root).

##### Example:

Shader code (`main.wgsl`):

```rs
const SOME_CONSTANT = 15;

//:include math.wgsl

fn pi_times_constant() -> f32 {
    return SOME_CONSTANT * PI;
}
```

Shader code (`math.wgsl`):

```rs
const PI: f32 = 3.1415926535897932384626433832795;

fn pi_multiplied_by(number: f32) -> f32 {
    return PI * number;
}
```

Resulting shader:

```rs
const SOME_CONSTANT = 15;

const PI: f32 = 3.1415926535897932384626433832795;

fn pi_multiplied_by(number: f32) -> f32 {
    return PI * number;
}

fn pi_times_constant() -> f32 {
    return SOME_CONSTANT * PI;
}
```
