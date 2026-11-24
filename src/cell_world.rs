use bevy::prelude::*;
use bevy::color::palettes::tailwind::*;
use rand;
use fastnoise_lite::*;
use ghx_proc_gen::{
    generator::{
        builder::GeneratorBuilder,
        model::{ModelCollection,Model},
        rules::RulesBuilder,
        socket::{SocketCollection, SocketsCartesian2D},
    },
    ghx_grid::cartesian::{
        coordinates::{Cartesian2D, CartesianPosition},
        grid::CartesianGrid,
    },
};




const CELL_SCALE: i8 = 2; // CELL_SIZE = 2^CELL_SCALE
const CELL_OFFSET: Vec3 = Vec3::new(0.5, 0.5, 0.0);
const WORLD_WIDTH: usize = 100;
const WORLD_HEIGHT: usize = 100;

const COL_WATER: Color = Color::Srgba(SKY_950);
const COL_SAND: Color = Color::srgb_u8(255, 251, 147);
const COL_GRASSLANDS: Color = Color::Srgba(LIME_400);
const COL_FOREST: Color = Color::Srgba(LIME_950);
const COL_MOUNTAIN: Color = Color::Srgba(BLUE_50);

const WATER_LEVEL: f32 = 0.685;

// const COL_ON: Srgba = tailwind::GRAY_100;
// const COL_OFF: Srgba = tailwind::GRAY_500;


#[derive(Component)]
struct Cell;

#[derive(Component, Default, Clone, Copy, PartialEq)]
struct GridPos {
    pos: IVec2,
    scalar: Scalar, // defines zoom specificity, enabling limited fine tuning (Vec * 2^scale)
}

impl GridPos {
    fn true_pos(&self) -> Vec3 {
        Vec3::new(
            self.scalar.scalar() * self.pos.x as f32,
            self.scalar.scalar() * self.pos.y as f32,
            -self.scalar.val() as f32,
        )
    }

    fn to_transform(&self, offset: Vec3) -> Transform {
        let two: i32 = 2;
        Transform::from_translation(self.true_pos() + offset * self.scalar.scalar())
            .with_scale(Vec3::splat(two.pow(self.scalar.0 as u32) as f32))
    }

    // fn divide(&self) -> [GridPos; 4] {
    //     [
    //         GridPos { pos: self.pos * 2 + IVec2::new(0,0), scalar: self.scalar.decr() },
    //         GridPos { pos: self.pos * 2 + IVec2::new(1,0), scalar: self.scalar.decr() },
    //         GridPos { pos: self.pos * 2 + IVec2::new(0,1), scalar: self.scalar.decr() },
    //         GridPos { pos: self.pos * 2 + IVec2::new(1,1), scalar: self.scalar.decr() },
    //     ]
    // }
}


#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
struct Scalar(i8);

impl Scalar {
    fn scalar(&self) -> f32 {
        let two: f32 = 2.0;
        two.powi(self.0 as i32)
    }
    fn val(&self) -> i8 {
        self.0
    }
    // fn decr(&self) -> Scalar {
    //     Scalar(self.0 - 1)
    // }
}


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
    let rect = Rectangle::default();

    let seed:i32 = rand::random();
    let mut fnl_terrain = FastNoiseLite::new();
    fnl_terrain.seed = seed;
    fnl_terrain.noise_type = NoiseType::OpenSimplex2;
    fnl_terrain.frequency = 0.05;
    fnl_terrain.set_fractal_octaves(Some(4));
    fnl_terrain.set_fractal_lacunarity(Some(2.0));
    fnl_terrain.set_fractal_gain(Some(0.5));
    fnl_terrain.domain_warp_2d(seed as f32, seed as f32);

    let mut fnl_plates = FastNoiseLite::new();
    fnl_plates.seed = seed;
    fnl_plates.noise_type = NoiseType::Cellular;
    fnl_plates.cellular_distance_function = CellularDistanceFunction::EuclideanSq;
    fnl_plates.frequency = 0.015;
    fnl_terrain.set_fractal_octaves(Some(3));
    fnl_terrain.set_fractal_lacunarity(Some(7.0));
    fnl_terrain.set_fractal_gain(Some(0.15));
    fnl_terrain.domain_warp_2d(seed as f32, seed as f32);

    let mut heights: [[f32; WORLD_HEIGHT as usize]; WORLD_WIDTH as usize] = [[0.0; WORLD_HEIGHT as usize]; WORLD_WIDTH as usize];

    let mut sockets = SocketCollection::new();
    let (water, sand, grasslands, forest, mountain) = (sockets.create(), sockets.create(), sockets.create(), sockets.create(), sockets.create());

    sockets.add_connection(water, vec![water, sand, grasslands]);
    sockets.add_connection(sand, vec![sand, grasslands]);
    sockets.add_connection(grasslands, vec![grasslands, forest]);
    sockets.add_connection(forest, vec![forest, mountain]);
    sockets.add_connection(mountain, vec![mountain]);


    let mut models = ModelCollection::<Cartesian2D>::new();
    let m_water = models.create(SocketsCartesian2D::Mono(water)).with_weight(0.).clone();
    models.create(SocketsCartesian2D::Mono(sand)).with_weight(0.5);
    models.create(SocketsCartesian2D::Mono(grasslands));
    models.create(SocketsCartesian2D::Mono(forest));
    models.create(SocketsCartesian2D::Mono(mountain)).with_weight(0.5);

    let colours = [
        ColorMaterial::from_color(COL_WATER),
        ColorMaterial::from_color(COL_SAND),
        ColorMaterial::from_color(COL_GRASSLANDS),
        ColorMaterial::from_color(COL_FOREST),
        ColorMaterial::from_color(COL_MOUNTAIN),
    ];

    let rules = RulesBuilder::new_cartesian_2d(models, sockets).build().unwrap();

    let grid = CartesianGrid::new_cartesian_2d(WORLD_WIDTH as u32, WORLD_HEIGHT as u32, false, false);

    let mut starting_tiles: Vec<(CartesianPosition, Model<Cartesian2D>)> = vec![];
    for y in 0..WORLD_HEIGHT {
        for x in 0..WORLD_WIDTH {
            let terrain_height = (fnl_terrain.get_noise_2d(x as f32, y as f32)+1.0)/2.0;
            let plate_height = (fnl_plates.get_noise_2d(x as f32, y as f32)+1.0)/2.0;
            let t = 1.0;
            let p = 3.0;
            let bal = 0.1*(t+p);
            let height = terrain_height.powf(bal*(1.0/t)) * plate_height.powf(bal*(1.0/p));
            heights[x as usize][y as usize] = height;
            if height < WATER_LEVEL {
                starting_tiles.push((CartesianPosition::new_xy(x as u32,y as u32), m_water.clone()));
            }
        }
    }

    let mut generator = GeneratorBuilder::new()
    .with_rules(rules)
    .with_grid(grid)
    .with_initial_nodes(starting_tiles).unwrap()
    .build()
    .unwrap();

    let (_gen_info, world_pattern) = generator.generate_grid().unwrap();
    for y in 0..WORLD_HEIGHT {
        for x in 0..WORLD_WIDTH {
            let grid_pos = GridPos { pos: IVec2::new(x as i32, y as i32), scalar: Scalar(CELL_SCALE)};
            let tile = world_pattern.get_2d(x as u32, y as u32);
            commands.spawn((
                Cell,
                Mesh2d(meshes.add(rect)),
                MeshMaterial2d(materials.add(colours[tile.model_index].clone())),
                grid_pos,
                grid_pos.to_transform(CELL_OFFSET),
            ));
        }
    }
}


