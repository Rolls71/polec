use bevy::prelude::*;
use bevy::color::palettes::tailwind::*;
// use rand;
// use fastnoise_lite::*;
use ghx_proc_gen::{
    generator::{
        builder::GeneratorBuilder,
        model::{ModelCollection},
        rules::RulesBuilder,
        socket::{SocketCollection, SocketsCartesian2D},
    },
    ghx_grid::cartesian::{
        coordinates::{Cartesian2D},
        grid::CartesianGrid,
    },
};

const WORLD_WIDTH: usize = 100;
const WORLD_HEIGHT: usize = 100;
const CELL_SIZE: f32 = 10.;

// const COL_WATER: Color = Color::Srgba(SKY_950);
const COL_SAND: Color = Color::srgb_u8(255, 251, 147);
const COL_GRASSLANDS: Color = Color::Srgba(LIME_400);
// const COL_FOREST: Color = Color::Srgba(LIME_950);
// const COL_MOUNTAIN: Color = Color::Srgba(BLUE_50);

pub struct WorldPlugin;


impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}


/// Generate a web of roads using model synthesis.
///
/// Spawns entities directly with Commands.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut sockets = SocketCollection::new();
    let ground_mono = sockets.create();
    let (road_side, road_end) = (sockets.create(), sockets.create());

    sockets
        .add_connection(ground_mono, vec![ground_mono, road_side])
        .add_connection(road_end, vec![road_end]);

    let mut models = ModelCollection::<Cartesian2D>::new();

    let _ground = models.create(SocketsCartesian2D::Mono(ground_mono)).with_weight(100.);
    let _road_straight = models.create(SocketsCartesian2D::Simple {
        x_pos: road_end,
        x_neg: road_end,
        y_pos: road_side,
        y_neg: road_side,
    })
    .with_all_rotations()
    .with_weight(50.);
    let _road_corner = models.create(SocketsCartesian2D::Simple {
        x_pos: road_end,
        x_neg: road_side,
        y_pos: road_end,
        y_neg: road_side,
    })
    .with_all_rotations()
    .with_weight(5.);
    let _road_t = models.create(SocketsCartesian2D::Simple {
        x_pos: road_end,
        x_neg: road_end,
        y_pos: road_end,
        y_neg: road_side,
    })
    .with_all_rotations()
    .with_weight(5.);

    let _road_x = models.create(SocketsCartesian2D::Mono(road_end))
    .with_weight(2.);

    let rules = RulesBuilder::new_cartesian_2d(models, sockets).build().unwrap();

    let grid = CartesianGrid::new_cartesian_2d(WORLD_WIDTH as u32, WORLD_HEIGHT as u32, false, false);

    let mut generator = GeneratorBuilder::new()
    .with_rules(rules)
    .with_grid(grid)
    .build()
    .unwrap();

    let colours = [
        ColorMaterial::from_color(COL_GRASSLANDS),
        ColorMaterial::from_color(COL_SAND),
        ColorMaterial::from_color(COL_SAND),
        ColorMaterial::from_color(COL_SAND),
        ColorMaterial::from_color(COL_SAND),
    ];

    let (_gen_info, world_pattern) = generator.generate_grid().unwrap();
    for y in 0..WORLD_HEIGHT {
        for x in 0..WORLD_WIDTH {
            let tile = world_pattern.get_2d(x as u32, y as u32);
            let scale = Vec3::new(CELL_SIZE, CELL_SIZE, 1.);
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::default())),
                MeshMaterial2d(materials.add(colours[tile.model_index].clone())),
                Transform::from_translation(Vec3::new(x as f32, y as f32, 0.)*scale).with_scale(scale)
            ));
        }
    }
}
