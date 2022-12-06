use gfx_maths::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct BoundingVolume {
    pub region: [f32; 6],
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Child {
    #[serde(rename = "boundingVolume")]
    pub bounding_volume: BoundingVolume,
    pub content: Content,
    #[serde(rename = "geometricError")]
    pub geometric_error: f32,
    pub refine: String,
    pub transform: [f32; 16],
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Asset {
    #[serde(rename = "gltfUpAxis")]
    pub gltf_up_axis: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    #[serde(rename = "boundingVolume")]
    pub bounding_volume: BoundingVolume,
    pub children: Vec<Child>,
    #[serde(rename = "geometricError")]
    pub geometric_error: i32,
    pub refine: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tiles {
    pub asset: Asset,
    #[serde(rename = "geometricError")]
    pub geometric_error: i32,
    pub root: Root,
}

pub fn get_root_bounding_volume(
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    min_h: f32,
    max_h: f32,
) -> BoundingVolume {
    let min_dx = min_x.to_radians();
    let max_dx = max_x.to_radians();
    let min_dy = min_y.to_radians();
    let max_dy = max_y.to_radians();
    let region = [min_dx, min_dy, max_dx, max_dy, min_h, max_h];
    BoundingVolume { region }
}

pub fn get_child_bounding_volume(
    cx: f32,
    cy: f32,
    xd: f32,
    yd: f32,
    min_h: f32,
    max_h: f32,
) -> BoundingVolume {
    let cxr = cx.to_radians();
    let cyr = cy.to_radians();
    let xdr = xd.to_radians() * 1.05;
    let ydr = yd.to_radians() * 1.05;
    let region = [
        cxr - xdr / 2.,
        cyr + ydr / 2.,
        cxr + xdr / 2.,
        cyr - ydr / 2.,
        min_h,
        max_h,
    ];
    BoundingVolume { region }
}

pub fn get_transform(lon: f32, lat: f32, min_h: f32) -> [f32; 16] {
    let lonr = lon.to_radians();
    let latr = lat.to_radians();
    let ellipsod_a = 40680631590769.;
    let ellipsod_b = 40680631590769.;
    let ellipsod_c = 40408299984661.4;
    let xn = lonr.cos() * latr.cos();
    let yn = lonr.sin() * latr.cos();
    let zn = latr.sin();
    let x0 = ellipsod_a * xn;
    let y0 = ellipsod_b * yn;
    let z0 = ellipsod_c * zn;
    let gamma = (xn * x0 + yn * y0 + zn * z0).sqrt();
    let px = x0 / gamma;
    let py = y0 / gamma;
    let pz = z0 / gamma;
    let dx = xn * min_h;
    let dy = yn * min_h;
    let dz = zn * min_h;
    let east_vec = Vec3::new(-y0, x0, 0.);
    let north_vec = Vec3::new(
        y0 * east_vec.z - east_vec.y * z0,
        z0 * east_vec.x - east_vec.z * x0,
        x0 * east_vec.y - east_vec.x * y0,
    );
    let e1 = (east_vec.x as f64) * (east_vec.x as f64);
    let e2 = (east_vec.y as f64) * (east_vec.y as f64);
    let e3 = (east_vec.z as f64) * (east_vec.z as f64);
    let east_len = (e1+ e2 + e3).sqrt();
    let l1 = (north_vec.x as f64) * (north_vec.x as f64);
    let l2 = (north_vec.y as f64) * (north_vec.y as f64);
    let l3 = (north_vec.z as f64) * (north_vec.z as f64);
    let nor_len = (l1+ l2 + l3).sqrt();
    let east_unit = east_vec / (east_len as f32);
    let north_unit = north_vec / (nor_len as f32);
    [
        east_unit.x,
        east_unit.y,
        east_unit.z,
        0.,
        north_unit.x,
        north_unit.y,
        north_unit.z,
        0.,
        xn,
        yn,
        zn,
        0.,
        px + dx,
        py + dy,
        pz + dz,
        1.,
    ]
}
