use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
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
            // Real mip levels prevent tiny mortar lines shimmering at street distance.
            let mut all = data.clone();
            let mut level = data;
            let mut width = 128usize;
            let mut levels = 1;
            while width > 1 {
                let next_width = width / 2;
                let mut next = vec![0u8; next_width * next_width * 4];
                for y in 0..next_width {
                    for x in 0..next_width {
                        for c in 0..4 {
                            let mut sum = 0u32;
                            for dy in 0..2 {
                                for dx in 0..2 {
                                    sum += u32::from(
                                        level[((2 * y + dy) * width + 2 * x + dx) * 4 + c],
                                    );
                                }
                            }
                            next[(y * next_width + x) * 4 + c] = (sum / 4) as u8;
                        }
                    }
                }
                all.extend_from_slice(&next);
                level = next;
                width = next_width;
                levels += 1;
            }
            image.data = Some(all);
            image.texture_descriptor.mip_level_count = levels;
            image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                mag_filter: ImageFilterMode::Linear,
                min_filter: ImageFilterMode::Linear,
                mipmap_filter: ImageFilterMode::Linear,
                anisotropy_clamp: 8,
                ..default()
            });
            let mut material = materials.get_mut(&out[index]).unwrap();
            material.base_color = Color::WHITE;
            material.base_color_texture = Some(images.add(image));
            if brick {
                let height = |x: i32, y: i32| {
                    let y = y.rem_euclid(128);
                    let x = (x + if (y / 32) % 2 == 0 { 0 } else { 32 }).rem_euclid(64);
                    let edge = x.min(63 - x).min(y % 32).min(31 - y % 32);
                    (edge as f32 / 3.0).clamp(0.0, 1.0)
                };
                let mut data = vec![];
                for y in 0..128 {
                    for x in 0..128 {
                        let normal = Vec3::new(
                            (height(x - 1, y) - height(x + 1, y)) * 0.35,
                            (height(x, y - 1) - height(x, y + 1)) * 0.35,
                            1.0,
                        )
                        .normalize();
                        data.extend([
                            (normal.x * 127.0 + 128.0) as u8,
                            (normal.y * 127.0 + 128.0) as u8,
                            (normal.z * 127.0 + 128.0) as u8,
                            255,
                        ]);
                    }
                }
                let mut bump = Image::new_fill(
                    Extent3d {
                        width: 128,
                        height: 128,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    &data,
                    TextureFormat::Rgba8Unorm,
                    RenderAssetUsages::default(),
                );
                bump.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    mag_filter: ImageFilterMode::Linear,
                    min_filter: ImageFilterMode::Linear,
                    ..default()
                });
                material.normal_map_texture = Some(images.add(bump));
            }
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
    pub parent: Option<Entity>,
}
impl<'a, 'w, 's> Builder<'a, 'w, 's> {
    pub fn new(
        commands: &'a mut Commands<'w, 's>,
        meshes: &'a mut Assets<Mesh>,
        palette: Palette,
    ) -> Self {
        let cube = meshes.add(Cuboid::default());
        let cylinder = meshes.add(Cylinder::new(0.5, 1.0).mesh().resolution(10));
        Self {
            commands,
            meshes,
            palette,
            cube,
            cylinder,
            parent: None,
        }
    }
    pub fn block(&mut self, p: Paint, pos: Vec3, size: Vec3) -> Entity {
        self.rotated(p, pos, size, Quat::IDENTITY)
    }
    pub fn rotated(&mut self, p: Paint, pos: Vec3, size: Vec3, rotation: Quat) -> Entity {
        let mesh = if matches!(p, Paint::Brick) {
            let origin = rotation.inverse() * pos;
            let mut mesh = Mesh::from(Cuboid::default());
            let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap() {
                VertexAttributeValues::Float32x3(v) => v.clone(),
                _ => unreachable!(),
            };
            let vertices = match mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
                VertexAttributeValues::Float32x3(v) => v.clone(),
                _ => unreachable!(),
            };
            if let Some(VertexAttributeValues::Float32x2(uvs)) =
                mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
            {
                for ((uv, n), v) in uvs.iter_mut().zip(normals).zip(vertices) {
                    *uv = if n[1].abs() > 0.5 {
                        [
                            (v[0] * size.x + origin.x) / 0.72,
                            (v[2] * size.z + origin.z) / 0.72,
                        ]
                    } else if n[0].abs() > 0.5 {
                        [
                            (v[2] * size.z + origin.z) / 0.72,
                            -(v[1] * size.y + origin.y) / 0.72,
                        ]
                    } else {
                        [
                            (v[0] * size.x + origin.x) / 0.72,
                            -(v[1] * size.y + origin.y) / 0.72,
                        ]
                    };
                }
            }
            mesh.generate_tangents().expect("Brick cuboid tangents");
            self.meshes.add(mesh)
        } else {
            self.cube.clone()
        };
        let entity = self
            .commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(self.palette.get(p)),
                Transform::from_translation(pos)
                    .with_scale(size)
                    .with_rotation(rotation),
            ))
            .id();
        if let Some(parent) = self.parent {
            self.commands.entity(parent).add_child(entity);
        }
        entity
    }
    pub fn round(&mut self, p: Paint, pos: Vec3, size: Vec3) -> Entity {
        let entity = self
            .commands
            .spawn((
                Mesh3d(self.cylinder.clone()),
                MeshMaterial3d(self.palette.get(p)),
                Transform::from_translation(pos).with_scale(size),
            ))
            .id();
        if let Some(parent) = self.parent {
            self.commands.entity(parent).add_child(entity);
        }
        entity
    }
    pub fn plane(&mut self, p: Paint, pos: Vec3, size: Vec2, rotation: Quat) {
        if matches!(p, Paint::Brick) {
            self.rotated(p, pos, Vec3::new(size.x, size.y, 0.36), rotation);
            return;
        }
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
        mesh.generate_tangents().expect("Surface tangents");
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
        let entity = self
            .commands
            .spawn((
                Mesh3d(self.meshes.add(mesh)),
                MeshMaterial3d(material),
                Transform::from_translation(pos).with_rotation(rotation),
            ))
            .id();
        if let Some(parent) = self.parent {
            self.commands.entity(parent).add_child(entity);
        }
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
    /// Watertight, 36 cm deep masonry module with an actual arched opening.
    pub fn window(&mut self, center: Vec3, rotation: Quat) {
        let r = 1.05;
        let spring = 2.8;
        let local = |x, y, z| center + rotation * Vec3::new(x, y, z);
        self.rotated(
            Paint::Brick,
            local(0.0, 0.4, 0.0),
            Vec3::new(3.2, 0.8, 0.36),
            rotation,
        );
        for x in [-1.325, 1.325] {
            self.rotated(
                Paint::Brick,
                local(x, 2.65, 0.0),
                Vec3::new(0.55, 3.7, 0.36),
                rotation,
            );
        }
        let mut vertices = vec![];
        let mut normals = vec![];
        let mut uv = vec![];
        let mut indices = vec![];
        for i in 0..24 {
            let a = PI - i as f32 * PI / 24.0;
            let b = PI - (i + 1) as f32 * PI / 24.0;
            let x1 = r * a.cos();
            let x2 = r * b.cos();
            let y1 = spring + r * a.sin();
            let y2 = spring + r * b.sin();
            for (points, normal) in [
                (
                    [
                        [x1, y1, 0.18],
                        [x2, y2, 0.18],
                        [x2, 4.5, 0.18],
                        [x1, 4.5, 0.18],
                    ],
                    [0.0, 0.0, 1.0],
                ),
                (
                    [
                        [x2, y2, -0.18],
                        [x1, y1, -0.18],
                        [x1, 4.5, -0.18],
                        [x2, 4.5, -0.18],
                    ],
                    [0.0, 0.0, -1.0],
                ),
                (
                    [
                        [x1, y1, -0.18],
                        [x2, y2, -0.18],
                        [x2, y2, 0.18],
                        [x1, y1, 0.18],
                    ],
                    [-((a + b) * 0.5).cos(), -((a + b) * 0.5).sin(), 0.0],
                ),
            ] {
                let start = vertices.len() as u32;
                for v in points {
                    vertices.push(v);
                    normals.push(normal);
                    let origin = rotation.inverse() * center;
                    uv.push([(v[0] + origin.x) / 0.72, -(v[1] + origin.y) / 0.72]);
                }
                indices.extend([start, start + 1, start + 2, start, start + 2, start + 3]);
            }
        }
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
        .with_inserted_indices(Indices::U32(indices));
        mesh.generate_tangents().expect("Arched header tangents");
        self.commands.spawn((
            Mesh3d(self.meshes.add(mesh)),
            MeshMaterial3d(self.palette.get(Paint::Brick)),
            Transform::from_translation(center).with_rotation(rotation),
        ));
        self.rotated(
            Paint::Cream,
            local(0.0, 0.83, 0.13),
            Vec3::new(2.35, 0.12, 0.65),
            rotation,
        );
        for x in [-r, r] {
            self.rotated(
                Paint::Metal,
                local(x, 1.84, 0.0),
                Vec3::new(0.065, 2.02, 0.14),
                rotation,
            );
        }
        for y in [1.46, 2.15, 2.8] {
            self.rotated(
                Paint::Metal,
                local(0.0, y, 0.0),
                Vec3::new(2.1, 0.045, 0.10),
                rotation,
            );
        }
        for x in [-0.52, 0.0, 0.52] {
            self.rotated(
                Paint::Metal,
                local(x, 1.84, 0.0),
                Vec3::new(0.035, 2.02, 0.1),
                rotation,
            );
        }
        self.rotated(
            Paint::Glass,
            local(0.0, 1.83, -0.08),
            Vec3::new(2.07, 1.93, 0.012),
            rotation,
        );
        // A fitted semicircular pane closes the fanlight above the spring line.
        let mut positions = vec![[0., spring, -0.08]];
        for i in 0..=24 {
            let angle = i as f32 * PI / 24.;
            positions.push([1.025 * angle.cos(), spring + 1.025 * angle.sin(), -0.08]);
        }
        let indices: Vec<u32> = (1..=24).flat_map(|i| [0, i, i + 1]).collect();
        let pane = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 0., 1.]; 26])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; 26])
        .with_inserted_indices(Indices::U32(indices));
        self.commands.spawn((
            Mesh3d(self.meshes.add(pane)),
            MeshMaterial3d(self.palette.get(Paint::Glass)),
            Transform::from_translation(center).with_rotation(rotation),
        ));
        for angle in [PI / 4., PI / 2., 3. * PI / 4.] {
            let direction = Vec3::new(angle.cos(), angle.sin(), 0.);
            self.rotated(
                Paint::Metal,
                local(0., spring, 0.) + rotation * direction * 0.51,
                Vec3::new(0.028, 1.02, 0.08),
                rotation * Quat::from_rotation_z(angle - PI / 2.),
            );
        }
        // Continuous faceted frame follows the same arch segments; no intersecting brick cubes.
        for i in 0..24 {
            let a = (i as f32 + 0.5) * PI / 24.0;
            self.rotated(
                Paint::Metal,
                local(r * a.cos(), spring + r * a.sin(), 0.0),
                Vec3::new(0.142, 0.055, 0.14),
                rotation * Quat::from_rotation_z(a - FRAC_PI_2),
            );
        }
        for y in [1.0, 1.42, 1.84, 2.26, 2.68] {
            for x in [-1.14, 1.14] {
                self.rotated(
                    Paint::Brick,
                    local(x, y, 0.20),
                    Vec3::new(0.18, 0.40, 0.12),
                    rotation,
                );
            }
        }
    }
}
