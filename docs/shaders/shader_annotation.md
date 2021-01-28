# Shader Annotation

This is a reference for all shader annotations that `rafx-shader-processor` understands.

The annotation system can be ignored if you only use `rafx-api` or systems in `rafx-framework` that do not require the
additional metadata.

## Syntax

Annotations are in the form of **comments**. This ensures that most other tools (like syntax-highlighting in editors)
can readily parse the shader. Both single-line and multi-line comments are supported. Multiple annotations on the same
value are allowed

```c
// @[export]
// @[slot_name("blur_texture")]
layout (set = 0, binding = 1) uniform texture2D in_blur;
```

An annotation may include parameters. This parameter can be a single value or a struct in RON format.

```c
/* 
@[immutable_samplers([
    (
        mag_filter: Nearest,
        min_filter: Nearest,
        mip_map_mode: Linear,
        address_mode_u: ClampToEdge,
        address_mode_v: ClampToEdge,
        address_mode_w: ClampToEdge,
    )
])]
*/
layout (set = 0, binding = 1) uniform sampler smp;
```

Annotations always affect the binding that comes after it.

## Summary of All Annotations

* **@[internal_buffer]**: Applied to UNIFORM data: Automatically binds space in a buffer, making it easy to quickly set these
  values 
* **@[export]**: Include bindings for this variable in the generated rust code
* **@[slot_name(...)]**: Overrides the shader field name with a custom name. This affects generated rust code and the
  name of the variable in the generated reflection data
* **@[immutable_samplers(...)]**: Automatically binds sampler(s) to the variable
* **@[semantic(...)]**: Used for vertex inputs, used to automatically generate pipelines to bind different `VertexDataSetLayout`s to
  this variable
  
## Reference Documentation

### @[internal_buffer]

(**Requires using `DescriptorSetAllocatorManager` in `rafx-framework`!**)

This annotation automatically binds a buffer to a uniform variable. `DescriptorSetAllocatorManager` allocates descriptor
sets in pooled chunks. A single buffer is used for all descriptors for the same variable in the chunk.

When combined with `@[export]` (which [generates rust code](generated_rust_code.md) to set the data and structs that 
match the data format) this results in an easy-to-use, type-safe interface for setting uniform data in shaders.

#### Example Usage

```c
// @[internal_buffer]
layout (set = 0, binding = 0) uniform PerViewData {
    vec4 uniform_color;
} uniform_data;
```

### @[export]

Exports the annotated field to rust code. If the field references other structs, they will also be exported. Export
supports uniform data, textures, and most other kinds of shader fields.

Shader code may have multiple memory layouts for a single struct. In this case, exporting a single shader struct might
produce multiple rust structs. For example, an exported `PointLight` in a shader might produce `PointLightStd140` and
`PointLightStd430` in rust code. In most cases, only one layout is required. But even multiple are required, the
interface for setting descriptor sets is type-safe and will only accept the correct one.

#### Example Usage

```c
// @[export]
layout (set = 0, binding = 0) uniform texture2D in_color;
```

### @[slot_name("...")]

By default, reflection data and generated rust code will infer a name from shader code. However, a name can be specified
manually. Overriding the name allows shader variables to be renamed without breaking dependent rust code or other
references to the name that might be stored in asset data.

#### Example Usage

```c
// @[slot_name("blur_texture")]
layout (set = 0, binding = 1) uniform texture2D in_blur;
```

### @[immutable_samplers(...)]

(**Requires using `DescriptorSetAllocatorManager` in `rafx-framework`!**)

Automatically create and bind the defined sampler to the annotated field in the shader. The samplers are hashed so
that many shaders share the same sampler if they have the same definition.

The `...` in `@[immutable_samplers(...)]` should be a `Vec<RafxSamplerDef>` in RON format. It would be something like
this:

```c
// @immutable_samplers([ 
//     (fields for sampler 1), 
//     (fields for sampler 2), 
// ])
```

#### Example Usage

```c
// @[immutable_samplers([
//     (
//         mag_filter: Nearest,
//         min_filter: Nearest,
//         mip_map_mode: Linear,
//         address_mode_u: ClampToEdge,
//         address_mode_v: ClampToEdge,
//         address_mode_w: ClampToEdge,
//     )
// ])]
layout (set = 0, binding = 1) uniform sampler smp;
```

### @[semantic("...")]

A semantic annotation indicates the kind of input data that is expected. Rafx uses this to produce pipelines that map
data stored in `VertexDataSet`s to your shader. Common values include `"POSITION"`, `"NORMAL"`, `"TANGENT"` or
`"TEXCOORD"`. However, rafx does not require any particular naming convention and arbitrary strings can be used.

This annotations is **required** when generating rust code or cooked shader packages. This is because these outputs are
generally for use with `rafx-framework` and forgetting to define the semantic when using `rafx-framework` is almost
certainly a mistake.

The concept has its roots in HLSL. Many people follow the naming conventions defined here:
https://docs.microsoft.com/en-us/windows/win32/direct3dhlsl/dx-graphics-hlsl-semantics

```c
// @[semantic("POSITION")]
layout (location = 0) in vec3 in_pos;
// @[semantic("NORMAL")]
layout (location = 1) in vec3 in_normal;
// @[semantic("TANGENT")]
layout (location = 2) in vec4 in_tangent;
// @[semantic("TEXCOORD")]
layout (location = 3) in vec2 in_uv;
```