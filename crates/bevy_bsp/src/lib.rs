use std::io::Cursor;
use std::ops::MulAssign;

use anyhow::format_err;
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::prelude::{shape, *};
use bevy::reflect::TypeUuid;
use bevy::render::mesh::Indices;
use bevy::render::pipeline::PrimitiveTopology;
use bevy::render::texture::{Extent3d, FilterMode, SamplerDescriptor, TextureFormat};
use bevy::render::wireframe::Wireframe;
use bevy::utils::BoxedFuture;
use decoder::format::GoldSrc30Bsp;
use decoder::BspFormat;

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
            .register_type::<Wireframe>()
            .add_asset::<BspFile>()
            .add_system(add_wireframes_system.system());
    }
}

#[derive(Debug, TypeUuid)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct BspFile {
    materials: Vec<Handle<StandardMaterial>>,
    meshes: Vec<Handle<Mesh>>,
    scene: Handle<Scene>,
    debug_volumes: Vec<Handle<Mesh>>,
}

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

    let mut materials = vec![];
    let mut meshes = vec![];
    let mut debug_volumes = vec![];

    // for idx in leaves {
    //     let leaf = bsp.leaves[idx as usize];

    //     for offset in 0..leaf.num_mark_surfaces {
    //         let mut positions = vec![];
    //         let mut normals = vec![];
    //         let mut tangents = vec![];
    //         let mut colors = vec![];
    //         let mut uvs = vec![];
    //         let mut texture = None;

    //         let face_idx =
    //             bsp.mark_surfaces[leaf.idx_first_mark_surface as usize + offset as usize].0;

    //         if let Some(face) = bsp.faces.get(face_idx as usize) {
    //             let lighting = bsp.lighting.get(face.lightmap_offset as usize / 3);

    //             // TODO: Get texture
    //             let tex_info = bsp.texture_info.get(face.texture_info as usize);
    //             texture = tex_info
    //                 .map(|info| bsp.textures.get(info.idx_miptex as usize))
    //                 .flatten();
    //             //let tex_name = texture.map(|t| String::from_utf8_lossy(&t.name[..]).to_string());

    //             // if tex_name
    //             //     .as_deref()
    //             //     .map(|name| name.starts_with("sky"))
    //             //     .unwrap_or_default()
    //             // {
    //             //     continue;
    //             // }

    //             if let Some(plane) = bsp.planes.get(face.plane as usize) {
    //                 let mut normal = plane.normal;

    //                 if face.plane_side > 0 {
    //                     normal.x *= -1.0;
    //                     normal.y *= -1.0;
    //                     normal.z *= -1.0;
    //                 }

    //                 let num_edges = face.edges as usize;
    //                 let first_edge = face.first_edge as usize;

    //                 for surfedge_idx in first_edge..first_edge + num_edges {
    //                     if let Some(surf_edge) = bsp.surf_edges.get(surfedge_idx) {
    //                         let edge_idx = surf_edge.0;
    //                         let edge_idx_abs = edge_idx.abs() as usize;

    //                         if let Some(edge) = bsp.edges.get(edge_idx_abs) {
    //                             let mut vert0_idx = edge.vertex.x as usize;
    //                             let mut vert1_idx = edge.vertex.y as usize;

    //                             if edge_idx < 0 {
    //                                 std::mem::swap(&mut vert0_idx, &mut vert1_idx);
    //                             }

    //                             let vert0 = bsp.vertices.get(vert0_idx);
    //                             let vert1 = bsp.vertices.get(vert1_idx);

    //                             if let (Some(vert0), Some(vert1)) = (vert0, vert1) {
    //                                 let mut tangent = glam::Vec3::default();
    //                                 tangent.x = vert0.0.x - vert1.0.x;
    //                                 tangent.y = vert0.0.y - vert1.0.y;
    //                                 tangent.z = vert0.0.z - vert1.0.z;

    //                                 let tangent_length = (tangent.x * tangent.x
    //                                     + tangent.y * tangent.y
    //                                     + tangent.z * tangent.z)
    //                                     .sqrt();

    //                                 tangent.x /= tangent_length;
    //                                 tangent.y /= tangent_length;
    //                                 tangent.z /= tangent_length;

    //                                 let mut color = [0; 3];
    //                                 if let Some(colors) = lighting {
    //                                     color[0] = colors.r as u32;
    //                                     color[0] = colors.g as u32;
    //                                     color[0] = colors.b as u32;
    //                                 }

    //                                 let mut u = 0.0;
    //                                 let mut v = 0.0;
    //                                 if let Some(tex_info) = tex_info {
    //                                     u = (tex_info.s_vector.x * vert0.0.x
    //                                         + tex_info.s_vector.y * vert0.0.y
    //                                         + tex_info.s_vector.z * vert0.0.z)
    //                                         + tex_info.s_shift;

    //                                     v = (tex_info.t_vector.x * vert0.0.x
    //                                         + tex_info.t_vector.y * vert0.0.y
    //                                         + tex_info.t_vector.z * vert0.0.z)
    //                                         + tex_info.t_shift;
    //                                 }

    //                                 positions.push(vert0.0);
    //                                 tangents.push(tangent);
    //                                 normals.push(normal);
    //                                 colors.push(color);
    //                                 uvs.push([u, v]);
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //         let indicies = triangulate(&positions);

    //         let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    //         mesh.set_attribute(
    //             Mesh::ATTRIBUTE_UV_0,
    //             uvs.into_iter().rev().collect::<Vec<_>>(),
    //         );
    //         mesh.set_attribute(
    //             Mesh::ATTRIBUTE_POSITION,
    //             positions
    //                 .into_iter()
    //                 .rev()
    //                 .map(vec3tofloat3)
    //                 .collect::<Vec<_>>(),
    //         );
    //         mesh.set_attribute(
    //             Mesh::ATTRIBUTE_NORMAL,
    //             normals
    //                 .into_iter()
    //                 .rev()
    //                 .map(vec3tofloat3)
    //                 .collect::<Vec<_>>(),
    //         );
    //         mesh.set_attribute(
    //             Mesh::ATTRIBUTE_TANGENT,
    //             tangents
    //                 .into_iter()
    //                 .rev()
    //                 .map(vec3tofloat3)
    //                 .collect::<Vec<_>>(),
    //         );
    //         mesh.set_attribute(
    //             Mesh::ATTRIBUTE_COLOR,
    //             colors.into_iter().rev().collect::<Vec<_>>(),
    //         );
    //         mesh.set_indices(Some(Indices::U16(indicies)));

    //         let mesh_label = format!("Mesh{}", face_idx);
    //         let handle = load_context.set_labeled_asset(&mesh_label, LoadedAsset::new(mesh));

    //         meshes.push(handle);

    //         if let Some(texture) = texture {
    //             // Skip WAD textures
    //             if texture.offsets[0] > 0 {
    //                 let mut transparent = false;
    //                 let mut data = vec![];

    //                 let is_tranparent =
    //                     |r: u8, g: u8, b: u8| -> bool { r == 0 && g == 0 && b == 255 };

    //                 for idx in texture.mip.iter() {
    //                     let idx = *idx as usize;

    //                     let r = texture.palette[idx.min(255)][0];
    //                     let g = texture.palette[idx.min(255)][1];
    //                     let b = texture.palette[idx.min(255)][2];

    //                     data.push(r);
    //                     data.push(g);
    //                     data.push(b);
    //                     data.push(if is_tranparent(r, g, b) { 0 } else { 255 });

    //                     if is_tranparent(r, g, b) {
    //                         transparent = true;
    //                     }
    //                 }

    //                 let texture = Texture {
    //                     data,
    //                     size: Extent3d {
    //                         width: texture.width,
    //                         height: texture.height,
    //                         depth: 1,
    //                     },
    //                     ..Default::default()
    //                 };
    //                 let tex_handle = load_context.set_labeled_asset(
    //                     &format!("Texture{}", face_idx),
    //                     LoadedAsset::new(texture),
    //                 );

    //                 let material = StandardMaterial {
    //                     base_color_texture: Some(tex_handle),
    //                     unlit: true,
    //                     ..Default::default()
    //                 };
    //                 let mat_handle = load_context.set_labeled_asset(
    //                     &format!("Material{}", face_idx),
    //                     LoadedAsset::new(material),
    //                 );

    //                 materials.push(mat_handle);
    //             }
    //         }
    //     }
    // }

    // Add debug volume for worldspawn
    {
        let mins = model.mins;
        let maxs = model.maxs;

        let x_length = maxs[1] - mins[1];
        let y_length = maxs[2] - mins[2];
        let z_length = maxs[0] - mins[0];
        let worldspawn_box: Mesh = shape::Box::new(x_length, y_length, z_length).into();

        let mesh_label = "WorldspawnDebugMesh";
        let handle = load_context.set_labeled_asset(mesh_label, LoadedAsset::new(worldspawn_box));

        debug_volumes.push(handle);
    }

    let num_faces = model.num_faces as usize;
    let first_face = model.idx_first_face as usize;

    for face_idx in first_face..first_face + num_faces {
        let mut positions = vec![];
        let mut normals = vec![];
        //let mut tangents = vec![];
        let mut colors = vec![];
        let mut uvs = vec![];
        let mut texture = None;

        if let Some(face) = bsp.faces.get(face_idx) {
            let lighting = bsp.lighting.get(face.lightmap_offset as usize / 3);

            // TODO: Get texture
            let tex_info = bsp.texture_info.get(face.texture_info as usize);
            texture = tex_info
                .map(|info| bsp.textures.get(info.idx_miptex as usize))
                .flatten();
            //let tex_name = texture.map(|t| String::from_utf8_lossy(&t.name[..]).to_string());

            // if tex_name
            //     .as_deref()
            //     .map(|name| name.starts_with("sky"))
            //     .unwrap_or_default()
            // {
            //     continue;
            // }

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
                            let vert1 = bsp.vertices.get(vert1_idx);

                            if let (Some(vert0), Some(vert1)) = (vert0, vert1) {
                                // let mut tangent = glam::Vec3::default();
                                // tangent.x = vert0.0.x - vert1.0.x;
                                // tangent.y = vert0.0.y - vert1.0.y;
                                // tangent.z = vert0.0.z - vert1.0.z;

                                // let tangent_length = (tangent.x * tangent.x
                                //     + tangent.y * tangent.y
                                //     + tangent.z * tangent.z)
                                //     .sqrt();

                                // tangent.x /= tangent_length;
                                // tangent.y /= tangent_length;
                                // tangent.z /= tangent_length;

                                let mut color = [0; 3];
                                if let Some(colors) = lighting {
                                    color[0] = colors.r as u32;
                                    color[0] = colors.g as u32;
                                    color[0] = colors.b as u32;
                                }

                                let mut u = 0.0;
                                let mut v = 0.0;
                                if let (Some(tex_info), Some(texture)) = (tex_info, texture) {
                                    u = (vert0.0.dot(tex_info.s_vector) + tex_info.s_shift)
                                        / texture.width as f32;

                                    v = (vert0.0.dot(tex_info.t_vector) + tex_info.t_shift)
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
            normals
                .into_iter()
                .rev()
                .map(vec3tofloat3)
                .collect::<Vec<_>>(),
        );
        // mesh.set_attribute(
        //     Mesh::ATTRIBUTE_TANGENT,
        //     tangents
        //         .into_iter()
        //         .rev()
        //         .map(vec3tofloat3)
        //         .collect::<Vec<_>>(),
        // );
        mesh.set_attribute(
            Mesh::ATTRIBUTE_COLOR,
            colors.into_iter().rev().collect::<Vec<_>>(),
        );
        mesh.set_indices(Some(Indices::U16(indicies)));

        let mesh_label = format!("Mesh{}", face_idx);
        let handle = load_context.set_labeled_asset(&mesh_label, LoadedAsset::new(mesh));

        meshes.push(handle);

        // if let Some(texture) = texture {
        //     // Skip WAD textures
        //     if texture.offsets[0] > 0 {
        //         let mut transparent = false;
        //         let mut data = vec![];

        //         let is_tranparent = |r: u8, g: u8, b: u8| -> bool { r == 0 && g == 0 && b == 255 };

        //         for idx in texture.mip.iter() {
        //             let idx = *idx as usize;

        //             let r = texture.palette[idx.min(255)][0];
        //             let g = texture.palette[idx.min(255)][1];
        //             let b = texture.palette[idx.min(255)][2];

        //             data.push(r);
        //             data.push(g);
        //             data.push(b);
        //             data.push(if is_tranparent(r, g, b) { 0 } else { 255 });

        //             if is_tranparent(r, g, b) {
        //                 transparent = true;
        //             }
        //         }

        //         let texture = Texture {
        //             data,
        //             size: Extent3d {
        //                 width: texture.width,
        //                 height: texture.height,
        //                 depth: 1,
        //             },
        //             sampler: SamplerDescriptor {
        //                 //mipmap_filter: FilterMode::Linear,
        //                 ..Default::default()
        //             },
        //             ..Default::default()
        //         };
        //         let tex_handle = load_context
        //             .set_labeled_asset(&format!("Texture{}", face_idx), LoadedAsset::new(texture));

        //         let material = StandardMaterial {
        //             base_color_texture: Some(tex_handle),
        //             unlit: true,
        //             ..Default::default()
        //         };
        //         let mat_handle = load_context.set_labeled_asset(
        //             &format!("Material{}", face_idx),
        //             LoadedAsset::new(material),
        //         );

        //         materials.push(mat_handle);
        //     }
        // }
    }

    let mut world = World::default();
    world
        .spawn()
        .insert_bundle((Transform::identity(), GlobalTransform::identity()))
        .with_children(|parent| {
            let mut map = parent.spawn_bundle((Transform::identity(), GlobalTransform::identity()));

            map.with_children(|parent| {
                let material: StandardMaterial = Color::DARK_GRAY.into();
                let default_material =
                    load_context.set_labeled_asset("DebugColor", LoadedAsset::new(material));

                for (idx, mesh) in meshes.clone().into_iter().enumerate() {
                    let material = materials.get(idx).cloned();

                    parent
                        .spawn_bundle(PbrBundle {
                            mesh,
                            material: material.unwrap_or_else(|| default_material.clone()),
                            visible: Visible {
                                is_transparent: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(BspMesh);
                }

                let material = StandardMaterial {
                    base_color: Color::Rgba {
                        red: 0.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 0.0,
                    },
                    ..Default::default()
                };
                let material =
                    load_context.set_labeled_asset("DebugVolumeColor", LoadedAsset::new(material));

                for mesh in debug_volumes.clone() {
                    //Fix: Add bounding boxes to actual asset handle incase we want to use more than worldspawn in future

                    let mins = model.mins;
                    let maxs = model.maxs;
                    let x = (mins[1] + maxs[1]) / 2.0;
                    let y = (mins[2] + maxs[2]) / 2.0;
                    let z = (mins[0] + maxs[0]) / 2.0;

                    parent
                        .spawn_bundle(PbrBundle {
                            mesh,
                            material: material.clone(),
                            transform: Transform::from_xyz(x, y, z),
                            visible: Visible {
                                is_transparent: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(BspMesh);
                }
            });
        });

    let scene = load_context.set_labeled_asset("Map", LoadedAsset::new(Scene::new(world)));

    load_context.set_default_asset(LoadedAsset::new(BspFile {
        materials,
        meshes,
        scene,
        debug_volumes,
    }));

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
