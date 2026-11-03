use bevy::{color::palettes::tailwind, prelude::*};
use rand;
const CELL_SCALE: i8 = 5; // CELL_SIZE = 2^CELL_SCALE
const CELL_OFFSET: Vec3 = Vec3::new(0.5, 0.5, 0.0);
const WORLD_WIDTH: i32 = 100;
const WORLD_HEIGHT: i32 = 100;

const COL_ON: Srgba = tailwind::GRAY_100;
const COL_OFF: Srgba = tailwind::GRAY_500;


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

    fn divide(&self) -> [GridPos; 4] {
        [
            GridPos { pos: self.pos * 2 + IVec2::new(0,0), scalar: self.scalar.decr() },
            GridPos { pos: self.pos * 2 + IVec2::new(1,0), scalar: self.scalar.decr() },
            GridPos { pos: self.pos * 2 + IVec2::new(0,1), scalar: self.scalar.decr() },
            GridPos { pos: self.pos * 2 + IVec2::new(1,1), scalar: self.scalar.decr() },
        ]
    }
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
    fn decr(&self) -> Scalar {
        Scalar(self.0 - 1)
    }
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
    let col_on = ColorMaterial::from(Color::from(COL_ON));
    let col_off = ColorMaterial::from(Color::from(COL_OFF));
    for y in -WORLD_HEIGHT..WORLD_HEIGHT {
        for x in -WORLD_WIDTH..WORLD_WIDTH {
            let coin_flip = rand::random_bool(0.5);
            let colour = if coin_flip { col_on.clone() } else { col_off.clone() };
            let grid_pos = GridPos { pos: IVec2::new(x, y), scalar: Scalar(CELL_SCALE)};
            commands.spawn((
                Cell,
                Mesh2d(meshes.add(rect)),
                MeshMaterial2d(materials.add(colour)),
                grid_pos,
                grid_pos.to_transform(CELL_OFFSET),
            ));
            if coin_flip && rand::random_bool(0.5) {
                for pos in grid_pos.divide() {
                    commands.spawn((
                        Cell,
                        Mesh2d(meshes.add(rect)),
                        MeshMaterial2d(materials.add(
                            ColorMaterial::from_color(Color::from(
                                Srgba::rgba_u8(203, 213, 225, rand::random()))))),
                        pos,
                        pos.to_transform(),
                    ));
                }
            }
        }
    }
}

