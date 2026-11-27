use bevy::prelude::*;

mod world_gen;


#[derive(Component)]
struct MainCamera;


fn main() {
    App::new()
    .add_plugins((DefaultPlugins, world_gen::WorldPlugin))
    .add_systems(Startup, camera_setup)
    .add_systems(Update, camera_movement)
    .run();
}

fn camera_setup(
    mut commands: Commands,
) {
    commands.spawn((Camera2d, MainCamera, Transform::from_xyz(500., 500., 0.)));
}

fn camera_movement(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera: Single<&mut Transform, With<MainCamera>>,
   ) {
    let mut direction = Vec2::ZERO;
    if input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if input.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }

    if direction != Vec2::ZERO {
        let speed = 300.0; // pixels per second
        let delta = direction.normalize() * speed * time.delta_secs();
        camera.translation.x += delta.x;
        camera.translation.y += delta.y;
    }
}
