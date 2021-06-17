use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufReader, Cursor};

use anyhow::format_err;
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::prelude::{shape, Texture as BevyTexture, *};
use bevy::reflect::TypeUuid;
use bevy::render::mesh::Indices;
use bevy::render::pipeline::PrimitiveTopology;
use bevy::render::texture::{AddressMode, Extent3d, SamplerDescriptor};
use bevy::render::wireframe::Wireframe;
use bevy::utils::BoxedFuture;
use decoder::format::gold_src_30::Texture;
use decoder::format::GoldSrc30Bsp;
use decoder::{BspFormat, WadDecoder};

#[derive(Debug, Clone, Default)]
pub struct BspConfig {
    pub show_wireframe: bool,
}

#[derive(Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct BspMesh;

/// Adds support for Bsp file rendering
#[derive(Default)]
pub struct BspPlugin;

impl Plugin for BspPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_asset_loader::<BspFileLoader>()
            .init_resource::<BspConfig>()
            .register_type::<BspMesh>()
            .register_type::<BspNeedsWad>()
            .add_asset::<BspFile>()
            .insert_resource(WadManager::default())
            .add_startup_system(load_wads_system.system())
            .add_system(add_wireframes_system.system())
            .add_system(apply_wad_textures_system.system());
    }
}

#[derive(Debug, TypeUuid)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct BspFile;

#[derive(Default)]
pub struct BspFileLoader;

impl AssetLoader for BspFileLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let cursor = Cursor::new(bytes);
            let decoder = decoder::BspDecoder::from_reader(cursor)?;
            let format = decoder.decode_any()?;

            load_format(format, load_context)?;

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["bsp"]
    }
}

fn load_format(format: BspFormat, load_context: &mut LoadContext) -> Result<(), anyhow::Error> {
    let BspFormat::GoldSrc30(gold_src) = format;

    load_gold_src_format(gold_src, load_context)?;

    Ok(())
}

struct BspFace {
    mesh: Handle<Mesh>,
    idx_miptex: Option<usize>,
}

struct BspDebugVolume {
    mesh: Handle<Mesh>,
    mins: [f32; 3],
    maxs: [f32; 3],
}

#[derive(Debug)]
struct BspTexture {
    idx: usize,
    material: Handle<StandardMaterial>,
    is_transparent: bool,
}

#[derive(Debug, Clone, Reflect, Default)]
#[reflect(Component)]
struct BspNeedsWad {
    name: String,
}

fn load_gold_src_format(
    bsp: GoldSrc30Bsp,
    load_context: &mut LoadContext,
) -> Result<(), anyhow::Error> {
    let model = bsp
        .models
        .get(0)
        .ok_or_else(|| format_err!("No worldspawn model"))?;

    let mut nodes = vec![bsp.nodes[model.idx_head_nodes[0] as usize]];
    let mut leaves = vec![];

    while !nodes.is_empty() {
        let node = nodes.pop().unwrap();

        let front = node.idx_children[0];
        let back = node.idx_children[1];

        let mut parse = |n: i16| {
            if n < -1 {
                leaves.push(n.abs() - 1);
            } else if n >= 0 {
                nodes.push(bsp.nodes[n as usize]);
            }
        };

        parse(front);
        parse(back);
    }

    let mut textures = vec![];
    let mut faces = vec![];
    let mut debug_volumes = vec![];
    let mut wad_indexes = HashMap::new();

    // Add textures
    for (idx, texture) in bsp.textures.iter().enumerate() {
        // Skip WAD textures
        if texture.offsets[0] > 0 {
            let (transparent, texture) = parse_texture(&texture);

            let tex_handle = load_context
                .set_labeled_asset(&format!("Texture{}", idx), LoadedAsset::new(texture));

            let material = StandardMaterial {
                base_color_texture: Some(tex_handle),
                unlit: true,
                ..Default::default()
            };
            let mat_handle = load_context.set_labeled_asset(
                &format!("TextureMaterial{}", idx),
                LoadedAsset::new(material),
            );

            let texture = BspTexture {
                idx,
                material: mat_handle,
                is_transparent: transparent,
            };

            textures.push(texture);
        } else {
            let name = String::from_utf8_lossy(&texture.name).to_string();
            let name = name.split('\0').next().unwrap_or_default().to_string();

            wad_indexes.insert(idx, BspNeedsWad { name });
        }
    }

    // Add debug volumes
    for (idx, model) in bsp.models.iter().enumerate() {
        let mins = model.mins;
        let maxs = model.maxs;

        let x_length = maxs[1] - mins[1];
        let y_length = maxs[2] - mins[2];
        let z_length = maxs[0] - mins[0];
        let volume: Mesh = shape::Box::new(x_length, y_length, z_length).into();

        let mesh_label = format!("DebugVolumeMesh{}", idx);
        let mesh = load_context.set_labeled_asset(&mesh_label, LoadedAsset::new(volume));

        let debug_volume = BspDebugVolume { mesh, mins, maxs };

        debug_volumes.push(debug_volume);
    }

    // Add faces
    {
        let num_faces = model.num_faces as usize;
        let first_face = model.idx_first_face as usize;

        for face_idx in first_face..first_face + num_faces {
            let mut positions = vec![];
            let mut normals = vec![];
            let mut colors = vec![];
            let mut uvs = vec![];
            let mut idx_miptex = None;

            if let Some(face) = bsp.faces.get(face_idx) {
                let lighting = bsp.lighting.get(face.lightmap_offset as usize / 3);

                let tex_info = bsp.texture_info.get(face.texture_info as usize);
                idx_miptex = tex_info.map(|info| info.idx_miptex as usize);
                let texture = tex_info
                    .map(|info| bsp.textures.get(info.idx_miptex as usize))
                    .flatten();

                if let Some(plane) = bsp.planes.get(face.plane as usize) {
                    let mut normal = plane.normal;

                    if face.plane_side > 0 {
                        normal *= -1.0;
                    }

                    let num_edges = face.edges as usize;
                    let first_edge = face.first_edge as usize;

                    for surfedge_idx in first_edge..first_edge + num_edges {
                        if let Some(surf_edge) = bsp.surf_edges.get(surfedge_idx) {
                            let edge_idx = surf_edge.0;
                            let edge_idx_abs = edge_idx.abs() as usize;

                            if let Some(edge) = bsp.edges.get(edge_idx_abs) {
                                let mut vert0_idx = edge.vertex[0] as usize;
                                let mut vert1_idx = edge.vertex[1] as usize;

                                if edge_idx < 0 {
                                    std::mem::swap(&mut vert0_idx, &mut vert1_idx);
                                }

                                let vert0 = bsp.vertices.get(vert0_idx);

                                if let Some(vert0) = vert0 {
                                    let mut color = [0; 3];
                                    if let Some(colors) = lighting {
                                        color[0] = colors.r as u32;
                                        color[0] = colors.g as u32;
                                        color[0] = colors.b as u32;
                                    }

                                    let mut u = 0.0;
                                    let mut v = 0.0;
                                    if let (Some(tex_info), Some(texture)) = (tex_info, texture) {
                                        let s_vector = tex_info.s_vector;
                                        let t_vector = tex_info.t_vector;

                                        u = (vert0.0.dot(s_vector) + tex_info.s_shift)
                                            / texture.width as f32;

                                        v = (vert0.0.dot(t_vector) + tex_info.t_shift)
                                            / texture.height as f32;
                                    }

                                    positions.push(vert0.0);
                                    //tangents.push(tangent);
                                    normals.push(normal);
                                    colors.push(color);
                                    uvs.push([u, v]);
                                }
                            } else {
                                println!(
                                    "Can't find edge {} from surfedge {}",
                                    surf_edge.0.abs(),
                                    surfedge_idx
                                );
                            }
                        } else {
                            println!("Can't find surfedge {}", surfedge_idx);
                        }
                    }
                } else {
                    println!("Can't find plane {}", face.plane);
                }
            } else {
                println!("Can't find face {}", face_idx);
            }

            let indicies = triangulate(&positions);

            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            mesh.set_attribute(
                Mesh::ATTRIBUTE_UV_0,
                uvs.into_iter().rev().collect::<Vec<_>>(),
            );
            mesh.set_attribute(
                Mesh::ATTRIBUTE_POSITION,
                positions
                    .into_iter()
                    .rev()
                    .map(vec3tofloat3)
                    .collect::<Vec<_>>(),
            );
            mesh.set_attribute(
                Mesh::ATTRIBUTE_NORMAL,
                normals.into_iter().map(vec3tofloat3).collect::<Vec<_>>(),
            );
            mesh.set_attribute(
                Mesh::ATTRIBUTE_COLOR,
                colors.into_iter().collect::<Vec<_>>(),
            );
            mesh.set_indices(Some(Indices::U16(indicies)));

            let mesh_label = format!("Mesh{}", face_idx);
            let mesh = load_context.set_labeled_asset(&mesh_label, LoadedAsset::new(mesh));

            let face = BspFace { mesh, idx_miptex };

            faces.push(face);
        }
    }

    let mut world = World::default();
    world
        .spawn()
        .insert_bundle((Transform::identity(), GlobalTransform::identity()))
        .with_children(|parent| {
            let mut map = parent.spawn_bundle((Transform::identity(), GlobalTransform::identity()));

            map.with_children(|parent| {
                // Spawn faces
                {
                    let material: StandardMaterial = Color::DARK_GRAY.into();
                    let default_material =
                        load_context.set_labeled_asset("FaceColor", LoadedAsset::new(material));

                    for face in faces.into_iter() {
                        let texture = face
                            .idx_miptex
                            .map(|idx| textures.iter().find(|t| t.idx == idx))
                            .flatten();

                        let mut entity = parent.spawn_bundle(PbrBundle {
                            mesh: face.mesh,
                            material: texture
                                .map(|t| t.material.clone())
                                .unwrap_or_else(|| default_material.clone()),
                            visible: Visible {
                                is_transparent: texture
                                    .map(|t| t.is_transparent)
                                    .unwrap_or_default(),
                                ..Default::default()
                            },
                            ..Default::default()
                        });
                        entity.insert(BspMesh);

                        if let Some(needs_wad) = face
                            .idx_miptex
                            .map(|idx| wad_indexes.get(&idx))
                            .flatten()
                            .cloned()
                        {
                            entity.insert(needs_wad);
                        }
                    }
                }

                // Spawn debug volumes
                {
                    let material = StandardMaterial {
                        base_color: Color::Rgba {
                            red: 0.0,
                            green: 0.0,
                            blue: 0.0,
                            alpha: 0.0,
                        },
                        ..Default::default()
                    };
                    let material = load_context
                        .set_labeled_asset("DebugVolumeColor", LoadedAsset::new(material));

                    for debug_volumes in debug_volumes.into_iter() {
                        let mins = debug_volumes.mins;
                        let maxs = debug_volumes.maxs;
                        let x = (mins[1] + maxs[1]) / 2.0;
                        let y = (mins[2] + maxs[2]) / 2.0;
                        let z = (mins[0] + maxs[0]) / 2.0;

                        let transform = GlobalTransform::from_xyz(x, y, z);

                        parent
                            .spawn_bundle(PbrBundle {
                                mesh: debug_volumes.mesh,
                                material: material.clone(),
                                global_transform: transform,
                                visible: Visible {
                                    is_transparent: true,
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(BspMesh);
                    }
                }
            });
        });

    load_context.set_labeled_asset("Map", LoadedAsset::new(Scene::new(world)));

    load_context.set_default_asset(LoadedAsset::new(BspFile));

    Ok(())
}

fn vec3tofloat3(vec3: glam::Vec3) -> [f32; 3] {
    [vec3.y, vec3.z, vec3.x]
}

fn triangulate(verts: &[glam::Vec3]) -> Vec<u16> {
    let mut indicies = vec![];

    if verts.len() < 3 {
        return indicies;
    }

    for i in 1..verts.len() - 1 {
        indicies.push(0);
        indicies.push(i as u16);
        indicies.push(i as u16 + 1);
    }

    indicies
}

#[allow(clippy::type_complexity)]
fn add_wireframes_system(
    mut commands: Commands,
    config: Res<BspConfig>,
    mut query: QuerySet<(
        Query<(Entity, &BspMesh), Without<Wireframe>>,
        Query<(Entity, &BspMesh), With<Wireframe>>,
    )>,
) {
    if config.show_wireframe {
        query.q0_mut().iter_mut().for_each(|(entity, _)| {
            commands.entity(entity).insert(Wireframe);
        })
    } else {
        query.q1_mut().iter_mut().for_each(|(entity, _)| {
            commands.entity(entity).remove::<Wireframe>();
        })
    }
}

#[derive(Debug, Default)]
struct WadManager {
    textures: HashMap<String, Texture>,
}

fn load_wads_system(mut manager: ResMut<WadManager>) {
    let mut wads = vec![];

    if let Ok(dir) = fs::read_dir("assets/wads") {
        for entry in dir.flatten() {
            let path = entry.path();

            if let Ok(name) = entry.file_name().into_string() {
                if name.ends_with(".wad") {
                    if let Ok(file) = fs::File::open(path) {
                        let reader = BufReader::new(file);

                        match WadDecoder::from_reader(reader).decode() {
                            Ok(wad) => wads.push(wad),
                            Err(e) => {
                                dbg!(&e);
                            }
                        }
                    }
                }
            }
        }
    }

    for wad in wads.into_iter() {
        for (name, texture) in wad.textures.into_iter() {
            if texture.width > 0 && texture.height > 0 {
                manager.textures.insert(name, texture);
            }
        }
    }
}

fn parse_texture(texture: &Texture) -> (bool, BevyTexture) {
    let mut transparent = false;
    let mut data = vec![];

    let is_tranparent = |r: u8, g: u8, b: u8| -> bool { r == 0 && g == 0 && b == 255 };

    for idx in texture.mip.iter() {
        let idx = *idx as usize;

        let r = texture.palette[idx.min(255)][0];
        let g = texture.palette[idx.min(255)][1];
        let b = texture.palette[idx.min(255)][2];

        data.push(r);
        data.push(g);
        data.push(b);
        data.push(if is_tranparent(r, g, b) { 0 } else { 255 });

        if is_tranparent(r, g, b) {
            transparent = true;
        }
    }

    (
        transparent,
        BevyTexture {
            data,
            size: Extent3d {
                width: texture.width,
                height: texture.height,
                depth_or_array_layers: 1,
            },
            sampler: SamplerDescriptor {
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                ..Default::default()
            },
            ..Default::default()
        },
    )
}

fn apply_wad_textures_system(
    mut commands: Commands,
    mut query: Query<(Entity, &BspNeedsWad, &mut Visible)>,
    manager: Res<WadManager>,
    mut textures: ResMut<Assets<BevyTexture>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut missing_textures = HashSet::new();
    let mut mat_handles = HashMap::new();

    for (_, wad, _) in query.iter_mut() {
        missing_textures.insert(wad.name.clone());
    }

    for name in missing_textures.iter() {
        if let Some(texture) = manager.textures.get(name) {
            let (transparent, texture) = parse_texture(&texture);

            let tex_handle = textures.add(texture);

            let material = StandardMaterial {
                base_color_texture: Some(tex_handle),
                unlit: true,
                ..Default::default()
            };
            let handle = materials.add(material);

            mat_handles.insert(name.clone(), (transparent, handle));
        }
    }

    missing_textures.drain();

    if !mat_handles.is_empty() {
        for (entity, wad, mut visible) in query.iter_mut() {
            let mut entity_commands = commands.entity(entity);

            if let Some((is_transparent, material)) = mat_handles.get(&wad.name).cloned() {
                visible.is_transparent = is_transparent;

                entity_commands.remove::<Handle<StandardMaterial>>();
                entity_commands.insert(material);
            } else {
                missing_textures.insert(wad.name.clone());
            }

            // No point in trying after first time
            entity_commands.remove::<BspNeedsWad>();
        }
    }

    for name in missing_textures {
        warn!("WAD texture missing: {}", name);
    }
}
