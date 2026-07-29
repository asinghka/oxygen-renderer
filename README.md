# Oxygen

A real-time [glTF](https://www.khronos.org/gltf/) viewer and renderer built from scratch in Rust on [wgpu](https://github.com/gfx-rs/wgpu). Oxygen presents the
renderer as a small game-engine editor: load a model, inspect its scene graph, fly through the
viewport, and tune its shading live.

<p align="center">
  <img src="assets/media/dragon-demo.gif" alt="Oxygen rendering a dragon model">
</p>

## How it works

Oxygen imports a glTF scene into a node tree and expands its meshes into independently drawable
primitives. Each primitive owns its vertex and index buffers together with a model/normal-matrix
uniform and texture bindings. The renderer then records a shadow pass followed by the scene pass
into an offscreen texture displayed directly in egui.

- **glTF scene graphs and materials.** The loader preserves node hierarchy and accumulated
  transforms, reads position, normal, UV, and tangent attributes, and loads base-color, normal, and
  metallic-roughness textures. Color textures are uploaded as sRGB. Normal, metallic-roughness, and other data textures remain linear.
- **Physically based shading.** A Cook–Torrance specular BRDF (GGX normal distribution, Smith
  geometry term, Schlick–Fresnel) paired with a Lambertian diffuse term weighted so energy is
  conserved. Metallic and roughness come from the glTF factors, multiplied by the blue and green
  channels of the metallic-roughness texture when a material supplies one.
- **Blinn–Phong lighting.** Adjustable ambient and
  diffuse terms with a view-dependent specular highlight. Light azimuth, elevation, strengths, and
  shininess are all live editor controls.
- **Tangent-space normal mapping.** Tangents include the handedness needed to reconstruct the
  bitangent, giving normal maps the correct orientation under transformed geometry. A per-material
  scale and a global bump-strength control adjust the effect.
- **Shadow mapping.** A depth-only pass renders the scene from a directional light into a
  `Depth32Float` shadow map at an adjustable resolution (4096² by default). The main pass uses a
  hardware comparison sampler with a percentage-closer filtering kernel to test light-space depth.
  Slope and constant depth bias reduce self-shadowing artifacts.
- **GPU-native editor viewport.** The scene is rendered to an sRGB texture, registered with egui,
  and drawn in the editor without a CPU readback. The hierarchy also supports per-node visibility.

## Render modes

Shading can be switched between physically based and Blinn–Phong, alongside views useful for
inspecting geometry:

| Color | Normal |
| :-: | :-: |
| ![Color render](assets/media/screen_color.png) | ![World-space normal render](assets/media/screen_normal.png) |
| Depth | Wireframe |
| ![Depth render](assets/media/screen_depth.png) | ![Wireframe render](assets/media/screen_wireframe.png) |

## Getting started

Install the Rust toolchain, then run:

```sh
cargo run --release
```

Use **File → Load file…** to open a binary glTF (`.glb`) scene. Sample scenes are included in
[`assets/glTF/`](assets/glTF/).

## Controls

- **Right mouse drag:** look around the viewport
- **W / A / S / D:** move forward / left / backward / right
- **Q / E:** move up / down
- **Scroll or pinch:** adjust field of view
- **View → Reset Camera:** restore the camera projection after resizing

## Features

- Binary glTF loading with node transforms, indexed meshes, base-color, normal and
  metallic-roughness textures
- Cook–Torrance physically based shading with metallic-roughness materials
- Adjustable Blinn–Phong shading with live ambient, diffuse, specular, and shininess controls
- Directional-light shadow mapping with a resolution slider and percentage-closer filtering
- Tangent-space normal mapping with per-material scale and a global bump-strength control
- Color, normal, depth, and hardware line-rasterized wireframe views
- Per-primitive uniforms, depth testing, back-face culling, and an sRGB render target
- Scene-tree visibility toggles, grid overlay, background-color picker, frame time, and geometry
  statistics

## Credits

The bundled glTF scenes are from the [Khronos glTF Sample Assets](https://github.khronos.org/glTF-Assets/) project.
