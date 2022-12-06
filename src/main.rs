mod b3dm;
mod glb;
mod mesh;
mod shptiff;
mod tileset;

use shapefile::{read_as,PolygonZ,dbase};
use geo::Centroid;
use geotiff_rs::GeoTiff;
use std::borrow::Cow;
use std::env;
use std::fs;
use std::path::Path;
use std::process::exit;
use std::time::Instant;

fn main() {
    get_3dtiles_file()
}

fn get_3dtiles_file() {
    let now = Instant::now();
    let args: Vec<String> = env::args().into_iter().collect();
    let filename = match args.get(1) {
        Some(arg) => arg,
        None => {
            println!("请输入第一个指令值以获取文件路径");
            exit(-1);
        }
    };
    let height = match args.get(2) {
        Some(arg) => arg,
        None => {
            println!("请输入第二个指令值以获取高度字段");
            exit(-1);
        }
    };

    let mut shp_tiff = None;

    match args.get(3) {
        Some(arg) => {
            let path = Path::new(arg);
            let parent = path.parent().unwrap().to_str().unwrap().to_string();
            let file_stem = path
                .file_stem()
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap();
            let file_name = file_stem + ".tfw";
            let contents = fs::read_to_string(parent + "\\" + &file_name).unwrap();
            let mut trans_form_line = contents.lines();
            let px = trans_form_line.next().unwrap();
            trans_form_line.next();
            trans_form_line.next();
            let py = trans_form_line.next().unwrap();
            let xr = trans_form_line.next().unwrap();
            let yr = trans_form_line.next().unwrap();
            let tiff_data = GeoTiff::from_file(path);
            match tiff_data {
                Ok(x) => {
                    shp_tiff = Some(shptiff::ShpTiff {
                        tiff: x,
                        xr: xr.parse::<f32>().unwrap(),
                        yr: yr.parse::<f32>().unwrap(),
                        px: px.parse::<f32>().unwrap(),
                        py: py.parse::<f32>().unwrap(),
                    })
                }
                Err(e) => println!("文件读取错误: {:?}", e),
            };
        }
        None => (),
    };

    let polygons = read_as::<_, PolygonZ, dbase::Record>(filename)
        .expect("无法正确的打开shp文件,请确保shp类型全为多面再试");

    let mut max_x = 0.;
    let mut min_x = 0.;
    let mut max_y = 0.;
    let mut min_y = 0.;
    // let mut min_h = 0.;
    let mut max_h = 0.;
    let mut cx = 0.;
    let mut cy = 0.;
    let mut bottom_h = 0;
    let mut first = true;
    let mut temp_h = 0;

    for (polygon, polygon_record) in polygons.clone() {
        let geo_polygon: geo::MultiPolygon<f64> = polygon.into();
        geo_polygon.iter().for_each(|poly| {
            let center_id = poly.centroid().unwrap();
            match &shp_tiff {
                Some(tiff_entity) => {
                    temp_h = tiff_entity
                        .get_height_by_geo_info(center_id.x() as f32, center_id.y() as f32);
                    if first {
                        bottom_h = temp_h;
                        first = false
                    } else {
                        bottom_h = i32::min(bottom_h, temp_h)
                    }
                }
                None => {}
            }
            let line = poly.exterior();
            line.points().for_each(|p| {
                max_x = f32::max(max_x, p.x() as f32);
                if min_x == 0. {
                    min_x = p.x() as f32;
                } else {
                    min_x = f32::min(min_x, p.x() as f32);
                };

                max_y = f32::max(max_y, p.y() as f32);
                if min_y == 0. {
                    min_y = p.y() as f32;
                } else {
                    min_y = f32::min(min_y, p.y() as f32);
                }
            })
        });
        let polygon_height = match polygon_record.get(height) {
            Some(dbase::FieldValue::Float(Some(x))) => x,
            Some(_) => panic!("高度字段{}必须是浮点类型", height),
            None => panic!("高度字段{}不存在，请重试", height),
        };
        max_h = f32::max(max_h, *polygon_height + temp_h as f32);
        cx = (max_x + min_x) / 2.;
        cy = (max_y + min_y) / 2.;
        let bw = max_x - min_x;
        let bh = max_y - min_y;
        let test = tileset::Tiles {
            asset: tileset::Asset {
                gltf_up_axis: "Z".to_string(),
                version: "1.0".to_string(),
            },
            geometric_error: 200,
            root: tileset::Root {
                bounding_volume: tileset::get_root_bounding_volume(
                    min_x,
                    max_x,
                    min_y,
                    max_y,
                    bottom_h as f32,
                    max_h,
                ),
                geometric_error: 200,
                refine: "REPLACE".to_string(),
                children: vec![tileset::Child {
                    content: tileset::Content {
                        uri: "0.b3dm".to_string(),
                    },
                    geometric_error: 100.,
                    refine: "REPLACE".to_string(),
                    // bounding_volume: tileset::get_root_bounding_volume(
                    //     min_x, max_x, min_y, max_y, max_h,
                    // ),
                    bounding_volume: tileset::get_child_bounding_volume(
                        cx,
                        cy,
                        bw,
                        bh,
                        bottom_h as f32,
                        max_h,
                    ),
                    transform: tileset::get_transform(cx, cy, bottom_h as f32),
                }],
            },
        };
        let _ = fs::create_dir("b3dm");
        let data = serde_json::to_string(&test).expect("Serialization error");
        // println!("{:?}", data);
        std::fs::write("b3dm/tileset.json", &mut data.as_bytes()).unwrap();
    }

    let mut id = 0;
    let mut meshes = vec![];
    let mut ids = vec![];
    let mut hight_vec = vec![];
    let mut names = vec![];
    for (polygon, polygon_record) in polygons {
        let geo_polygon: geo::MultiPolygon<f64> = polygon.into();
        let the_bottom = 0.;
        let polygon_h = match polygon_record.get(height) {
            Some(dbase::FieldValue::Float(Some(x))) => x,
            Some(_) => panic!("高度字段{}必须是浮点类型", height),
            None => panic!("高度字段{}不存在，请重试", height),
        };
        // let polygon_h_with_bottom = polygon_h + the_bottom;
        let mesh = mesh::Mesh::init(
            cx as f64,
            cy as f64,
            polygon_h,
            the_bottom,
            geo_polygon,
            id,
        );
        meshes.push(mesh);
        ids.push(id as u32);
        // hight_vec.push(polygon_h_with_bottom);
        hight_vec.push(*polygon_h);
        names.push("mesh_".to_string() + &id.to_string());
        id = id + 1;
    }

    let test_batch_table = b3dm::BatchTable {
        batch_id: ids,
        height: hight_vec,
        name: names,
    };
    let test_feature_table = b3dm::FeatureTable {
        batch_length: id as u32,
    };

    let test1 = serde_json::to_string(&test_batch_table).expect("Serialization error");
    let test2 = serde_json::to_string(&test_feature_table).expect("Serialization error");
    let mut feature_table_json_byte_length = test2.len();
    align_to_multiple_of_four(&mut feature_table_json_byte_length);
    let mut batch_table_json_byte_length = test1.len();
    align_to_multiple_of_four(&mut batch_table_json_byte_length);

    let vec2 = test1.into_bytes();
    let vec1 = test2.into_bytes();

    let glb = glb::get_glb(meshes);
    let b33dm = b3dm::MakeB3dm {
        feature_table_json_byte_length,
        batch_table_json_byte_length,
        glb: Some(Cow::Owned(glb)),
    };

    let writer = std::fs::File::create("b3dm/0.b3dm").expect("I/O error");
    b33dm
        .to_writer(writer, vec1, vec2)
        .expect("glTF binary output error");
        println!("执行时间: {}", now.elapsed().as_millis());
}

fn align_to_multiple_of_four(n: &mut usize) {
    *n = (*n + 3) & !3;
}
