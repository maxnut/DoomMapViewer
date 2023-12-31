pub(crate) mod complete_map;

use crate::flat::Flat;
use bevy::prelude::*;
use bevy::render::mesh;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_resource::{AddressMode, Extent3d, Face, TextureFormat};
use bevy::render::texture::ImageSampler;
use bevy::utils::hashbrown::HashMap;
use bevy_earcutr::*;
use complete_map::*;
use tinywad::lump::{LumpData, LumpKind};
use tinywad::lumps::palette::Palettes;
use tinywad::lumps::patch::DoomImage;
use tinywad::models::lump::Lump;
use tinywad::wad::Wad;

pub struct MapManager {
    palette: Palettes,
    pub res_wads: Vec<Wad>,
    pub map: CompleteMap,
    pub tex_map: HashMap<String, Handle<Image>>,
    pub mat_map: HashMap<String, Handle<StandardMaterial>>,
}

impl MapManager {
    pub fn new(iwad_path: String, pwad_path: String, map_ind: i32) -> Self {
        let mut manager = MapManager {
            res_wads: Vec::new(),
            map: CompleteMap::default(),
            palette: Palettes::default(),
            mat_map: HashMap::new(),
            tex_map: HashMap::new(),
        };

        let mut pwad = Wad::new();
        pwad.load_from_file(pwad_path);

        let mut iwad = Wad::new();
        iwad.load_from_file(iwad_path);

        manager.res_wads.push(pwad);
        manager.res_wads.push(iwad);

        let mut map_ind_string = "";

        let s = map_ind.to_string();

        if map_ind > 0
        {
            map_ind_string = s.as_str();
        }

        info!("KUR {}", map_ind_string);

        let things = manager.res_wads[0].lump(format!("THINGS{}", map_ind_string).as_str()).unwrap().data();
        let things_num = things.metadata.size as usize / 10;

        let linedefs = manager.res_wads[0].lump(format!("LINEDEFS{}", map_ind_string).as_str()).unwrap().data();
        let linedefs_num = linedefs.metadata.size as usize / 14;

        let verts = manager.res_wads[0].lump(format!("VERTEXES{}", map_ind_string).as_str()).unwrap().data();
        let verts_num = verts.metadata.size as usize / 4;

        let sectors = manager.res_wads[0].lump(format!("SECTORS{}", map_ind_string).as_str()).unwrap().data();
        let sectors_num = sectors.metadata.size as usize / 26;

        let sidedefs = manager.res_wads[0].lump(format!("SIDEDEFS{}", map_ind_string).as_str()).unwrap().data();
        let sidedefs_num = sidedefs.metadata.size as usize / 30;

        let mut found_pal = false;

        for (wad_ind, wad) in manager.res_wads.iter().enumerate() {
            if !found_pal {
                let pal_opt = wad.lump("PLAYPAL");
                match pal_opt {
                    None => {}
                    Some(x) => {
                        manager.palette = Palettes::new();
                        manager.palette.set_data(x.data());
                        manager.palette.parse();
                        found_pal = true;
                    }
                }
            }

            let mut names: Vec<String> = Vec::new();

            let pnames_opt = wad.lump("PNAMES");
            let pnames: LumpData;
            match pnames_opt {
                None => {
                    manager.map.pnames.push(names);
                    continue;
                }
                Some(x) => pnames = x.data(),
            }

            let pnames_num = i32::from_le_bytes([
                pnames.buffer[0],
                pnames.buffer[1],
                pnames.buffer[2],
                pnames.buffer[3],
            ]) as usize;

            for i in 1..pnames_num {
                let offset = (i * 8) - 4;
                let mut arr = [0; 8];
                arr.copy_from_slice(&pnames.buffer[(offset)..(offset + 8)]);
                names.push(std::str::from_utf8(arr.as_slice()).unwrap().to_string());
            }

            manager.map.pnames.push(names);

            let texture1_opt = wad.lump("TEXTURE1");
            let texture1: LumpData;
            match texture1_opt {
                None => {
                    continue;
                }
                Some(x) => texture1 = x.data(),
            }

            let texture1_num = i32::from_le_bytes([
                texture1.buffer[0],
                texture1.buffer[1],
                texture1.buffer[2],
                texture1.buffer[3],
            ]) as usize;

            for i in 1..texture1_num {
                let tex_offset = i * 4;

                let offset = i32::from_le_bytes([
                    texture1.buffer[tex_offset],
                    texture1.buffer[tex_offset + 1],
                    texture1.buffer[tex_offset + 2],
                    texture1.buffer[tex_offset + 3],
                ]) as usize;

                let mut tex_name = [0; 8];
                tex_name.copy_from_slice(&texture1.buffer[(offset)..(offset + 8)]);

                if manager
                    .map
                    .texture_defs
                    .contains_key(std::str::from_utf8(tex_name.as_slice()).unwrap())
                {
                    continue;
                }

                let mut tex_entry = TextureEntry {
                    width: i16::from_le_bytes([
                        texture1.buffer[offset + 12],
                        texture1.buffer[offset + 13],
                    ]),
                    height: i16::from_le_bytes([
                        texture1.buffer[offset + 14],
                        texture1.buffer[offset + 15],
                    ]),
                    patch_count: i16::from_le_bytes([
                        texture1.buffer[offset + 20],
                        texture1.buffer[offset + 21],
                    ]),
                    wad_ind: wad_ind,
                    ..default()
                };

                for j in 0..tex_entry.patch_count {
                    let entry_offset = offset + 22 + (j as usize * 10);
                    let patch = TexturePatch {
                        origin_x: i16::from_le_bytes([
                            texture1.buffer[entry_offset],
                            texture1.buffer[entry_offset + 1],
                        ]),
                        origin_y: i16::from_le_bytes([
                            texture1.buffer[entry_offset + 2],
                            texture1.buffer[entry_offset + 3],
                        ]),
                        patch: i16::from_le_bytes([
                            texture1.buffer[entry_offset + 4],
                            texture1.buffer[entry_offset + 5],
                        ]),

                        ..default()
                    };

                    tex_entry.patches.push(patch);
                }

                manager.map.texture_defs.insert(
                    std::str::from_utf8(tex_name.as_slice())
                        .unwrap()
                        .to_string(),
                    tex_entry,
                );
            }
        }

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
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        mut images: &mut Assets<Image>,
        mut materials: &mut Assets<StandardMaterial>,
        vert1: Vert,
        vert2: Vert,
        mut floor_height: i16,
        mut ceiling_height: i16,
        off_x: i16,
        off_y: i16,
        tex_name: String,
        backface: bool,
        pegged: i16,
        mid: bool,
    ) {
        if tex_name.as_str().trim_matches(char::from(0)) == "-" {
            return;
        }

        let mut tex_width = 1.;
        let mut tex_height = 1.;

        if self.map.texture_defs.contains_key(&tex_name) {
            match self
                .generate_image_from_texentry(images, self.map.texture_defs[&tex_name].clone())
            {
                Ok(tex) => {
                    let img = images.get(&tex);
                    tex_width = img.unwrap().size().x;
                    tex_height = img.unwrap().size().y;
                }
                Err(err) => {
                    error!(err);
                }
            }
        }

        if mid {
            if pegged == 0 {
                floor_height = ceiling_height - tex_height as i16;
            } else {
                ceiling_height = floor_height + tex_height as i16;
            }
            floor_height += off_y;
            ceiling_height += off_y;
        }

        let len = (Vec2::new(vert1.x as f32, vert1.y as f32)
            - Vec2::new(vert2.x as f32, vert2.y as f32))
        .length();
        let height = (ceiling_height - floor_height) as f32;
        let u = len / (tex_width);
        let v = height / (tex_height);
        let ox = off_x as f32 / tex_width;
        let oy = off_y as f32 / tex_height;

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        let mut vertices: Vec<Vec3>;
        vertices = Vec::new();

        let mut normals: Vec<Vec3>;
        normals = Vec::new();

        let mut indices: Vec<u32>;
        indices = Vec::new();

        let mut uvs: Vec<Vec2>;
        uvs = Vec::new();

        vertices.push(Vec3::new(
            -vert1.x as f32,
            floor_height as f32,
            vert1.y as f32,
        ));
        vertices.push(Vec3::new(
            -vert1.x as f32,
            ceiling_height as f32,
            vert1.y as f32,
        ));
        vertices.push(Vec3::new(
            -vert2.x as f32,
            floor_height as f32,
            vert2.y as f32,
        ));
        vertices.push(Vec3::new(
            -vert2.x as f32,
            ceiling_height as f32,
            vert2.y as f32,
        ));

        if pegged == 2 {
            let sheight = ceiling_height - floor_height;
            let sv = sheight as f32 / (tex_height);

            uvs.push(Vec2::new(ox, 1. - sv + v));
            uvs.push(Vec2::new(ox, 1. - sv));
            uvs.push(Vec2::new(u + ox, 1. - sv + v));
            uvs.push(Vec2::new(u + ox, 1. - sv));
        } else if pegged == 1 {
            uvs.push(Vec2::new(ox, v + oy));
            uvs.push(Vec2::new(ox, oy));
            uvs.push(Vec2::new(u + ox, v + oy));
            uvs.push(Vec2::new(u + ox, oy));
        } else {
            uvs.push(Vec2::new(ox, 1. - oy));
            uvs.push(Vec2::new(ox, 1. - v - oy));
            uvs.push(Vec2::new(u + ox, 1. - oy));
            uvs.push(Vec2::new(u + ox, 1. - v - oy));
        }

        if backface {
            indices.push(0);
            indices.push(1);
            indices.push(2);
            indices.push(2);
            indices.push(1);
            indices.push(3);
        } else {
            indices.push(2);
            indices.push(1);
            indices.push(0);
            indices.push(3);
            indices.push(1);
            indices.push(2);
        }

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

        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

        mesh.set_indices(Some(mesh::Indices::U32(indices)));

        commands.spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material: self.get_texture(&mut images, &mut materials, tex_name, false),
            ..default()
        });
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

            // println!("\nShape");

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

    fn get_patch(
        &mut self,
        mut images: &mut Assets<Image>,
        mut name: String,
    ) -> Result<Handle<Image>, String> {
        name = name.as_str().replace("_flip", "");

        name = name.to_uppercase();

        if self.tex_map.contains_key(&name) {
            return Ok(self.tex_map[&name].clone());
        }

        let mut texture_lump_data: LumpData = LumpData::default();

        let mut found = false;

        for wad in &self.res_wads {
            let lump = wad.lump(name.as_str().trim_matches(char::from(0)));
            if lump.is_some() {
                texture_lump_data = lump.unwrap().data();
                found = true;
                break;
            }
        }

        if !found {
            return Err(format!(
                "Could not get lump for {}",
                name.as_str().trim_matches(char::from(0))
            ));
        };

        if texture_lump_data.metadata.size <= 0 {
            return Err("Lump size was 0".parse().unwrap());
        }

        let (image_data, width, height) = if texture_lump_data.kind == LumpKind::Flat {
            (
                Flat::from_lump(texture_lump_data.buffer.as_slice())
                    .get_image(self.palette.palette().unwrap()),
                64,
                64,
            )
        } else {
            let mut doom_image = DoomImage::new(self.palette.clone(), texture_lump_data);

            doom_image.parse();

            (
                doom_image.buffer(),
                doom_image.img_info.width,
                doom_image.img_info.height,
            )
        };

        if width == 0 || height == 0 {
            return Err(format!(
                "Width or height was 0 {}",
                name.as_str().trim_matches(char::from(0))
            ));
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

        let handle = images.add(coolasstexture);

        self.tex_map.insert(name.clone(), handle.clone());

        Ok(handle)
    }

    fn generate_image_from_texentry(
        &mut self,
        mut images: &mut Assets<Image>,
        entry: TextureEntry,
    ) -> Result<Handle<Image>, String> {
        let mut data: Vec<u8> = vec![0; entry.width as usize * entry.height as usize * 4];

        for patch in entry.patches {
            match self.get_patch(
                images,
                self.map.pnames[entry.wad_ind][patch.patch as usize].clone(),
            ) {
                Ok(image) => {
                    if patch.origin_x < 0 || patch.origin_y < 0 {
                        continue;
                    }

                    let offset_x = patch.origin_x as usize;
                    let offset_y = patch.origin_y as usize;

                    let size = images.get(&image).unwrap().size();
                    let pixels = images.get(&image).unwrap().data.clone();

                    for y in 0..size.y as usize {
                        for x in 0..size.x as usize {
                            let src_index = y * size.x as usize * 4 + x * 4;
                            let dest_index =
                                (offset_y + y) * entry.width as usize * 4 + (offset_x + x) * 4;

                            for i in 0..4 {
                                if dest_index + i >= data.len() {
                                    break;
                                }
                                data[dest_index + i] = pixels[src_index + i];
                            }
                        }
                    }
                }
                Err(str) => error!(str),
            };
        }

        let ext: Extent3d = Extent3d {
            width: entry.width as u32,
            height: entry.height as u32,
            ..default()
        };

        let mut image = Image::new_fill(
            ext,
            bevy::render::render_resource::TextureDimension::D2,
            data.as_slice(),
            TextureFormat::Rgba8Unorm,
        );

        let mut descriptor = ImageSampler::nearest_descriptor();
        descriptor.address_mode_u = AddressMode::Repeat;
        descriptor.address_mode_v = AddressMode::Repeat;
        image.sampler_descriptor = ImageSampler::Descriptor(descriptor);

        Ok(images.add(image))
    }

    pub fn get_texture(
        &mut self,
        mut images: &mut Assets<Image>,
        mut materials: &mut Assets<StandardMaterial>,
        mut name: String,
        flip: bool,
    ) -> Handle<StandardMaterial> {
        if flip {
            name += "_flip";
        }

        if self.mat_map.contains_key(&name) {
            return self.mat_map[&name].clone();
        }

        let mut coolasstexture: Handle<Image> = Handle::default();

        if self.map.texture_defs.contains_key(&name) {
            // info!("Found {}", name);
            match self.generate_image_from_texentry(images, self.map.texture_defs[&name].clone()) {
                Ok(tex) => coolasstexture = tex,
                Err(err) => {
                    error!(err);
                    return Handle::default();
                }
            }
        } else {
            match self.get_patch(images, name.clone()) {
                Ok(tex) => coolasstexture = tex,
                Err(err) => {
                    error!(err);
                    return Handle::default();
                }
            }
        }

        let material = materials.add(StandardMaterial {
            base_color_texture: Some(coolasstexture),
            cull_mode: if flip {
                Some(Face::Front)
            } else {
                Some(Face::Back)
            },
            unlit: true,
            ..Default::default()
        });

        self.mat_map.insert(name, material.clone());

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

        // print!("\n");

        for vec in vert_contains_list {
            // println!("({}, {})", vec.x, vec.y);
            vertices.push(vec.x as f64);
            vertices.push(vec.y as f64);
        }

        return vertices;
    }
}
