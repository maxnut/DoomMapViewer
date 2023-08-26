
use crate::AppState;
use crate::mapmanager::complete_map::{Sector, Vert};
use crate::mapmanager::MapManager;
use crate::state::GameState;
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_earcutr::{EarcutrInput, PolygonMeshBuilder};


pub struct MapViewPlugin;

impl Plugin for MapViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnEnter(GameState::MapView)));
    }
}

fn point_inside_aabb(aabb_min: Vec2, aabb_max: Vec2, point: Vert) -> bool {
    point.x > aabb_min.x as i16
        && point.x < aabb_max.x as i16
        && point.y > aabb_min.y as i16
        && point.y < aabb_max.y as i16
}

fn spawn_floor(
    mut commands: &mut Commands,
    mut floor_vertices: &mut Vec<f64>,
    holes: Vec<Vec<f64>>,
    mut meshes: &mut Assets<Mesh>,
    mut images: &mut Assets<Image>,
    mut materials: &mut Assets<StandardMaterial>,
    sector: &Sector,
    mapmanager: &mut MapManager
) {
    let mesh_floor: Option<Mesh>;

    if holes.len() > 0 {
        mesh_floor = MapManager::triangulate_polygon_with_holes(&mut floor_vertices, &holes);
    } else {
        let mut builder = PolygonMeshBuilder::default();

        builder.add_earcutr_input(EarcutrInput {
            vertices: floor_vertices.clone(),
            interior_indices: Vec::new(),
        });

        mesh_floor = builder.build();
    }

    match mesh_floor {
        Some(_) => {
            let mut actual_mesh = mesh_floor.unwrap();

            let mut uvs: Vec<Vec2> = Vec::new();

            for i in (0..floor_vertices.len()).step_by(2) {
                uvs.push(Vec2::new(
                    floor_vertices[i] as f32 / 64.,
                    floor_vertices[i + 1] as f32 / 64.,
                ));
            }

            actual_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

            commands
                .spawn(PbrBundle {
                    mesh: meshes.add(actual_mesh.clone()),
                    material: mapmanager.get_texture(
                        &mut images,
                        &mut materials,
                        std::str::from_utf8(&sector.floor_tex).unwrap().to_string(),
                        false,
                    ),
                    ..default()
                })
                .insert(Transform {
                    translation: Vec3 {
                        x: 0.,
                        y: sector.floor_height as f32,
                        z: 0.,
                    },
                    rotation: Quat::from_rotation_x(-90.0f32.to_radians())
                        * Quat::from_rotation_z(180.0f32.to_radians()),
                    ..Default::default()
                });

            commands
                .spawn(PbrBundle {
                    mesh: meshes.add(actual_mesh.clone()),
                    material: mapmanager.get_texture(
                        &mut images,
                        &mut materials,
                        std::str::from_utf8(&sector.ceil_tex).unwrap().to_string(),
                        true,
                    ),
                    ..default()
                })
                .insert(Transform {
                    translation: Vec3 {
                        x: 0.,
                        y: sector.ceil_height as f32,
                        z: 0.,
                    },
                    rotation: Quat::from_rotation_x(-90.0f32.to_radians())
                        * Quat::from_rotation_z(180.0f32.to_radians()),
                    ..Default::default()
                });
        }
        None => println!("Merda culo"),
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut windows: Query<&mut Window>,
    mut appstate: ResMut<AppState>
) {
    let mut window = windows.single_mut();

    window.present_mode = PresentMode::AutoNoVsync;
    // window.mode = WindowMode::Fullscreen;

    let mut mapmanager = MapManager::new(appstate.iwad_path.clone(), appstate.pwad_path.clone(), appstate.map_ind);

    for (i, linedef) in mapmanager.map.linedef_vec.clone().iter_mut().enumerate() {
        if linedef.front_sidedef >= 0 {
            let front_sidedef = &mapmanager.map.sidefef_vec[linedef.front_sidedef as usize].clone();

            linedef.front = front_sidedef.clone();

            if linedef.back_sidedef >= 0 {
                let back_sidedef =
                    &mapmanager.map.sidefef_vec[linedef.back_sidedef as usize].clone();
                linedef.back = back_sidedef.clone();
                if front_sidedef.sector != back_sidedef.sector {
                    mapmanager.map.sector_vec[front_sidedef.sector as usize]
                        .linedefs
                        .push(i as i16);
                    mapmanager.map.sector_vec[back_sidedef.sector as usize]
                        .linedefs
                        .push(i as i16);
                }
            } else {
                mapmanager.map.sector_vec[front_sidedef.sector as usize]
                    .linedefs
                    .push(i as i16);
            }

            let front_sec = &mapmanager.map.sector_vec[front_sidedef.sector as usize].clone();

            let vert1 = &mapmanager.map.vert_vec[linedef.start_vert as usize].clone();
            let vert2 = &mapmanager.map.vert_vec[linedef.end_vert as usize].clone();

            linedef.start = vert1.clone();
            linedef.end = vert2.clone();

            if linedef.back_sidedef >= 0 {
                let back_sidedef =
                    &mapmanager.map.sidefef_vec[linedef.back_sidedef as usize].clone();

                let back_sec = &mapmanager.map.sector_vec[back_sidedef.sector as usize].clone();

                if front_sec.ceil_height > back_sec.ceil_height {
                    mapmanager.generate_wall(
                        &mut commands,
                        &mut meshes,
                        &mut images,
                        &mut materials,
                        vert1.clone(),
                        vert2.clone(),
                        back_sec.ceil_height,
                        front_sec.ceil_height,
                        front_sidedef.x_off,
                        if (linedef.flags & (1 << 3)) != 0 {
                            front_sidedef.y_off
                        } else {
                            -front_sidedef.y_off
                        },
                        std::str::from_utf8(front_sidedef.upper_tex.as_slice())
                            .unwrap()
                            .to_string(),
                        false,
                        if (linedef.flags & (1 << 3)) != 0 {
                            0
                        } else {
                            1
                        },
                        false,
                    );
                }

                if front_sec.ceil_height < back_sec.ceil_height {
                    mapmanager.generate_wall(
                        &mut commands,
                        &mut meshes,
                        &mut images,
                        &mut materials,
                        vert1.clone(),
                        vert2.clone(),
                        front_sec.ceil_height,
                        back_sec.ceil_height,
                        back_sidedef.x_off,
                        if (linedef.flags & (1 << 3)) != 0 {
                            back_sidedef.y_off
                        } else {
                            -back_sidedef.y_off
                        },
                        std::str::from_utf8(back_sidedef.upper_tex.as_slice())
                            .unwrap()
                            .to_string(),
                        true,
                        if (linedef.flags & (1 << 3)) != 0 {
                            0
                        } else {
                            1
                        },
                        false,
                    );
                }

                if front_sec.floor_height < back_sec.floor_height {
                    mapmanager.generate_wall(
                        &mut commands,
                        &mut meshes,
                        &mut images,
                        &mut materials,
                        vert1.clone(),
                        vert2.clone(),
                        front_sec.floor_height,
                        back_sec.floor_height,
                        front_sidedef.x_off,
                        front_sidedef.y_off,
                        std::str::from_utf8(front_sidedef.lower_tex.as_slice())
                            .unwrap()
                            .to_string(),
                        false,
                        if (linedef.flags & (1 << 4)) != 0 {
                            2
                        } else {
                            0
                        },
                        false,
                    );
                }

                if front_sec.floor_height > back_sec.floor_height {
                    mapmanager.generate_wall(
                        &mut commands,
                        &mut meshes,
                        &mut images,
                        &mut materials,
                        vert1.clone(),
                        vert2.clone(),
                        back_sec.floor_height,
                        front_sec.floor_height,
                        back_sidedef.x_off,
                        front_sidedef.y_off,
                        std::str::from_utf8(back_sidedef.lower_tex.as_slice())
                            .unwrap()
                            .to_string(),
                        true,
                        if (linedef.flags & (1 << 4)) != 0 {
                            2
                        } else {
                            0
                        },
                        false,
                    );
                }

                mapmanager.generate_wall(
                    &mut commands,
                    &mut meshes,
                    &mut images,
                    &mut materials,
                    vert1.clone(),
                    vert2.clone(),
                    i16::max(front_sec.floor_height, back_sec.floor_height),
                    i16::min(front_sec.ceil_height, back_sec.ceil_height),
                    front_sidedef.x_off,
                    front_sidedef.y_off,
                    std::str::from_utf8(front_sidedef.mid_tex.as_slice())
                        .unwrap()
                        .to_string(),
                    false,
                    if (linedef.flags & (1 << 4)) != 0 {
                        1
                    } else {
                        0
                    },
                    true,
                );

                mapmanager.generate_wall(
                    &mut commands,
                    &mut meshes,
                    &mut images,
                    &mut materials,
                    vert1.clone(),
                    vert2.clone(),
                    i16::max(front_sec.floor_height, back_sec.floor_height),
                    i16::min(front_sec.ceil_height, back_sec.ceil_height),
                    back_sidedef.x_off,
                    back_sidedef.y_off,
                    std::str::from_utf8(back_sidedef.mid_tex.as_slice())
                        .unwrap()
                        .to_string(),
                    true,
                    if (linedef.flags & (1 << 4)) != 0 {
                        1
                    } else {
                        0
                    },
                    true,
                );
            } else {
                mapmanager.generate_wall(
                    &mut commands,
                    &mut meshes,
                    &mut images,
                    &mut materials,
                    vert1.clone(),
                    vert2.clone(),
                    front_sec.floor_height,
                    front_sec.ceil_height,
                    front_sidedef.x_off,
                    front_sidedef.y_off,
                    std::str::from_utf8(front_sidedef.mid_tex.as_slice())
                        .unwrap()
                        .to_string(),
                    false,
                    if (linedef.flags & (1 << 4)) != 0 {
                        1
                    } else {
                        0
                    },
                    false,
                );
            }
        }
    }

    for sector in &mapmanager.map.sector_vec.clone() {
        let shapes = mapmanager.detect_shapes(sector);

        if shapes.len() <= 0
        {
            continue;
        }

        info!("Detected shapes: {}", shapes.len());

        let mut biggest_area = f32::MIN;
        let mut biggest_aabb_index = 0;

        let mut holes: Vec<Vec<f64>>;
        holes = Vec::new();

        let mut aabb_min = Vec2::new(-1., -1.);
        let mut aabb_max = Vec2::new(-1., -1.);

        for (j, shape) in shapes.clone().into_iter().enumerate() {
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;

            let shape_vec = mapmanager.get_linedef_vector_as_vertices(&shape);

            holes.push(shape_vec);

            for i in shape {
                let linedef = &mapmanager.map.linedef_vec[i as usize];
                let vert1 = &mapmanager.map.vert_vec[linedef.start_vert as usize];
                let vert2 = &mapmanager.map.vert_vec[linedef.end_vert as usize];

                let x1 = vert1.x as f32;
                let y1 = vert1.y as f32;
                let x2 = vert2.x as f32;
                let y2 = vert2.y as f32;

                min_x = min_x.min(x1).min(x2);
                max_x = max_x.max(x1).max(x2);
                min_y = min_y.min(y1).min(y2);
                max_y = max_y.max(y1).max(y2);
            }

            let _aabb_min = Vec2::new(min_x, min_y);
            let _aabb_max = Vec2::new(max_x, max_y);

            let area = (_aabb_max.x - _aabb_min.x) * (_aabb_max.y - _aabb_min.y);

            if area > biggest_area {
                biggest_area = area;
                biggest_aabb_index = j;
                aabb_max = _aabb_max;
                aabb_min = _aabb_min;
            }
        }

        let mut floor_vertices: Vec<f64>;
        floor_vertices = holes[biggest_aabb_index].clone();

        for (j, shape) in shapes.clone().into_iter().enumerate() {
            if j == biggest_aabb_index {
                continue;
            }

            let mut inside: bool;
            for index in shape.clone() {
                let line = &mapmanager.map.linedef_vec[index as usize];
                let s_vert = &mapmanager.map.vert_vec[line.start_vert as usize];
                let e_vert = &mapmanager.map.vert_vec[line.start_vert as usize];

                inside = point_inside_aabb(aabb_min, aabb_max, s_vert.clone());

                inside = inside && point_inside_aabb(aabb_min, aabb_max, e_vert.clone());

                if !inside {
                    let mut shape_vec = mapmanager.get_linedef_vector_as_vertices(&shape);
                    spawn_floor(&mut commands, &mut shape_vec, Vec::new(), &mut meshes, &mut images, &mut materials, sector, &mut mapmanager);
                    if let Some(index) = holes.iter().position(|x| x == &shape_vec) {
                        holes.remove(index);
                    }
                    break;
                }
            }
        }

        let shape_vec = mapmanager.get_linedef_vector_as_vertices(&shapes[biggest_aabb_index]);
        if let Some(index) = holes.iter().position(|x| x == &shape_vec) {
            holes.remove(index);
        }

        spawn_floor(&mut commands, &mut floor_vertices, holes, &mut meshes, &mut images, &mut materials, sector, &mut mapmanager);
    }

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 0., 5.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
