#[derive(Clone, Debug)]
pub struct Extent {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}

#[derive(Clone)]
pub struct TileGrid {
    extent: Extent,
}

impl TileGrid {
    pub fn new(extent: Extent) -> Self {
        Self { extent }
    }

    pub fn tile_extent(&self, x: u32, y: u32, z: u8) -> Extent {
        let tile_w = (self.extent.xmax - self.extent.xmin) / (1 << z) as f64;
        let tile_h = (self.extent.ymax - self.extent.ymin) / (1 << z) as f64;

        let tile_extent = Extent {
            xmin: self.extent.xmin + tile_w * x as f64,
            ymin: self.extent.ymin + tile_h * y as f64,
            xmax: self.extent.xmin + tile_w * (x + 1) as f64,
            ymax: self.extent.ymin + tile_h * (y + 1) as f64,
        };
        tile_extent
    }

    pub fn web_mercator() -> Self {
        let origin_shift = 20037508.3427892480;
        Self::new(Extent {
            xmin: -origin_shift,
            ymin: -origin_shift,
            xmax: origin_shift,
            ymax: origin_shift,
        })
    }
}
