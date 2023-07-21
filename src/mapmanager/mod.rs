pub(crate) mod completemap;

use crate::flat::Flat;
use bevy::prelude::*;
use bevy::render::render_resource::{AddressMode, Extent3d, TextureFormat};
use bevy::render::texture::ImageSampler;
use bevy::utils::hashbrown::HashMap;
use bevy_earcutr::*;
use completemap::*;
use tinywad::lump::LumpKind;
use tinywad::lumps::palette::Palettes;
use tinywad::lumps::patch::DoomImage;
use tinywad::models::lump::Lump;
use tinywad::wad::Wad;

pub struct MapManager {
    palette: Palettes,
    pub wad: Wad,
    pub map: CompleteMap,
    pub tex_map: HashMap<String, Handle<StandardMaterial>>,
}

impl MapManager {
    pub fn new() -> Self {
        let mut manager = MapManager {
            wad: Wad::new(),
            map: CompleteMap::default(),
            palette: Palettes::default(),
            tex_map: HashMap::new(),
        };

        manager.wad.load_from_file("DOOM2.wad");

        let things = manager.wad.lump("THINGS").unwrap().data();
        let things_num = things.metadata.size as usize / 10;

        let linedefs = manager.wad.lump("LINEDEFS").unwrap().data();
        let linedefs_num = linedefs.metadata.size as usize / 14;

        let verts = manager.wad.lump("VERTEXES").unwrap().data();
        let verts_num = verts.metadata.size as usize / 4;

        let sectors = manager.wad.lump("SECTORS").unwrap().data();
        let sectors_num = sectors.metadata.size as usize / 26;

        let sidedefs = manager.wad.lump("SIDEDEFS").unwrap().data();
        let sidedefs_num = sidedefs.metadata.size as usize / 30;

        let paldata = manager.wad.lump("PLAYPAL").unwrap().data();

        manager.palette = Palettes::new();
        manager.palette.set_data(paldata);
        manager.palette.parse();

        for i in 0..things_num {
            let thing_offset = i * 10;
            let thing = Thing {
                x: i16::from_le_bytes([
                    things.buffer[thing_offset],
                    things.buffer[thing_offset + 1],
                ]),
                y: i16::from_le_bytes([
                    things.buffer[thing_offset + 2],
                    things.buffer[thing_offset + 3],
                ]),
                angle: i16::from_le_bytes([
                    things.buffer[thing_offset + 4],
                    things.buffer[thing_offset + 5],
                ]),
                thing_type: i16::from_le_bytes([
                    things.buffer[thing_offset + 6],
                    things.buffer[thing_offset + 7],
                ]),
                flags: i16::from_le_bytes([
                    things.buffer[thing_offset + 8],
                    things.buffer[thing_offset + 9],
                ]),
            };
            manager.map.things_vec.push(thing);
        }

        for i in 0..linedefs_num {
            let line_offset = i * 14;
            let line = Linedef {
                start_vert: i16::from_le_bytes([
                    linedefs.buffer[line_offset],
                    linedefs.buffer[line_offset + 1],
                ]),
                end_vert: i16::from_le_bytes([
                    linedefs.buffer[line_offset + 2],
                    linedefs.buffer[line_offset + 3],
                ]),
                flags: i16::from_le_bytes([
                    linedefs.buffer[line_offset + 4],
                    linedefs.buffer[line_offset + 5],
                ]),
                special_type: i16::from_le_bytes([
                    linedefs.buffer[line_offset + 6],
                    linedefs.buffer[line_offset + 7],
                ]),
                sector_tag: i16::from_le_bytes([
                    linedefs.buffer[line_offset + 8],
                    linedefs.buffer[line_offset + 9],
                ]),
                front_sidedef: i16::from_le_bytes([
                    linedefs.buffer[line_offset + 10],
                    linedefs.buffer[line_offset + 11],
                ]),
                back_sidedef: i16::from_le_bytes([
                    linedefs.buffer[line_offset + 12],
                    linedefs.buffer[line_offset + 13],
                ]),
                ..default()
            };
            manager.map.linedef_vec.push(line);
        }

        for i in 0..verts_num {
            let vert_offset = i * 4;
            let vert = Vert {
                x: i16::from_le_bytes([verts.buffer[vert_offset], verts.buffer[vert_offset + 1]]),
                y: i16::from_le_bytes([
                    verts.buffer[vert_offset + 2],
                    verts.buffer[vert_offset + 3],
                ]),
            };
            manager.map.vert_vec.push(vert);
        }

        for i in 0..sectors_num {
            let sec_offset = i * 26;
            let sec = Sector {
                floor_height: i16::from_le_bytes([
                    sectors.buffer[sec_offset],
                    sectors.buffer[sec_offset + 1],
                ]),
                ceil_height: i16::from_le_bytes([
                    sectors.buffer[sec_offset + 2],
                    sectors.buffer[sec_offset + 3],
                ]),
                floor_tex: {
                    let mut arr = [0; 8];
                    arr.copy_from_slice(&sectors.buffer[(sec_offset + 4)..(sec_offset + 12)]);
                    arr
                },
                ceil_tex: {
                    let mut arr = [0; 8];
                    arr.copy_from_slice(&sectors.buffer[(sec_offset + 12)..(sec_offset + 20)]);
                    arr
                },
                light_level: i16::from_le_bytes([
                    sectors.buffer[sec_offset + 20],
                    sectors.buffer[sec_offset + 21],
                ]),
                special: i16::from_le_bytes([
                    sectors.buffer[sec_offset + 22],
                    sectors.buffer[sec_offset + 23],
                ]),
                tag: i16::from_le_bytes([
                    sectors.buffer[sec_offset + 24],
                    sectors.buffer[sec_offset + 25],
                ]),
                ..Default::default()
            };
            manager.map.sector_vec.push(sec);
        }

        for i in 0..sidedefs_num {
            let side_offset = i * 30;
            let side: Sidedef = Sidedef {
                x_off: i16::from_le_bytes([
                    sidedefs.buffer[side_offset],
                    sidedefs.buffer[side_offset + 1],
                ]),
                y_off: i16::from_le_bytes([
                    sidedefs.buffer[side_offset + 2],
                    sidedefs.buffer[side_offset + 3],
                ]),
                upper_tex: {
                    let mut arr = [0; 8];
                    arr.copy_from_slice(&sidedefs.buffer[(side_offset + 4)..(side_offset + 12)]);
                    arr
                },
                lower_tex: {
                    let mut arr = [0; 8];
                    arr.copy_from_slice(&sidedefs.buffer[(side_offset + 12)..(side_offset + 20)]);
                    arr
                },
                mid_tex: {
                    let mut arr = [0; 8];
                    arr.copy_from_slice(&sidedefs.buffer[(side_offset + 20)..(side_offset + 28)]);
                    arr
                },
                sector: i16::from_le_bytes([
                    sidedefs.buffer[side_offset + 28],
                    sidedefs.buffer[side_offset + 29],
                ]),
            };
            manager.map.sidefef_vec.push(side);
        }

        return manager;
    }

    pub fn generate_wall(
        verts: &mut Vec<Vec3>,
        indices: &mut Vec<u32>,
        uvs: &mut Vec<Vec2>,
        normals: &mut Vec<Vec3>,
        vert1: Vert,
        vert2: Vert,
        floor_height: i16,
        ceiling_height: i16,
        ind_index: &mut u32,
        backface: bool,
    ) {
        verts.push(Vec3::new(
            -vert1.x as f32,
            floor_height as f32,
            vert1.y as f32,
        ));
        verts.push(Vec3::new(
            -vert1.x as f32,
            ceiling_height as f32,
            vert1.y as f32,
        ));
        verts.push(Vec3::new(
            -vert2.x as f32,
            floor_height as f32,
            vert2.y as f32,
        ));
        verts.push(Vec3::new(
            -vert2.x as f32,
            ceiling_height as f32,
            vert2.y as f32,
        ));

        if backface {
            indices.push(*ind_index + 0);
            indices.push(*ind_index + 1);
            indices.push(*ind_index + 2);
            indices.push(*ind_index + 2);
            indices.push(*ind_index + 1);
            indices.push(*ind_index + 3);
        } else {
            indices.push(*ind_index + 2);
            indices.push(*ind_index + 1);
            indices.push(*ind_index + 0);
            indices.push(*ind_index + 3);
            indices.push(*ind_index + 1);
            indices.push(*ind_index + 2);
        }

        *ind_index += 4;

        let _uvs = [
            Vec2::new(0.0, 1.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 0.0),
        ];

        uvs.push(_uvs[0]);
        uvs.push(_uvs[1]);
        uvs.push(_uvs[2]);
        uvs.push(_uvs[3]);

        let normal = Vec3::new(
            vert1.x as f32 - vert2.x as f32,
            vert1.y as f32 - vert2.y as f32,
            0.,
        )
        .cross(Vec3::Y)
        .normalize();
        normals.push(normal);
        normals.push(normal);
        normals.push(normal);
        normals.push(normal);
    }

    //this took 3 days to figure out bruh
    pub fn detect_shapes(&self, sector: &Sector) -> Vec<Vec<i16>> {
        let mut shapes: Vec<Vec<i16>> = Vec::new();
        let mut order_count: usize = 0;

        let mut analyzed: Vec<i16> = Vec::new();

        let mut current_linedef = -1;
        let mut current_vert = -1;

        while order_count < sector.linedefs.len() {
            let mut shape: Vec<i16> = Vec::new();

            let mut found = false;

            for i in sector.linedefs.clone() {
                if !analyzed.contains(&i) {
                    current_linedef = i;
                    found = true;
                    break;
                }
            }

            if !found {
                break;
            }

            current_vert = self.map.linedef_vec[current_linedef as usize].end_vert;

            println!("\nShape");

            loop {
                shape.push(current_linedef);

                analyzed.push(current_linedef);

                let prev = order_count;

                for line in sector.linedefs.clone() {
                    if shape.contains(&line) {
                        continue;
                    }

                    let linedef2 = &self.map.linedef_vec[line as usize];

                    if current_vert == linedef2.start_vert || current_vert == linedef2.end_vert {
                        // println!("Found line {} for line {}", line + 1, current_linedef + 1);
                        current_linedef = line;
                        current_vert = if linedef2.start_vert == current_vert {
                            linedef2.end_vert
                        } else {
                            linedef2.start_vert
                        };
                        order_count += 1;
                        break;
                    }
                }

                if order_count == prev {
                    // println!("Did not find anything for line {}", current_linedef + 1);
                    break;
                }
            }

            // println!("Final shape");

            // for i in shape.clone() {
            //     println!("{}", i + 1);
            // }

            shapes.push(shape);
        }

        shapes
    }

    pub fn triangulate_polygon_with_holes(body: &mut Vec<f64>, holes: &[Vec<f64>]) -> Option<Mesh> {
        const DIM: usize = 2;

        let mut builder = PolygonMeshBuilder::default();

        let mut hole_indices: Vec<usize> = Vec::new();

        for hole in holes {
            hole_indices.push(body.len() / DIM);
            body.extend(hole);
        }

        builder.add_earcutr_input(EarcutrInput {
            vertices: body.to_vec(),
            interior_indices: hole_indices.to_vec(),
        });

        return builder.build();
    }

    pub fn get_texture(
        &mut self,
        mut images: &mut Assets<Image>,
        mut materials: &mut Assets<StandardMaterial>,
        name: String,
    ) -> Handle<StandardMaterial> {
        if self.tex_map.contains_key(&name) {
            return self.tex_map[&name].clone();
        }

        let Some(texture_lump) = self.wad.lump(name.as_str())
        else
        {
            return Handle::default();
        };

        let texture_lump_data = texture_lump.data();

        info!(texture_lump_data.metadata.size);
        info!(
            "{}",
            std::str::from_utf8(&texture_lump_data.metadata.name).unwrap()
        );

        if texture_lump_data.metadata.size <= 0 {
            return Handle::default();
        }

        let (image_data, width, height) = if texture_lump_data.kind == LumpKind::Flat {
            (
                Flat::from_lump(texture_lump_data.buffer.as_slice())
                    .get_image(self.palette.palette().unwrap()),
                4096,
                4096,
            )
        } else {
            let mut slime16 = DoomImage::new(self.palette.clone(), texture_lump_data);

            slime16.parse();

            (
                slime16.buffer(),
                slime16.img_info.width,
                slime16.img_info.height,
            )
        };

        let ext: Extent3d = Extent3d {
            width: width as u32,
            height: height as u32,
            ..default()
        };

        let mut coolasstexture = Image::new_fill(
            ext,
            bevy::render::render_resource::TextureDimension::D2,
            image_data.as_slice(),
            TextureFormat::Rgba8Unorm,
        );

        let mut descriptor = ImageSampler::nearest_descriptor();

        descriptor.address_mode_u = AddressMode::Repeat;
        descriptor.address_mode_v = AddressMode::Repeat;

        coolasstexture.sampler_descriptor = ImageSampler::Descriptor(descriptor);

        let material = materials.add(StandardMaterial {
            base_color_texture: Some(images.add(coolasstexture)),
            ..Default::default()
        });

        self.tex_map.insert(name, material.clone());

        return material;
    }

    pub fn get_linedef_vector_as_vertices(&self, vec: &Vec<i16>) -> Vec<f64> {
        let mut vertices: Vec<f64>;
        vertices = Vec::new();
        let mut vert_contains_list: Vec<Vec2>;
        vert_contains_list = Vec::new();

        for line_index in vec {
            let linedef = &self.map.linedef_vec[*line_index as usize];

            let vert1 = &self.map.vert_vec[linedef.start_vert as usize];
            let vert2 = &self.map.vert_vec[linedef.end_vert as usize];

            let coords = vec![
                vert1.x as f64,
                vert1.y as f64,
                vert2.x as f64,
                vert2.y as f64,
            ];

            if !vert_contains_list.contains(&Vec2::new(coords[0] as f32, coords[1] as f32)) {
                vert_contains_list.push(Vec2::new(coords[0] as f32, coords[1] as f32));
            }

            if !vert_contains_list.contains(&Vec2::new(coords[2] as f32, coords[3] as f32)) {
                vert_contains_list.push(Vec2::new(coords[2] as f32, coords[3] as f32));
            }
        }

        print!("\n");

        for vec in vert_contains_list {
            println!("({}, {})", vec.x, vec.y);
            vertices.push(vec.x as f64);
            vertices.push(vec.y as f64);
        }

        return vertices;
    }
}
