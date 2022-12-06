use geotiff_rs::GeoTiff;
pub struct ShpTiff {
   pub tiff: GeoTiff,
   pub xr: f32,
   pub yr: f32,
   pub px: f32,
   pub py: f32
}

impl ShpTiff {
   pub fn get_height_by_geo_info(&self,lon:f32,lat:f32) -> i32 {
        let x = (lon - self.xr) / self.px;
        let y = (self.yr - lat) / self.py;
        self.tiff.get_pixel(x.abs() as usize, y.abs() as usize)
    }
}