use bevy::{color::palettes::tailwind, prelude::*};
use rand;


const CELL_WIDTH: i32 = 32;
const CELL_HEIGHT: i32 = 32;
const WORLD_WIDTH: i32 = 100;
const WORLD_HEIGHT: i32 = 100;

const COL_ON: Srgba = tailwind::GRAY_100;
const COL_OFF: Srgba = tailwind::GRAY_500;


#[derive(Component)]
struct Cell;


pub struct CellWorldPlugin;

impl Plugin for CellWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}



fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {

    for y in -WORLD_HEIGHT..WORLD_HEIGHT {
        for x in -WORLD_WIDTH..WORLD_WIDTH {
            let colour: Color = Color::from(if rand::random_bool(0.5) { COL_ON } else { COL_OFF });
            commands.spawn((
                Cell,
                Mesh2d(meshes.add(Rectangle::default())),
                MeshMaterial2d(materials.add(ColorMaterial::from(colour))),
                Transform::from_xyz((x*CELL_WIDTH) as f32, (y*CELL_HEIGHT) as f32, 0 as f32)
                    .with_scale(Vec3::splat((CELL_WIDTH-1) as f32)),
            ));
        }
    }
}


