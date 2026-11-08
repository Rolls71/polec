use bevy::prelude::*;
use rand;
use fastnoise_lite::*;

const CELL_SCALE: i8 = 2; // CELL_SIZE = 2^CELL_SCALE
const CELL_OFFSET: Vec3 = Vec3::new(0.5, 0.5, 0.0);
const WORLD_WIDTH: i32 = 100;
const WORLD_HEIGHT: i32 = 100;

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
    let mut cols: Vec<ColorMaterial> = Vec::new();
    for i in 0..10 {
        cols.push(ColorMaterial::from_color(Color::srgb_u8((256*i / 10) as u8,(256*i / 10) as u8,(256*i / 10) as u8)));
    }

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



    for y in -WORLD_HEIGHT..WORLD_HEIGHT {
        for x in -WORLD_WIDTH..WORLD_WIDTH {
            let terrain_height = (fnl_terrain.get_noise_2d(x as f32, y as f32)+1.0)/2.0;
            let plate_height = (fnl_plates.get_noise_2d(x as f32, y as f32)+1.0)/2.0;
            let t = 1.0;
            let p = 3.0;
            let bal = 0.1*(t+p);
            let height = terrain_height.powf(bal*(1.0/t)) * plate_height.powf(bal*(1.0/p));
            let colour = cols.get(((height+1.0)*5.0) as usize %10).unwrap().clone();
            let grid_pos = GridPos { pos: IVec2::new(x, y), scalar: Scalar(CELL_SCALE)};
            commands.spawn((
                Cell,
                Mesh2d(meshes.add(rect)),
                MeshMaterial2d(materials.add(colour)),
                grid_pos,
                grid_pos.to_transform(CELL_OFFSET),
            ));
        }
    }

}


