use bevy::utils::HashMap;

#[derive(Clone, Default)]
pub struct TexturePatch {
    pub origin_x: i16,
    pub origin_y: i16,
    pub patch: i16,
}
#[derive(Clone, Default)]
pub struct TextureEntry {
    pub masked: bool,
    pub width: i16,
    pub height: i16,
    pub patch_count: i16,
    pub patches: Vec<TexturePatch>,
}

#[derive(Clone, Default)]
pub struct Thing {
    pub x: i16,
    pub y: i16,
    pub angle: i16,
    pub thing_type: i16,
    pub flags: i16,
}

#[derive(Clone, Default)]
pub struct Linedef {
    pub start_vert: i16,
    pub end_vert: i16,
    pub flags: i16,
    pub special_type: i16,
    pub sector_tag: i16,
    pub front_sidedef: i16,
    pub back_sidedef: i16,
    pub front: Sidedef,
    pub back: Sidedef,
    pub start: Vert,
    pub end: Vert,
}

#[derive(Clone, Default)]
pub struct Vert {
    pub x: i16,
    pub y: i16,
}

#[derive(Clone, Default)]
pub struct Sidedef {
    pub x_off: i16,
    pub y_off: i16,
    pub upper_tex: [u8; 8],
    pub lower_tex: [u8; 8],
    pub mid_tex: [u8; 8],
    pub sector: i16,
}

#[derive(Clone, Default)]
pub struct Sector {
    pub floor_height: i16,
    pub ceil_height: i16,
    pub floor_tex: [u8; 8],
    pub ceil_tex: [u8; 8],
    pub light_level: i16,
    pub special: i16,
    pub tag: i16,
    pub linedefs: Vec<i16>,
}

#[derive(Default)]
pub struct CompleteMap {
    pub things_vec: Vec<Thing>,
    pub linedef_vec: Vec<Linedef>,
    pub vert_vec: Vec<Vert>,
    pub sector_vec: Vec<Sector>,
    pub sidefef_vec: Vec<Sidedef>,
    pub pnames: Vec<String>,
    pub texture_defs: HashMap<String, TextureEntry>,
}
