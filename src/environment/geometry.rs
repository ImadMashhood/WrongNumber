use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    mesh::{Indices, VertexAttributeValues},
    prelude::*,
    render::render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat},
};
use std::f32::consts::{FRAC_PI_2, PI};
#[derive(Clone, Copy)]
pub enum Paint {
    Brick,
    Plaster,
    Floor,
    Wood,
    Metal,
    Plastic,
    Glass,
    Warm,
    Cool,
    Paper,
    Green,
    Red,
    Cork,
    Cream,
}
pub struct Palette(Vec<Handle<StandardMaterial>>);
impl Palette {
    pub fn new(materials: &mut Assets<StandardMaterial>, images: &mut Assets<Image>) -> Self {
        let mut out = Vec::new();
        for (color, glow, alpha) in [
            (Color::srgb(0.58, 0.29, 0.19), 0.0, 1.0),
            (Color::srgb(0.40, 0.36, 0.31), 0.0, 1.0),
            (Color::srgb(0.43, 0.41, 0.36), 0.0, 1.0),
            (Color::srgb(0.39, 0.24, 0.12), 0.0, 1.0),
            (Color::srgb(0.08, 0.09, 0.1), 0.0, 1.0),
            (Color::srgb(0.16, 0.18, 0.20), 0.0, 1.0),
            (Color::srgb(0.28, 0.43, 0.48), 0.0, 0.12),
            (Color::srgb(1.0, 0.70, 0.33), 3.0, 1.0),
            (Color::srgb(0.25, 0.60, 0.72), 0.55, 1.0),
            (Color::srgb(0.77, 0.73, 0.61), 0.0, 1.0),
            (Color::srgb(0.16, 0.27, 0.13), 0.0, 1.0),
            (Color::srgb(0.55, 0.08, 0.045), 0.0, 1.0),
            (Color::srgb(0.34, 0.22, 0.12), 0.0, 1.0),
            (Color::srgb(0.65, 0.62, 0.51), 0.0, 1.0),
        ] {
            let lin = color.to_linear();
            out.push(materials.add(StandardMaterial {
                base_color: color.with_alpha(alpha),
                perceptual_roughness: 0.88,
                emissive: LinearRgba::new(lin.red * glow, lin.green * glow, lin.blue * glow, 1.0),
                alpha_mode: if alpha < 1.0 {
                    AlphaMode::Blend
                } else {
                    AlphaMode::Opaque
                },
                cull_mode: None,
                ..default()
            }));
        }
        for (index, brick) in [(0, true), (2, false)] {
            let mut data = Vec::new();
            for y in 0..128 {
                for x in 0..128 {
                    let row = y / 32;
                    let shifted = (x + if row % 2 == 0 { 0 } else { 32 }) % 128;
                    let (line, base) = if brick {
                        (y % 32 < 2 || shifted % 64 < 2, [151_u8, 79, 55])
                    } else {
                        (x % 64 < 2 || y % 64 < 2, [103, 101, 93])
                    };
                    let shade = if brick {
                        ((row * 7 + shifted / 64 * 13) % 17) as u8
                    } else {
                        ((x * 3 + y * 5) % 3) as u8
                    };
                    let rgb = if line {
                        [50, 48, 43]
                    } else {
                        [base[0] - shade, base[1] - shade / 2, base[2] - shade / 2]
                    };
                    data.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
                }
            }
            let mut image = Image::new_fill(
                Extent3d {
                    width: 128,
                    height: 128,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                &data,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            );
            image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                ..default()
            });
            let mut material = materials.get_mut(&out[index]).unwrap();
            material.base_color = Color::WHITE;
            material.base_color_texture = Some(images.add(image));
        }
        Self(out)
    }
    pub fn get(&self, paint: Paint) -> Handle<StandardMaterial> {
        self.0[paint as usize].clone()
    }
}
pub struct Builder<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub meshes: &'a mut Assets<Mesh>,
    pub palette: Palette,
    pub cube: Handle<Mesh>,
    pub cylinder: Handle<Mesh>,
    pub cone: Handle<Mesh>,
}
impl<'a, 'w, 's> Builder<'a, 'w, 's> {
    pub fn new(
        commands: &'a mut Commands<'w, 's>,
        meshes: &'a mut Assets<Mesh>,
        palette: Palette,
    ) -> Self {
        let cube = meshes.add(Cuboid::default());
        let cylinder = meshes.add(Cylinder::new(0.5, 1.0).mesh().resolution(10));
        let cone = meshes.add(Cone::new(0.5, 1.0).mesh().resolution(10));
        Self {
            commands,
            meshes,
            palette,
            cube,
            cylinder,
            cone,
        }
    }
    pub fn block(&mut self, p: Paint, pos: Vec3, size: Vec3) -> Entity {
        self.rotated(p, pos, size, Quat::IDENTITY)
    }
    pub fn rotated(&mut self, p: Paint, pos: Vec3, size: Vec3, rotation: Quat) -> Entity {
        self.commands
            .spawn((
                Mesh3d(self.cube.clone()),
                MeshMaterial3d(self.palette.get(p)),
                Transform::from_translation(pos)
                    .with_scale(size)
                    .with_rotation(rotation),
            ))
            .id()
    }
    pub fn round(&mut self, p: Paint, pos: Vec3, size: Vec3) -> Entity {
        self.commands
            .spawn((
                Mesh3d(self.cylinder.clone()),
                MeshMaterial3d(self.palette.get(p)),
                Transform::from_translation(pos).with_scale(size),
            ))
            .id()
    }
    pub fn plane(&mut self, p: Paint, pos: Vec3, size: Vec2, rotation: Quat) {
        let mut mesh = Mesh::from(Rectangle::new(size.x, size.y));
        let scale = if matches!(p, Paint::Brick) { 0.72 } else { 1.5 };
        if let Some(VertexAttributeValues::Float32x2(uvs)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
        {
            for uv in uvs {
                uv[0] *= size.x / scale;
                uv[1] *= size.y / scale;
            }
        }
        self.commands.spawn((
            Mesh3d(self.meshes.add(mesh)),
            MeshMaterial3d(self.palette.get(p)),
            Transform::from_translation(pos).with_rotation(rotation),
        ));
    }
    pub fn picture(
        &mut self,
        material: Handle<StandardMaterial>,
        pos: Vec3,
        size: Vec2,
        rotation: Quat,
        crop: Option<(f32, f32)>,
    ) {
        let mut mesh = Mesh::from(Rectangle::new(size.x, size.y));
        if let Some((left, right)) = crop
            && let Some(VertexAttributeValues::Float32x2(uvs)) =
                mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
        {
            for uv in uvs {
                uv[0] = left + uv[0] * (right - left);
            }
        }
        self.commands.spawn((
            Mesh3d(self.meshes.add(mesh)),
            MeshMaterial3d(material),
            Transform::from_translation(pos).with_rotation(rotation),
        ));
    }
    pub fn chair(&mut self, pos: Vec3, angle: f32) {
        let r = Quat::from_rotation_y(angle);
        let mut part = |paint, offset, size| {
            self.rotated(paint, pos + r * offset, size, r);
        };
        part(
            Paint::Plastic,
            Vec3::new(0.0, 0.49, 0.0),
            Vec3::new(0.62, 0.13, 0.56),
        );
        part(
            Paint::Plastic,
            Vec3::new(0.0, 0.88, 0.29),
            Vec3::new(0.59, 0.67, 0.12),
        );
        part(
            Paint::Metal,
            Vec3::new(0.0, 0.25, 0.0),
            Vec3::new(0.08, 0.5, 0.08),
        );
        for x in [-0.35, 0.35] {
            part(
                Paint::Metal,
                Vec3::new(x, 0.64, 0.05),
                Vec3::new(0.055, 0.28, 0.055),
            );
            part(
                Paint::Plastic,
                Vec3::new(x, 0.78, -0.04),
                Vec3::new(0.1, 0.06, 0.34),
            );
        }
        for i in 0..5 {
            let a = i as f32 * PI * 2.0 / 5.0;
            let rr = Quat::from_rotation_y(a);
            self.rotated(
                Paint::Metal,
                pos + Vec3::new(a.sin() * 0.16, 0.09, a.cos() * 0.16),
                Vec3::new(0.045, 0.04, 0.4),
                rr,
            );
            self.block(
                Paint::Plastic,
                pos + Vec3::new(a.sin() * 0.34, 0.065, a.cos() * 0.34),
                Vec3::new(0.09, 0.1, 0.1),
            );
        }
    }
    pub fn plant(&mut self, pos: Vec3) {
        self.round(
            Paint::Metal,
            pos + Vec3::Y * 0.23,
            Vec3::new(0.4, 0.46, 0.4),
        );
        self.block(
            Paint::Wood,
            pos + Vec3::Y * 0.65,
            Vec3::new(0.04, 0.7, 0.04),
        );
        for i in 0..7 {
            let a = i as f32 * 2.4;
            self.rotated(
                Paint::Green,
                pos + Vec3::new(a.cos() * 0.18, 0.7 + (i % 3) as f32 * 0.18, a.sin() * 0.18),
                Vec3::new(0.2, 0.55, 0.08),
                Quat::from_euler(EulerRot::YXZ, a, 0.4, 0.35),
            );
        }
    }
    pub fn noticeboard(&mut self, pos: Vec3, rotation: Quat) {
        self.rotated(Paint::Wood, pos, Vec3::new(1.3, 0.95, 0.08), rotation);
        self.rotated(
            Paint::Cork,
            pos + rotation * Vec3::Z * 0.05,
            Vec3::new(1.17, 0.82, 0.015),
            rotation,
        );
        for (x, y) in [(-0.34, 0.13), (0.0, -0.08), (0.35, 0.16)] {
            self.rotated(
                Paint::Paper,
                pos + rotation * Vec3::new(x, y, 0.065),
                Vec3::new(0.26, 0.34, 0.01),
                rotation,
            );
            self.rotated(
                Paint::Red,
                pos + rotation * Vec3::new(x, y + 0.13, 0.08),
                Vec3::splat(0.035),
                rotation,
            );
        }
    }
    pub fn pendant(&mut self, pos: Vec3, power: f32) {
        self.block(
            Paint::Metal,
            pos + Vec3::Y * 0.35,
            Vec3::new(0.025, 0.7, 0.025),
        );
        self.block(Paint::Metal, pos, Vec3::new(0.82, 0.2, 0.58));
        self.block(
            Paint::Warm,
            pos - Vec3::Y * 0.105,
            Vec3::new(0.68, 0.025, 0.44),
        );
        self.commands.spawn((
            PointLight {
                intensity: power,
                range: 9.0,
                color: Color::srgb(1.0, 0.79, 0.58),
                ..default()
            },
            Transform::from_translation(pos - Vec3::Y * 0.2),
        ));
    }
    /// A true arched opening with a faceted header and shared repeating brick material.
    pub fn window(&mut self, center: Vec3, rotation: Quat) {
        let r = 1.05;
        let spring = 2.8;
        let local = |x, y, z| center + rotation * Vec3::new(x, y, z);
        self.plane(
            Paint::Brick,
            local(0.0, 0.4, 0.0),
            Vec2::new(2.6, 0.8),
            rotation,
        );
        // Fill only the area above the arch, leaving an actual opening.
        let mut vertices = Vec::new();
        let mut uv = Vec::new();
        let mut indices = Vec::new();
        for i in 0..12 {
            let a = PI - i as f32 * PI / 12.0;
            let b = PI - (i + 1) as f32 * PI / 12.0;
            let x1 = r * a.cos();
            let x2 = r * b.cos();
            let y1 = spring + r * a.sin();
            let y2 = spring + r * b.sin();
            let start = vertices.len() as u32;
            for (x, y) in [(x1, y1), (x2, y2), (x2, 4.5), (x1, 4.5)] {
                vertices.push([x, y, 0.0]);
                uv.push([x / 0.72, y / 0.72]);
            }
            indices.extend([start, start + 1, start + 2, start, start + 2, start + 3]);
        }
        let len = vertices.len();
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; len])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
        .with_inserted_indices(Indices::U32(indices));
        self.commands.spawn((
            Mesh3d(self.meshes.add(mesh)),
            MeshMaterial3d(self.palette.get(Paint::Brick)),
            Transform::from_translation(center).with_rotation(rotation),
        ));
        self.rotated(
            Paint::Metal,
            local(0.0, 0.84, 0.05),
            Vec3::new(2.4, 0.14, 0.42),
            rotation,
        );
        for x in [-r, r] {
            self.rotated(
                Paint::Metal,
                local(x, 1.82, 0.04),
                Vec3::new(0.07, 2.0, 0.1),
                rotation,
            );
        }
        for y in [1.46, 2.15, 2.8] {
            self.rotated(
                Paint::Metal,
                local(0.0, y, 0.04),
                Vec3::new(2.1, 0.055, 0.09),
                rotation,
            );
        }
        for x in [-0.52, 0.0, 0.52] {
            self.rotated(
                Paint::Metal,
                local(x, 1.85, 0.04),
                Vec3::new(0.045, 2.0, 0.09),
                rotation,
            );
        }
        self.rotated(
            Paint::Glass,
            local(0.0, 1.83, -0.035),
            Vec3::new(2.07, 1.93, 0.012),
            rotation,
        );
        for i in 0..11 {
            let a = (i as f32 + 0.5) * PI / 11.0;
            let rr = rotation * Quat::from_rotation_z(a - FRAC_PI_2);
            self.rotated(
                Paint::Brick,
                local(1.17 * a.cos(), spring + 1.17 * a.sin(), 0.025),
                Vec3::new(0.31, 0.19, 0.25),
                rr,
            );
            self.rotated(
                Paint::Metal,
                local(1.045 * a.cos(), spring + 1.045 * a.sin(), 0.04),
                Vec3::new(0.3, 0.055, 0.09),
                rr,
            );
        }
    }
}
