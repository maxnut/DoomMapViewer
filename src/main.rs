mod flat;
#[allow(clippy::too_many_arguments)]
mod mapmanager;

use bevy::prelude::*;
use bevy::render::mesh::{self, PrimitiveTopology};
use bevy::render::render_resource::Face;
use bevy::render::texture::Image;
use mapmanager::completemap::*;
use mapmanager::*;

use bevy_earcutr::*;
use bevy_editor_pls::EditorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EditorPlugin::default())
        .add_startup_system(setup)
        // .add_system(camera_movement_system)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut mapmanager = MapManager::new();

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
                        std::str::from_utf8(front_sidedef.upper_tex.as_slice())
                            .unwrap()
                            .to_string(),
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
                        std::str::from_utf8(back_sidedef.upper_tex.as_slice())
                            .unwrap()
                            .to_string(),
                        true,
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
                        std::str::from_utf8(front_sidedef.lower_tex.as_slice())
                            .unwrap()
                            .to_string(),
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
                        std::str::from_utf8(back_sidedef.lower_tex.as_slice())
                            .unwrap()
                            .to_string(),
                        true,
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
                    std::str::from_utf8(front_sidedef.mid_tex.as_slice())
                        .unwrap()
                        .to_string(),
                    false,
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
                    std::str::from_utf8(back_sidedef.mid_tex.as_slice())
                        .unwrap()
                        .to_string(),
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
                    std::str::from_utf8(front_sidedef.mid_tex.as_slice())
                        .unwrap()
                        .to_string(),
                    false,
                );
            }
        }
    }

    for sector in &mapmanager.map.sector_vec.clone() {
        // println!("\nNew Vector {} ---------------------", sector.light_level);

        let shapes = mapmanager.detect_shapes(sector);

        // println!("Detected shapes: {}", shapes.len());

        let mut biggest_area = f32::MIN;
        let mut biggest_aabb_index = 0;

        let mut holes: Vec<Vec<f64>>;
        holes = Vec::new();

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

            let aabb_min = Vec2::new(min_x, min_y);
            let aabb_max = Vec2::new(max_x, max_y);

            let area = (aabb_max.x - aabb_min.x) * (aabb_max.y - aabb_min.y);

            if area > biggest_area {
                biggest_area = area;
                biggest_aabb_index = j;
            }
        }

        let mut floor_vertices: Vec<f64>;
        floor_vertices = holes[biggest_aabb_index].clone();

        holes.remove(biggest_aabb_index);

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
                let actual_mesh = mesh_floor.unwrap();
                commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(actual_mesh.clone()),
                        material: mapmanager.get_texture(
                            &mut images,
                            &mut materials,
                            std::str::from_utf8(&sector.floor_tex).unwrap().to_string(),
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

                //TODO: save flipped materials in hashmap

                let ceil_mat_handle = mapmanager.get_texture(
                    &mut images,
                    &mut materials,
                    std::str::from_utf8(&sector.ceil_tex).unwrap().to_string(),
                );

                let mut ceil_mat = materials.get(&ceil_mat_handle).unwrap().clone();
                ceil_mat.cull_mode = Some(Face::Front);

                commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(actual_mesh.clone()),
                        material: materials.add(ceil_mat),
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

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 0., 5.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn camera_movement_system(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    for mut transform in query.iter_mut() {
        let mut _rotation = transform.rotation;
        let translation = &mut transform.translation;
        let mut direction = Vec3::ZERO;
        let mut rot = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::W) {
            direction += _rotation.mul_vec3(-Vec3::Z);
        }
        if keyboard_input.pressed(KeyCode::S) {
            direction += _rotation.mul_vec3(Vec3::Z);
        }
        if keyboard_input.pressed(KeyCode::A) {
            direction += _rotation.mul_vec3(-Vec3::X);
        }
        if keyboard_input.pressed(KeyCode::D) {
            direction += _rotation.mul_vec3(Vec3::X);
        }
        if keyboard_input.pressed(KeyCode::Q) {
            direction += -Vec3::Y * 0.3;
        }
        if keyboard_input.pressed(KeyCode::E) {
            direction += Vec3::Y * 0.3;
        }

        if keyboard_input.pressed(KeyCode::Right) {
            rot += -Vec3::Y;
        }
        if keyboard_input.pressed(KeyCode::Left) {
            rot += Vec3::Y;
        }

        if keyboard_input.pressed(KeyCode::Up) {
            rot += -Vec3::X;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            rot += Vec3::X;
        }

        let translation_delta = direction * 1000.0 * time.delta_seconds();
        let rot_delta = rot * 2.0 * time.delta_seconds();
        translation.x += translation_delta.x;
        translation.z += translation_delta.z;
        translation.y += translation_delta.y;

        let rotation_quat_y = Quat::from_rotation_y(rot_delta.y);
        transform.rotation *= rotation_quat_y;

        let rotation_quat_x = Quat::from_rotation_x(rot_delta.x);
        transform.rotation *= rotation_quat_x;
    }
}
