use gfx_maths::{Vec2,Vec3};
use  earcutr::{flatten,earcut};

pub struct Mesh {
    pub vertex: Vec<[f64; 3]>,
    pub mesh_name: String,
    pub index: Vec<[i32; 3]>,
    pub normal: Vec<[f32; 3]>,
    pub height: f64,
}

fn lon_to_meters(diff: f64, lat: f64) -> f64 {
    return diff.to_radians() / 0.000000156785 * lat.to_radians().cos();
}

fn lat_to_meters(diff: f64) -> f64 {
    return diff.to_radians() / 0.000000157891;
}

impl Mesh {
    pub fn init(
        center_x: f64,
        center_y: f64,
        height: &f32,
        bottom: f64,
        mult_polygon: geo::MultiPolygon<f64>,
        id: i32,
    ) -> Mesh {
        let mut vertex: Vec<[f64; 3]> = Vec::new();
        let mut index: Vec<[i32; 3]> = Vec::new();
        let id = id.to_string();
        let mesh_name = "mesh_".to_string() + &id;
        let mut normal = vec![];
        mult_polygon.into_iter().for_each(|polygon| {
            let line = polygon.exterior();
            let len: Vec<_> = line.points().collect();
            // 读取的线的点数，会默认把第一个点重复一次，所以会多出来一次
            let len = len.len();
            let mut idx1 = 0;
            while idx1 < len {
                let point = &line[idx1];
                let (x, y) = point.x_y();
                let point_x = x - center_x;
                let point_y = y - center_y;
                // println!("{} {}",point_x,point_y);
                let px = lon_to_meters(point_x, center_y);
                let py = lat_to_meters(point_y);
                vertex.push([px, py, bottom]);
                vertex.push([px, py, *height as f64]);
                if idx1 != 0 && idx1 != len - 1 {
                    vertex.push([px, py, bottom]);
                    vertex.push([px, py, *height as f64]);
                }
                idx1 = idx1 + 1
            }
            let vertex_num: i32 = (vertex.len() / 2) as i32;
            // println!("{}",vertex_num);
            let mut n: i32 = 0;
            while n < vertex_num {
                if n != vertex_num - 1 {
                    index.push([2 * n, 2 * n + 1, 2 * (n + 1) + 1]);
                    index.push([2 * (n + 1), 2 * n, 2 * (n + 1) + 1]);
                }
                n = n + 2;
            }
            normal = Self::calc_normal(0, vertex_num, &vertex);
            let pt_count = 2 * vertex_num;
            // 内部含有环的情况
            // 暂不完成
            //上下底面
            let mut ear_cut_polygon: Vec<Vec<Vec<f64>>> = vec![vec![]];
            let mut idx2 = 0;
            while idx2 < len {
                let point = &line[idx2];
                let (x, y) = point.x_y();
                let point_x = x - center_x;
                let point_y = y - center_y;
                let px = lon_to_meters(point_x, center_y);
                let py = lat_to_meters(point_y);
                ear_cut_polygon[0].push(vec![px, py]);
                // println!("{} {}", px, py);
                vertex.push([px, py, bottom]);
                vertex.push([px, py, *height as f64]);
                normal.push([0.0, 0.0, -1.0]);
                normal.push([0.0, 0., 1.]);
                idx2 = idx2 + 1
            }
            ear_cut_polygon.push(vec![]);
            let (vertices, _, dimensions) = flatten(&ear_cut_polygon);
            let triangles = earcut(&vertices, &[], dimensions);
            // println!("{:?}",triangles);
            let mut idx = 0;
            let tri_len = triangles.len();
            while idx < tri_len {
                let p1 = pt_count + 2 * triangles[idx] as i32;
                let p2 = pt_count + 2 * triangles[idx + 2] as i32;
                let p3 = pt_count + 2 * triangles[idx + 1] as i32;
                index.push([p1, p2, p3]);
                idx = idx + 3;
            }
            let mut idx4 = 0;
            while idx4 < tri_len {
                let p1 = pt_count + 2 * triangles[idx4] as i32 + 1;
                let p2 = pt_count + 2 * triangles[idx4 + 1] as i32 + 1;
                let p3 = pt_count + 2 * triangles[idx4 + 2] as i32 + 1;
                index.push([p1, p2, p3]);
                idx4 = idx4 + 3;
            }
        });

        Mesh {
            vertex,
            mesh_name,
            index,
            normal,
            height: *height as f64,
        }
    }

    fn calc_normal(base_cut: i32, pt_num: i32, vertex: &Vec<[f64; 3]>) -> Vec<[f32; 3]> {
        let mut normal: Vec<[f32; 3]> = Vec::new();
        let mut i = 0;
        // println!("{}",pt_num);
        while i < pt_num {
            let nor1 = Vec2::new(
                (vertex[(base_cut + 2 * (i + 1)) as usize][0]) as f32,
                (vertex[(base_cut + 2 * (i + 1)) as usize][1]) as f32,
            );
            let nor1 = nor1
                - Vec2::new(
                    (vertex[(base_cut + 2 * i) as usize][0]) as f32,
                    (vertex[(base_cut + 2 * i) as usize][1]) as f32,
                );
            let nor1_x = nor1.x;
            let nor1_y = nor1.y;
            // println!("x:{},y:{}", nor1_x, nor1_y);
            let mut nor3 = Vec3::new(-nor1_y, nor1_x, 0.);
            let nor2 = nor3.normalize();

            let nor4 = [nor2.x, nor2.y, nor2.z];
            
            normal.push(nor4);
            normal.push(nor4);
            normal.push(nor4);
            normal.push(nor4);
            i = i + 2;
        }
        normal
    }
}
