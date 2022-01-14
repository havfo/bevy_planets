use rand::Rng;
use std::env;

use bevy::{
	core::FixedTimestep,
	prelude::*,
	diagnostic::{
		FrameTimeDiagnosticsPlugin,
		LogDiagnosticsPlugin,
	}
};

use bevy_mod_raycast::{
	DefaultRaycastingPlugin,
	RayCastMesh,
	RayCastMethod,
	RayCastSource,
	RaycastSystem,
};

#[derive(Default)]
struct GameSettings {
	planets: i32,
}

#[derive(Component)]
struct Planetoid {
	speed: f64,
	orbit_radius: f32,
	time: f64,
}

#[derive(Component)]
struct Pickable;

const TIME_STEP: f32 = 1.0 / 60.0;

fn main() {
	let args: Vec<String> = env::args().collect();
	let planets: i32 = args[1].trim().parse()
		.expect("please give me correct string number!");

	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugin(LogDiagnosticsPlugin::default())
		.add_plugin(FrameTimeDiagnosticsPlugin::default())
		.add_plugin(DefaultRaycastingPlugin::<Pickable>::default())
		.insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
		.insert_resource(GameSettings { planets })
		.add_startup_system(setup)
		.add_system_to_stage(
			CoreStage::PreUpdate,
			update_raycast_with_cursor.before(RaycastSystem::BuildRays),
		)
		.add_system_set(
			SystemSet::new()
				.with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
				.with_system(planetoid_movement_system),
		)
		.add_system(pick_planetoid)
		.add_system(bevy::input::system::exit_on_esc_system)
		.run();
}

fn update_raycast_with_cursor(
	mut cursor: EventReader<CursorMoved>,
	mut query: Query<&mut RayCastSource<Pickable>>,
) {
	for mut pick_source in &mut query.iter_mut() {
		if let Some(cursor_latest) = cursor.iter().last() {
			pick_source.cast_method = RayCastMethod::Screenspace(cursor_latest.position);
		}
	}
}


fn setup(
	settings: Res<GameSettings>,
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let mut rng = rand::thread_rng();

	commands
		.spawn_bundle(PbrBundle {
			mesh: meshes.add(Mesh::from(shape::Plane {
				size: 100.0,
				..Default::default()
			})),
			material: materials.add(StandardMaterial {
				base_color: Color::BLACK,
				..Default::default()
			}),
			transform: Transform::from_xyz(0.0, 0.0, 0.0),
			..Default::default()
		})
		.insert(RayCastMesh::<Pickable>::default());

	// Planets
	for i in 0..settings.planets {
		commands
			.spawn_bundle(PbrBundle {
				mesh: meshes.add(Mesh::from(shape::UVSphere {
					radius: rng.gen_range(0.005..0.01),
					..Default::default()
				})),
				material: materials.add(StandardMaterial {
					base_color: Color::LIME_GREEN,
					..Default::default()
				}),
				transform: Transform::from_xyz(0.0, 0.0, 0.0),
				..Default::default()
			})
			.with_children(|parent| {
				let moons = rng.gen_range(1..4);
				// Moons
				for j in 0..moons {
					parent
						.spawn_bundle(PbrBundle {
							mesh: meshes.add(Mesh::from(shape::UVSphere {
								radius: rng.gen_range(0.003..0.006),
								..Default::default()
							})),
							material: materials.add(StandardMaterial {
								base_color: Color::WHITE,
								..Default::default()
							}),
							transform: Transform::from_xyz(0.0, 0.0, 0.0),
							..Default::default()
						})
						.insert(Planetoid {
							speed: rng.gen_range(1.0..2.0),
							orbit_radius: 0.05 + ((j as f32) / 35.0),
							time: rng.gen_range(0.0..10.0),
						});
				}
			})
			.insert(Planetoid {
				speed: rng.gen_range(0.1..0.5),
				orbit_radius: 0.15 + ((i as f32) / 6.0),
				time: rng.gen_range(0.0..10.0),
			});
	}

	// Sun
	commands
		.spawn_bundle(PointLightBundle {
			transform: Transform::from_xyz(0.0, 0.0, 0.0),
			point_light: PointLight {
				intensity: 50.0, // lumens
				color: Color::YELLOW,
				shadows_enabled: true,
				..Default::default()
			},
			..Default::default()
		})
		.with_children(|builder| {
			builder.spawn_bundle(PbrBundle {
				mesh: meshes.add(Mesh::from(shape::UVSphere {
					radius: 0.02,
					..Default::default()
				})),
				material: materials.add(StandardMaterial {
					base_color: Color::YELLOW,
					emissive: Color::YELLOW,
					..Default::default()
				}),
				..Default::default()
			});
		});

	// camera
	commands
		.spawn_bundle(PerspectiveCameraBundle {
			transform: Transform::from_xyz(0.0, 2.0, 0.0).looking_at(Vec3::ZERO, Vec3::X),
			..Default::default()
		})
		.insert(RayCastSource::<Pickable>::new());
}

fn pick_planetoid(
	planetoids_query: Query<(&Planetoid, &GlobalTransform, &Handle<StandardMaterial>)>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	to: Query<&RayCastSource<Pickable>>,
	mouse_event: Res<Input<MouseButton>>,
) {
	if let Ok(raycast_source) = to.get_single() {
		if let Some(top_intersection) = raycast_source.intersect_top() {
			let mut new_position = top_intersection.1.position();
			new_position.y = 0.0;

			if mouse_event.just_pressed(MouseButton::Left) {
				let mut shortest_distance = 100.0;

				// Lets just change color of the closest planetoid for now
				let mut closest_color_handle : Option<&Handle<StandardMaterial>> = None;

				for (_planetoid, transform, handle) in planetoids_query.iter() {
					let current_distance = transform.translation.distance(new_position);

					// Reset all the colors
					let color = &mut materials.get_mut(handle).unwrap().base_color;
					color.set_r(0.0);
					color.set_g(1.0);
					color.set_b(0.0);

					if current_distance < shortest_distance {
						shortest_distance = current_distance;

						closest_color_handle = Some(handle);
					}
				}

				println!("{} shortest_distance", shortest_distance);

				if closest_color_handle.is_some() {
					let color = &mut materials.get_mut(closest_color_handle.unwrap()).unwrap().base_color;
					color.set_r(1.0);
					color.set_g(1.0);
					color.set_b(1.0);
				}
			}
		}
	}
}

fn planetoid_movement_system(
	time: Res<Time>,
	mut planetoids_query: Query<(&Planetoid, &mut Transform)>,
) {
	for (planetoid, mut transform) in planetoids_query.iter_mut() {
		let angle = (planetoid.speed * (planetoid.time - time.seconds_since_startup())) as f32;
		let rotation = Vec3::new(angle.cos(), 0.0, angle.sin());

		transform.translation = rotation * planetoid.orbit_radius;
	}
}
