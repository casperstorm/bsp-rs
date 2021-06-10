use std::io::Cursor;

use anyhow::format_err;
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::mesh::Indices;
use bevy::render::pipeline::PrimitiveTopology;
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
pub struct BspFace;

/// Adds support for Bsp file rendering
#[derive(Default)]
pub struct BspPlugin;

impl Plugin for BspPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_asset_loader::<BspFileLoader>()
            .init_resource::<BspConfig>()
            .register_type::<BspFace>()
            .register_type::<Wireframe>()
            .add_asset::<BspFile>()
            .add_system(add_wireframes_system.system());
    }
}

#[derive(Debug, TypeUuid)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct BspFile {
    meshes: Vec<Handle<Mesh>>,
    scene: Handle<Scene>,
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

    let num_faces = model.num_faces as usize;
    let first_face = model.idx_first_face as usize;

    let mut meshes = vec![];

    for face_idx in first_face..first_face + num_faces {
        let mut positions = vec![];
        let mut normals = vec![];
        let mut tangents = vec![];

        if let Some(face) = bsp.faces.get(face_idx) {
            // TODO: Get texture
            // let tex_info = bsp.texture_info.get(face.texture_info as usize);
            // let texture = tex_info
            //     .map(|info| bsp.textures.get(info.idx_miptex as usize))
            //     .flatten();
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
                    normal.x *= -1.0;
                    normal.y *= -1.0;
                    normal.z *= -1.0;
                }

                let num_edges = face.edges as usize;
                let first_edge = face.first_edge as usize;

                for surfedge_idx in first_edge..first_edge + num_edges {
                    if let Some(surf_edge) = bsp.surf_edges.get(surfedge_idx) {
                        let edge_idx = surf_edge.0;
                        let edge_idx_abs = edge_idx.abs() as usize;

                        if let Some(edge) = bsp.edges.get(edge_idx_abs) {
                            let mut vert0_idx = edge.vertex.x as usize;
                            let mut vert1_idx = edge.vertex.y as usize;

                            if edge_idx < 0 {
                                std::mem::swap(&mut vert0_idx, &mut vert1_idx);
                            }

                            let vert0 = bsp.vertices.get(vert0_idx);
                            let vert1 = bsp.vertices.get(vert1_idx);

                            if let (Some(vert0), Some(vert1)) = (vert0, vert1) {
                                let mut tangent = glam::Vec3::default();
                                tangent.x = vert0.0.x - vert1.0.x;
                                tangent.y = vert0.0.y - vert1.0.y;
                                tangent.z = vert0.0.z - vert1.0.z;

                                let tangent_length = (tangent.x * tangent.x
                                    + tangent.y * tangent.y
                                    + tangent.z * tangent.z)
                                    .sqrt();

                                tangent.x /= tangent_length;
                                tangent.y /= tangent_length;
                                tangent.z /= tangent_length;

                                positions.push(vert0.0);
                                tangents.push(tangent);
                                normals.push(normal);
                            }
                        }
                    }
                }
            }
        }

        let indicies = triangulate(&positions);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; positions.len()]);
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
        mesh.set_attribute(
            Mesh::ATTRIBUTE_TANGENT,
            tangents
                .into_iter()
                .rev()
                .map(vec3tofloat3)
                .collect::<Vec<_>>(),
        );
        mesh.set_indices(Some(Indices::U16(indicies)));

        let mesh_label = format!("Mesh{}", face_idx);
        let handle = load_context.set_labeled_asset(&mesh_label, LoadedAsset::new(mesh));

        meshes.push(handle);
    }

    let mut world = World::default();
    world
        .spawn()
        .insert_bundle((Transform::identity(), GlobalTransform::identity()))
        .with_children(|parent| {
            let mut map = parent.spawn_bundle((Transform::identity(), GlobalTransform::identity()));

            map.with_children(|parent| {
                let material: StandardMaterial = Color::CYAN.into();
                let material =
                    load_context.set_labeled_asset("DebugColor", LoadedAsset::new(material));

                for mesh in meshes.clone() {
                    parent
                        .spawn_bundle(PbrBundle {
                            mesh,
                            material: material.clone(),
                            ..Default::default()
                        })
                        .insert(BspFace);
                }
            });
        });

    let scene = load_context.set_labeled_asset("Map", LoadedAsset::new(Scene::new(world)));

    load_context.set_default_asset(LoadedAsset::new(BspFile { meshes, scene }));

    Ok(())
}

fn vec3tofloat3(vec3: glam::Vec3) -> [f32; 3] {
    [vec3.x, vec3.y, vec3.z]
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
        Query<(Entity, &BspFace), Without<Wireframe>>,
        Query<(Entity, &BspFace), With<Wireframe>>,
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
