use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, RwLock};

use anyhow::format_err;
use decoder::format::gold_src_30::Texture;
use decoder::{BspDecoder, WadDecoder};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rg3d::core::algebra::{Vector2, Vector3};
use rg3d::core::color::Color;
use rg3d::core::math::aabb::AxisAlignedBoundingBox;
use rg3d::core::math::TriangleDefinition;
use rg3d::resource::{texture, ResourceState};
use rg3d::scene::base::BaseBuilder;
use rg3d::scene::camera::CameraBuilder;
use rg3d::scene::light::{BaseLightBuilder, DirectionalLightBuilder};
use rg3d::scene::mesh::buffer::{GeometryBuffer, VertexBuffer};
use rg3d::scene::mesh::surface::{Surface, SurfaceData};
use rg3d::scene::mesh::vertex::StaticVertex;
use rg3d::scene::mesh::{MeshBuilder, RenderPath};
use rg3d::scene::transform::TransformBuilder;
use rg3d::scene::Scene;

pub fn load_maps<P: AsRef<Path>>(directory: P, wad_manager: &WadManager) -> Vec<Scene> {
    let mut maps = vec![];

    if let Ok(dir) = fs::read_dir(directory) {
        for entry in dir.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.ends_with(".bsp") {
                    maps.push(entry.path());
                }
            }
        }
    }

    maps.sort();

    maps.par_iter()
        .map(|path| load_map(path, wad_manager))
        .flatten()
        .collect()
}

fn load_map<P: AsRef<Path>>(path: P, wad_manager: &WadManager) -> Result<Scene, anyhow::Error> {
    let mut scene = Scene::new();
    scene.enabled = false;

    DirectionalLightBuilder::new(
        BaseLightBuilder::new(
            BaseBuilder::new().with_local_transform(
                TransformBuilder::new()
                    .with_local_position(Vector3::new(0.0, 500.0, 0.0))
                    .build(),
            ),
        )
        .with_color(Color::opaque(100, 100, 100)),
    )
    .build(&mut scene.graph);

    BaseBuilder::new()
        .with_children(&[CameraBuilder::new(
            BaseBuilder::new().with_local_transform(
                TransformBuilder::new()
                    .with_local_position(Vector3::zeros())
                    .build(),
            ),
        )
        .with_fov(90.0)
        .with_z_near(0.1)
        .with_z_far(6000.0)
        .build(&mut scene.graph)])
        .build(&mut scene.graph);

    let file = fs::File::open(path)?;
    let mut decoder = BspDecoder::from_reader(BufReader::new(file))?;
    let bsp = decoder.decode_gold_src_30()?;

    let model = bsp
        .models
        .get(0)
        .ok_or_else(|| format_err!("No worldspawn model"))?;

    let mut textures = HashMap::new();
    let mut surfaces = vec![];

    // Add textures
    for texture in bsp.textures.iter() {
        let name = String::from_utf8_lossy(&texture.name).to_string();
        let name = name.split('\0').next().unwrap_or_default().to_string();

        // WAD textures are 0
        if let Some(bsp_texture) = if texture.offsets[0] > 0 {
            Some(texture)
        } else {
            wad_manager.textures.get(&name)
        } {
            if let Some(texture) = parse_texture(&bsp_texture) {
                textures.insert(name, texture);
            }
        }
    }

    // Add debug volumes
    for model in bsp.models.iter() {
        let bb_mins = Vector3::new(model.mins[1], model.mins[2], model.mins[0]);
        let bb_maxs = Vector3::new(model.maxs[1], model.maxs[2], model.maxs[0]);

        let bb = AxisAlignedBoundingBox::from_min_max(bb_mins, bb_maxs);

        scene.drawing_context.draw_aabb(&bb, Color::WHITE);
    }

    // Add surfaces
    {
        let num_faces = model.num_faces as usize;
        let first_face = model.idx_first_face as usize;

        for face_idx in first_face..first_face + num_faces {
            let mut verticies = vec![];
            let mut miptex_name = None;

            if let Some(face) = bsp.faces.get(face_idx) {
                let lighting = bsp.lighting.get(face.lightmap_offset as usize / 3);

                let tex_info = bsp.texture_info.get(face.texture_info as usize);
                let texture = tex_info
                    .map(|info| bsp.textures.get(info.idx_miptex as usize))
                    .flatten();
                if let Some(texture) = texture {
                    let name = String::from_utf8_lossy(&texture.name).to_string();
                    let name = name.split('\0').next().unwrap_or_default().to_string();

                    // Skip skybox
                    if &name == "sky" {
                        continue;
                    }

                    miptex_name = Some(name);
                }

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

                                    verticies.push(StaticVertex::from_pos_uv_normal(
                                        Vector3::new(vert0.0[1], vert0.0[2], vert0.0[0]),
                                        Vector2::new(u, v),
                                        Vector3::new(normal[1], normal[2], normal[0]),
                                    ));
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

            let indicies = triangulate(&verticies);

            verticies.reverse();

            if let Ok(buffer) =
                VertexBuffer::new(verticies.len(), StaticVertex::layout(), verticies)
            {
                let triangles = GeometryBuffer::new(indicies);

                let mut surface = SurfaceData::new(buffer, triangles, true);
                surface.calculate_tangents().ok();

                let mut surface = Surface::new(Arc::new(RwLock::new(surface)));

                let mut texture_used = false;

                if let Some(name) = miptex_name {
                    if let Some(texture) = textures.get(&name) {
                        surface.set_diffuse_texture(Some(texture.clone()));
                        texture_used = true;
                    }
                }

                if !texture_used {
                    surface.set_color(Color::from_rgba(63, 63, 63, 255));
                }

                surfaces.push(surface);
            }
        }
    }

    MeshBuilder::new(BaseBuilder::new())
        .with_cast_shadows(true)
        .with_surfaces(surfaces)
        .with_render_path(RenderPath::Forward)
        .build(&mut scene.graph);

    Ok(scene)
}

#[derive(Debug, Default)]
pub struct WadManager {
    textures: HashMap<String, Texture>,
}

impl WadManager {
    pub fn new<P: AsRef<Path>>(directory: P) -> Self {
        let mut manager = WadManager::default();

        let mut wads = vec![];

        if let Ok(dir) = fs::read_dir(directory) {
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

        manager
    }
}

fn triangulate(verts: &[StaticVertex]) -> Vec<TriangleDefinition> {
    let mut indicies = vec![];

    if verts.len() < 3 {
        return indicies;
    }

    for i in 1..verts.len() - 1 {
        indicies.push(TriangleDefinition([0, i as u32, i as u32 + 1]));
    }

    indicies
}

fn parse_texture(texture: &Texture) -> Option<texture::Texture> {
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

    if let Some(mut texture) = texture::TextureData::from_bytes(
        texture::TextureKind::Rectangle {
            width: texture.width,
            height: texture.height,
        },
        texture::TexturePixelKind::RGBA8,
        data,
        false,
    ) {
        texture.set_s_wrap_mode(texture::TextureWrapMode::Repeat);
        texture.set_t_wrap_mode(texture::TextureWrapMode::Repeat);

        return Some(texture::Texture::new(ResourceState::Ok(texture)));
    }

    None
}
