use glam::Vec3;
use world::{
    Block, CornerHeading, DirtBlock, FormBlock, MeshTarget, Quad, SandBlock,
    SnowBlock, StoneBlock, MaterialBlock, SurfaceFrame, WedgeHeading,
};

struct QuadCatalogCollector {
    quads: Vec<Quad>,
}

impl QuadCatalogCollector {
    fn empty() -> Self {
        Self { quads: Vec::new() }
    }
}

impl MeshTarget for QuadCatalogCollector {
    fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }
}

enum ShapeKind {
    Cube,
    Slab,
    Wedge,
    RidgeCorner,
    ValleyCorner,
    Peak,
    Stair,
    StairCornerConvex,
    StairCornerConcave,
    GentleLowerWedge,
    GentleUpperWedge,
    GentleLowerRidge,
    GentleUpperRidge,
    GentleLowerValley,
    GentleUpperValley,
}

impl ShapeKind {
    fn all() -> &'static [ShapeKind] {
        &[
            Self::Cube, Self::Slab, Self::Wedge, Self::RidgeCorner, Self::ValleyCorner,
            Self::Peak, Self::Stair, Self::StairCornerConvex, Self::StairCornerConcave,
            Self::GentleLowerWedge, Self::GentleUpperWedge, Self::GentleLowerRidge,
            Self::GentleUpperRidge, Self::GentleLowerValley, Self::GentleUpperValley,
        ]
    }

    fn id(&self) -> &'static str {
        match self {
            Self::Cube => "cube",
            Self::Slab => "slab",
            Self::Wedge => "wedge",
            Self::RidgeCorner => "ridge_corner",
            Self::ValleyCorner => "valley_corner",
            Self::Peak => "peak",
            Self::Stair => "stair",
            Self::StairCornerConvex => "stair_corner_convex",
            Self::StairCornerConcave => "stair_corner_concave",
            Self::GentleLowerWedge => "gentle_lower_wedge",
            Self::GentleUpperWedge => "gentle_upper_wedge",
            Self::GentleLowerRidge => "gentle_lower_ridge",
            Self::GentleUpperRidge => "gentle_upper_ridge",
            Self::GentleLowerValley => "gentle_lower_valley",
            Self::GentleUpperValley => "gentle_upper_valley",
        }
    }

    fn title_de(&self) -> &'static str {
        match self {
            Self::Cube => "Vollblock",
            Self::Slab => "Halbstufe",
            Self::Wedge => "Rampe (45°)",
            Self::RidgeCorner => "Grat-Außenecke (45°)",
            Self::ValleyCorner => "Kehl-Innenecke (45°)",
            Self::Peak => "Pyramidenspitze",
            Self::Stair => "Treppe",
            Self::StairCornerConvex => "Treppen-Außenecke",
            Self::StairCornerConcave => "Treppen-Innenecke",
            Self::GentleLowerWedge => "Sanfte Rampe Unten (2:1)",
            Self::GentleUpperWedge => "Sanfte Rampe Oben (2:1)",
            Self::GentleLowerRidge => "Sanfte Grat-Außenecke Unten",
            Self::GentleUpperRidge => "Sanfte Grat-Außenecke Oben",
            Self::GentleLowerValley => "Sanfte Kehl-Innenecke Unten",
            Self::GentleUpperValley => "Sanfte Kehl-Innenecke Oben",
        }
    }

    fn form_block(&self) -> FormBlock {
        match self {
            Self::Cube => FormBlock::cube(0, 0, 0.0),
            Self::Slab => FormBlock::slab(0, 0, 0.0),
            Self::Wedge => FormBlock::wedge(0, 0, 0.0, WedgeHeading::North),
            Self::RidgeCorner => FormBlock::ridge_corner(0, 0, 0.0, CornerHeading::NorthEast),
            Self::ValleyCorner => FormBlock::valley_corner(0, 0, 0.0, CornerHeading::NorthEast),
            Self::Peak => FormBlock::peak(0, 0, 0.0),
            Self::Stair => FormBlock::stair(0, 0, 0.0, WedgeHeading::North),
            Self::StairCornerConvex => FormBlock::stair_corner_convex(0, 0, 0.0, CornerHeading::NorthEast),
            Self::StairCornerConcave => FormBlock::stair_corner_concave(0, 0, 0.0, CornerHeading::NorthEast),
            Self::GentleLowerWedge => FormBlock::gentle_lower_wedge(0, 0, 0.0, WedgeHeading::North),
            Self::GentleUpperWedge => FormBlock::gentle_upper_wedge(0, 0, 0.0, WedgeHeading::North),
            Self::GentleLowerRidge => FormBlock::gentle_lower_ridge(0, 0, 0.0, CornerHeading::NorthEast),
            Self::GentleUpperRidge => FormBlock::gentle_upper_ridge(0, 0, 0.0, CornerHeading::NorthEast),
            Self::GentleLowerValley => FormBlock::gentle_lower_valley(0, 0, 0.0, CornerHeading::NorthEast),
            Self::GentleUpperValley => FormBlock::gentle_upper_valley(0, 0, 0.0, CornerHeading::NorthEast),
        }
    }
}

enum MaterialKind {
    Form,
    Dirt,
    Sand,
    Snow,
    Stone,
    DarkStone,
    Lava,
}

impl MaterialKind {
    fn all() -> &'static [MaterialKind] {
        &[Self::Form, Self::Dirt, Self::Sand, Self::Snow, Self::Stone, Self::DarkStone, Self::Lava]
    }

    fn id(&self) -> &'static str {
        match self {
            Self::Form => "form",
            Self::Dirt => "dirt",
            Self::Sand => "sand",
            Self::Snow => "snow",
            Self::Stone => "stone",
            Self::DarkStone => "dark_stone",
            Self::Lava => "lava",
        }
    }

    fn title_de(&self) -> &'static str {
        match self {
            Self::Form => "Formblock (Schablone)",
            Self::Dirt => "Erde (Dirt)",
            Self::Sand => "Sand",
            Self::Snow => "Schnee (Snow)",
            Self::Stone => "Stein (Granit)",
            Self::DarkStone => "Dunkler Stein (Schiefer)",
            Self::Lava => "Lava",
        }
    }

    fn make_block(&self, shape: &ShapeKind) -> Box<dyn Block> {
        let form = shape.form_block();
        match self {
            Self::Form => Box::new(form),
            Self::Dirt => Box::new(DirtBlock::natural(form)),
            Self::Sand => Box::new(SandBlock::natural(form)),
            Self::Snow => Box::new(SnowBlock::natural(form)),
            Self::Stone => Box::new(StoneBlock::natural(form)),
            Self::DarkStone => Box::new(StoneBlock::dark(form)),
            Self::Lava => Box::new(MaterialBlock::new(Box::new(form), Vec3::new(0.95, 0.28, 0.05))),
        }
    }
}

fn emit_quad_json(q: &Quad) -> String {
    q.transfer(|v0, v1, v2, v3, n, c| {
        format!(
            "{{\"v\":[[{:.4},{:.4},{:.4}],[{:.4},{:.4},{:.4}],[{:.4},{:.4},{:.4}],[{:.4},{:.4},{:.4}]],\"n\":[{:.4},{:.4},{:.4}],\"c\":[{:.4},{:.4},{:.4}]}}",
            v0.x, v0.y, v0.z, v1.x, v1.y, v1.z, v2.x, v2.y, v2.z, v3.x, v3.y, v3.z, n.x, n.y, n.z, c.x, c.y, c.z
        )
    })
}

fn main() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let mut entries = Vec::new();

    for shape in ShapeKind::all() {
        for mat in MaterialKind::all() {
            let block = mat.make_block(shape);
            let mut collector = QuadCatalogCollector::empty();
            block.emit_surface(&frame, &mut collector);

            let quad_strs: Vec<String> = collector.quads.iter().map(emit_quad_json).collect();
            let key = format!("{}_{}", mat.id(), shape.id());
            let title = format!("{} - {}", mat.title_de(), shape.title_de());
            entries.push(format!(
                "\"{}\":{{\"id\":\"{}\",\"mat_id\":\"{}\",\"shape_id\":\"{}\",\"title\":\"{}\",\"quads\":[{}]}}",
                key, key, mat.id(), shape.id(), title, quad_strs.join(",")
            ));
        }
    }

    println!("{{{}}}", entries.join(","));
}
