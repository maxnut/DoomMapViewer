mod mapmanager;

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::render::mesh::{self, PrimitiveTopology};
use bevy::render::texture::Image;
use mapmanager::completemap::*;
use mapmanager::*;

use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};

use bevy_earcutr::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_startup_system(setup)
        .add_system(camera_movement_system)
        .run();
}

fn setup(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    wireframe_config.global = true;

    let mut mapmanager = MapManager::new();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mut vertices: Vec<Vec3>;
    vertices = Vec::new();

    let mut normals: Vec<Vec3>;
    normals = Vec::new();

    let mut indices: Vec<u32>;
    indices = Vec::new();

    let mut uvs: Vec<Vec2>;
    uvs = Vec::new();

    let mut indices_index = 0;

    let mut i = 0;

    for linedef in &mut mapmanager.map.linedef_vec {
        if linedef.front_sidedef >= 0 {
            let front_sidedef = &mapmanager.map.sidefef_vec[linedef.front_sidedef as usize];

            linedef.front = front_sidedef.clone();

            if linedef.back_sidedef >= 0 {
                let back_sidedef = &mapmanager.map.sidefef_vec[linedef.back_sidedef as usize];
                linedef.back = back_sidedef.clone();
                if (front_sidedef.sector != back_sidedef.sector) {
                    mapmanager.map.sector_vec[front_sidedef.sector as usize]
                        .linedefs
                        .push(i);
                    mapmanager.map.sector_vec[back_sidedef.sector as usize]
                        .linedefs
                        .push(i);
                }
            } else {
                mapmanager.map.sector_vec[front_sidedef.sector as usize]
                    .linedefs
                    .push(i);
            }

            let front_sec = &mapmanager.map.sector_vec[front_sidedef.sector as usize];

            let vert1 = &mapmanager.map.vert_vec[linedef.start_vert as usize];
            let vert2 = &mapmanager.map.vert_vec[linedef.end_vert as usize];

            linedef.start = vert1.clone();
            linedef.end = vert2.clone();

            if linedef.back_sidedef >= 0 {
                let back_sidedef = &mapmanager.map.sidefef_vec[linedef.back_sidedef as usize];

                let back_sec = &mapmanager.map.sector_vec[back_sidedef.sector as usize];

                if front_sec.ceil_height > back_sec.ceil_height {
                    MapManager::generateWall(
                        &mut vertices,
                        &mut indices,
                        &mut uvs,
                        &mut normals,
                        vert1.clone(),
                        vert2.clone(),
                        back_sec.ceil_height,
                        front_sec.ceil_height,
                        &mut indices_index,
                        false,
                    );
                }

                if front_sec.ceil_height < back_sec.ceil_height {
                    MapManager::generateWall(
                        &mut vertices,
                        &mut indices,
                        &mut uvs,
                        &mut normals,
                        vert1.clone(),
                        vert2.clone(),
                        front_sec.ceil_height,
                        back_sec.ceil_height,
                        &mut indices_index,
                        true,
                    );
                }

                if front_sec.floor_height < back_sec.floor_height {
                    MapManager::generateWall(
                        &mut vertices,
                        &mut indices,
                        &mut uvs,
                        &mut normals,
                        vert1.clone(),
                        vert2.clone(),
                        front_sec.floor_height,
                        back_sec.floor_height,
                        &mut indices_index,
                        false,
                    );
                }

                if front_sec.floor_height > back_sec.floor_height {
                    MapManager::generateWall(
                        &mut vertices,
                        &mut indices,
                        &mut uvs,
                        &mut normals,
                        vert1.clone(),
                        vert2.clone(),
                        back_sec.floor_height,
                        front_sec.floor_height,
                        &mut indices_index,
                        true,
                    );
                }
            } else {
                MapManager::generateWall(
                    &mut vertices,
                    &mut indices,
                    &mut uvs,
                    &mut normals,
                    vert1.clone(),
                    vert2.clone(),
                    front_sec.floor_height,
                    front_sec.ceil_height,
                    &mut indices_index,
                    false,
                );
            }
        }
        i += 1;
    }

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(mapmanager.getTexture(images, "VILEJ5".to_string())),
        ..Default::default()
    });

    for sector in &mapmanager.map.sector_vec {
        println!("\nNew Vector {} ---------------------", sector.light_level);

        let shapes = mapmanager.detect_shapes(sector);

        println!("Detected shapes: {}", shapes.len());

        let mut biggest_area = std::f32::MIN;
        let mut biggest_aabb_index = 0;

        let mut j = 0;

        let mut holes: Vec<Vec<f64>>;
        holes = Vec::new();

        for shape in shapes.clone() {
            let mut min_x = std::f32::MAX;
            let mut max_x = std::f32::MIN;
            let mut min_y = std::f32::MAX;
            let mut max_y = std::f32::MIN;

            let mut shape_vec = mapmanager.get_linedef_vector_as_vertices(&shape);

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

            j += 1;
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
                let mut actual_mesh = mesh_floor.unwrap();
                let cacca = commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(actual_mesh),
                        material: material.clone(),
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
            }
            None => println!("Merda culo"),
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    mesh.set_indices(Some(mesh::Indices::U32(indices)));

    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: material.clone(),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 100000.0,
            color: Color::WHITE,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

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

fn camera_rotation_system(
    mut query: Query<&mut Transform, With<Camera3d>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
) {
    // for event in mouse_motion_events.iter() {
    //     let sensitivity = 0.002;

    //     for mut transform in query.iter_mut() {
    //         let yaw = -event.delta.x * sensitivity;
    //         let pitch = -event.delta.y * sensitivity;

    //         let rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch);
    //         transform.rotation.x += rotation.x;
    //         transform.rotation.y += rotation.y;
    //         // transform.rotation.z += rotation.z;
    //     }
    // }
}
