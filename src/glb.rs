use gltf_json as json;
use std::mem;

use json::validation::Checked::Valid;
use std::borrow::Cow;
use crate::mesh::Mesh;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

/// Calculate bounding coordinates of a list of vertices, used for the clipping distance of the model
fn bounding_coords(points: &Vec<[f32; 3]>) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX, f32::MAX, f32::MAX];
    let mut max = [f32::MIN, f32::MIN, f32::MIN];

    for point in points {
        for i in 0..3 {
            min[i] = f32::min(min[i], point[i]);
            max[i] = f32::max(max[i], point[i]);
        }
    }
    (min, max)
}

fn calc_max(index: &Vec<[i32; 3]>) -> i32 {
    let mut min = 0;
    for idx in index {
        for i in 0..3 {
            if min < idx[i] {
                min = idx[i]
            }
        }
    }
    min
}

fn align_to_multiple_of_four(n: &mut u32) {
    *n = (*n + 3) & !3;
}

pub fn get_glb(meshes: Vec<Mesh>) -> Vec<u8> {
    let mut offset = 0;
    let mut accessors_vec = vec![];
    let mut buffer_views_vec = vec![];
    let mut meshes_vec = vec![];
    let mut nodes_vec = vec![];
    let mut scenes_vec = vec![];
    let mut res_vec = vec![];
    meshes.iter().enumerate().for_each(|(idx, mesh)| {
        let position = &mesh.vertex;
        let normal = &mesh.normal;
        let index = &mesh.index;
        let mesh_name = mesh.mesh_name.clone();
        let position = position
            .iter()
            .map(|p| {
                let x = p[0] as f32;
                let y = p[1] as f32;
                let z = p[2] as f32;
                [x, y, z]
            })
            .collect();

        let (min, max) = bounding_coords(&position);
        let (n_min, n_max) = bounding_coords(normal);
        let max_i = calc_max(index);

        let mut indeice_buffer_length = (index.len() * 3 * mem::size_of::<u16>()) as u32;
        while indeice_buffer_length % 4 != 0 {
            indeice_buffer_length += 1
        }
        let indices_buffer_view = json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: indeice_buffer_length,
            byte_offset: Some(offset),
            byte_stride: Some(mem::size_of::<u16>() as u32),
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: Some(Valid(json::buffer::Target::ElementArrayBuffer)),
        };

        offset = offset + indeice_buffer_length;

        let mut position_buffer_length = (position.len() * mem::size_of::<[f32; 3]>()) as u32;
        while position_buffer_length % 4 != 0 {
            position_buffer_length += 1
        }
        let position_buffer_view = json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: position_buffer_length,
            byte_offset: Some(offset),
            byte_stride: Some(mem::size_of::<[f32; 3]>() as u32),
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: Some(Valid(json::buffer::Target::ArrayBuffer)),
        };
        offset = offset + position_buffer_length;

        let mut normal_buffer_length = (normal.len() * mem::size_of::<[f32; 3]>()) as u32;
        while normal_buffer_length % 4 != 0 {
            normal_buffer_length += 1
        }
        let normal_buffer_view = json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: normal_buffer_length,
            byte_offset: Some(offset),
            byte_stride: Some(mem::size_of::<[f32; 3]>() as u32),
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: Some(Valid(json::buffer::Target::ArrayBuffer)),
        };
        offset = offset + normal_buffer_length;

        let mut mesh_buffer_length = (position.len() * mem::size_of::<u16>()) as u32;
        while mesh_buffer_length % 4 != 0 {
            mesh_buffer_length += 1
        }
        let mesh1_buffer_view = json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: mesh_buffer_length,
            byte_offset: Some(offset),
            byte_stride: Some(mem::size_of::<u16>() as u32),
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: Some(Valid(json::buffer::Target::ElementArrayBuffer)),
        };
        offset = offset + mesh_buffer_length;

        let idc_acc = json::Accessor {
            buffer_view: Some(json::Index::new((4 * idx) as u32)),
            byte_offset: 0,
            count: (index.len() * 3) as u32,
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::U16,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Scalar),
            min: Some(json::Value::from(vec![0.])),
            max: Some(json::Value::from(vec![max_i as f32])),
            name: None,
            normalized: false,
            sparse: None,
        };
        let pos_acc = json::Accessor {
            buffer_view: Some(json::Index::new((4 * idx + 1) as u32)),
            byte_offset: 0,
            count: position.len() as u32,
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec3),
            min: Some(json::Value::from(Vec::from(min))),
            max: Some(json::Value::from(Vec::from(max))),
            name: None,
            normalized: false,
            sparse: None,
        };
        let nor_acc = json::Accessor {
            buffer_view: Some(json::Index::new((4 * idx + 2) as u32)),
            byte_offset: 0,
            count: normal.len() as u32,
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec3),
            min: Some(json::Value::from(Vec::from(n_min))),
            max: Some(json::Value::from(Vec::from(n_max))),
            name: None,
            normalized: false,
            sparse: None,
        };

        let mesh_acc = json::Accessor {
            buffer_view: Some(json::Index::new((4 * idx + 3) as u32)),
            byte_offset: 0,
            count: position.len() as u32,
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::U16,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Scalar),
            min: Some(json::Value::from(vec![idx as f32])),
            max: Some(json::Value::from(vec![idx as f32])),
            name: None,
            normalized: false,
            sparse: None,
        };

        let primitive = json::mesh::Primitive {
            attributes: {
                let mut map = std::collections::HashMap::new();
                map.insert(
                    Valid(json::mesh::Semantic::Positions),
                    json::Index::new((4 * idx + 1) as u32),
                );
                map.insert(
                    Valid(json::mesh::Semantic::Normals),
                    json::Index::new((4 * idx + 2) as u32),
                );
                map.insert(
                    Valid(json::mesh::Semantic::Extras("BATCHID".to_string())),
                    json::Index::new((4 * idx + 3) as u32),
                );
                map
            },
            extensions: Default::default(),
            extras: Default::default(),
            indices: Some(json::Index::new((4 * idx) as u32)),
            material: Some(json::Index::new(0)),
            mode: Valid(json::mesh::Mode::Triangles),
            targets: None,
        };

        let mesh = json::Mesh {
            extensions: Default::default(),
            extras: Default::default(),
            name: Some(mesh_name),
            primitives: vec![primitive],
            weights: None,
        };

        let node = json::Node {
            camera: None,
            children: None,
            extensions: Default::default(),
            extras: Default::default(),
            matrix: None,
            mesh: Some(json::Index::new(idx as u32)),
            name: None,
            rotation: None,
            scale: None,
            translation: None,
            skin: None,
            weights: None,
        };

        nodes_vec.push(node);
        scenes_vec.push(json::Index::new(idx as u32));

        accessors_vec.push(idc_acc);
        accessors_vec.push(pos_acc);
        accessors_vec.push(nor_acc);
        accessors_vec.push(mesh_acc);
        buffer_views_vec.push(indices_buffer_view);
        buffer_views_vec.push(position_buffer_view);
        buffer_views_vec.push(normal_buffer_view);
        buffer_views_vec.push(mesh1_buffer_view);

        meshes_vec.push(mesh);

        // 计算存储的buffer
        index.iter().for_each(|ps| {
            ps.iter().for_each(|p| {
                let p = *p as u16;
                let temp = p.to_le_bytes();
                temp.iter().for_each(|u| res_vec.push(*u))
            })
        });
        while res_vec.len() % 4 != 0 {
            res_vec.push(0); // pad to multiple of four bytes
        }
        position.iter().for_each(|ps| {
            ps.iter().for_each(|p| {
                let temp = p.to_le_bytes();
                temp.iter().for_each(|u| res_vec.push(*u))
            })
        });
        while res_vec.len() % 4 != 0 {
            res_vec.push(0); // pad to multiple of four bytes
        }
        normal.iter().for_each(|ps| {
            ps.iter().for_each(|p| {
                let temp = p.to_le_bytes();
                temp.iter().for_each(|u| res_vec.push(*u))
            })
        });
        while res_vec.len() % 4 != 0 {
            res_vec.push(0); // pad to multiple of four bytes
        }
        position.iter().for_each(|_| {
            let p = idx as u16;
            let temp = p.to_le_bytes();
            temp.iter().for_each(|u| res_vec.push(*u))
        });
        while res_vec.len() % 4 != 0 {
            res_vec.push(0); // pad to multiple of four bytes
        }
    });

    let buffer = json::Buffer {
        byte_length: offset,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        uri: None,
    };

    let pbr = json::material::PbrMetallicRoughness {
        metallic_factor: json::material::StrengthFactor(0.3),
        roughness_factor: json::material::StrengthFactor(0.7),
        base_color_factor: Default::default(),
        base_color_texture: None,
        metallic_roughness_texture: None,
        extensions: None,
        extras: Default::default(),
    };

    let material = json::material::Material {
        name: Some("default".to_string()),
        pbr_metallic_roughness: pbr,
        ..Default::default()
    };

    let root = json::Root {
        accessors: accessors_vec,
        buffers: vec![buffer],
        buffer_views: buffer_views_vec,
        meshes: meshes_vec,
        nodes: nodes_vec,
        scene: Some(json::Index::new(0)),
        scenes: vec![json::Scene {
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            nodes: scenes_vec,
        }],
        materials: vec![material],
        ..Default::default()
    };

    let json_string = json::serialize::to_string(&root).expect("Serialization error");
    let mut json_offset = json_string.len() as u32;
    align_to_multiple_of_four(&mut json_offset);
    let glb = gltf::binary::Glb {
        header: gltf::binary::Header {
            magic: *b"glTF",
            version: 2,
            length: json_offset + offset,
        },
        bin: Some(Cow::Owned(res_vec)),
        json: Cow::Owned(json_string.into_bytes()),
    };
    glb.to_vec().unwrap()
}
