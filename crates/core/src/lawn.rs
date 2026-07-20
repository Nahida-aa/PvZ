use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::state::GameState;

pub const GRID_COLS: u32 = 9;
pub const GRID_ROWS: u32 = 5;
pub const CELL_WIDTH: f32 = 80.0;
pub const CELL_HEIGHT: f32 = 99.0;
pub const GRID_ORIGIN_X: f32 = 35.0;
/// 草坪网格在屏幕坐标系中的 Y 轴起点
pub const GRID_ORIGIN_Y: f32 = 80.0;

pub const WIN_W: f32 = 800.0;
pub const WIN_H: f32 = 600.0;

pub fn screen_to_world(sx: f32, sy: f32) -> Vec2 {
    Vec2::new(sx - WIN_W / 2.0, WIN_H / 2.0 - sy)
}

pub fn world_to_screen(pos: Vec2) -> Vec2 {
    Vec2::new(pos.x + WIN_W / 2.0, WIN_H / 2.0 - pos.y)
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPos {
    pub col: u32,
    pub row: u32,
}

impl GridPos {
    pub fn new(col: u32, row: u32) -> Self {
        Self { col, row }
    }

    pub fn world_pos(&self) -> Vec2 {
        let sx = GRID_ORIGIN_X + self.col as f32 * CELL_WIDTH + CELL_WIDTH / 2.0;
        let sy = GRID_ORIGIN_Y + self.row as f32 * CELL_HEIGHT + CELL_HEIGHT / 2.0;
        screen_to_world(sx, sy)
    }

    pub fn world_bottom(&self) -> Vec2 {
        let sx = GRID_ORIGIN_X + self.col as f32 * CELL_WIDTH + CELL_WIDTH / 2.0;
        let sy = GRID_ORIGIN_Y + self.row as f32 * CELL_HEIGHT + CELL_HEIGHT * 0.81;
        screen_to_world(sx, sy)
    }

    pub fn from_world(pos: Vec2) -> Option<Self> {
        let sp = world_to_screen(pos);
        if sp.y < GRID_ORIGIN_Y || sp.x < GRID_ORIGIN_X {
            return None;
        }
        let col = ((sp.x - GRID_ORIGIN_X) / CELL_WIDTH) as i32;
        let row = ((sp.y - GRID_ORIGIN_Y) / CELL_HEIGHT) as i32;
        if col >= 0 && col < GRID_COLS as i32 && row >= 0 && row < GRID_ROWS as i32 {
            Some(Self::new(col as u32, row as u32))
        } else {
            None
        }
    }
}

#[derive(Resource, Debug)]
pub struct LawnOccupancy {
    cells: [[bool; GRID_ROWS as usize]; GRID_COLS as usize],
}

impl Default for LawnOccupancy {
    fn default() -> Self {
        Self {
            cells: [[false; GRID_ROWS as usize]; GRID_COLS as usize],
        }
    }
}

impl LawnOccupancy {
    pub fn is_free(&self, pos: GridPos) -> bool {
        if pos.col >= GRID_COLS || pos.row >= GRID_ROWS {
            return false;
        }
        !self.cells[pos.col as usize][pos.row as usize]
    }

    pub fn occupy(&mut self, pos: GridPos) {
        if pos.col < GRID_COLS && pos.row < GRID_ROWS {
            self.cells[pos.col as usize][pos.row as usize] = true;
        }
    }

    pub fn free(&mut self, pos: GridPos) {
        if pos.col < GRID_COLS && pos.row < GRID_ROWS {
            self.cells[pos.col as usize][pos.row as usize] = false;
        }
    }
}

pub struct LawnPlugin;

impl Plugin for LawnPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LawnOccupancy::default())
            .add_systems(OnEnter(GameState::Playing), draw_background);
    }
}

fn draw_background(mut commands: Commands, assets: Res<GameAssets>) {
    const BG_IMG_W: f32 = 1400.0;
    const BG_IMG_H: f32 = 600.0;
    const BG_VP_X: f32 = 220.0;

    let ox = -WIN_W / 2.0 - (BG_VP_X - BG_IMG_W / 2.0);
    let oy = -WIN_H / 2.0 + BG_IMG_H / 2.0;
    commands.spawn((
        Sprite::from_image(assets.background.clone()),
        Transform::from_translation(Vec3::new(ox, oy, -10.0)),
        crate::state::GameplayEntity,
    ));

    let grid_w = GRID_COLS as f32 * CELL_WIDTH;
    let grid_h = GRID_ROWS as f32 * CELL_HEIGHT;
    let color = Color::srgba(1.0, 0.0, 0.0, 0.3);
    for col in 0..=GRID_COLS {
        let sx = GRID_ORIGIN_X + col as f32 * CELL_WIDTH;
        let center = screen_to_world(sx, GRID_ORIGIN_Y + grid_h / 2.0);
        commands.spawn((
            Sprite::from_color(color, Vec2::new(1.0, grid_h)),
            Transform::from_translation(center.extend(5.0)),
            crate::state::GameplayEntity,
        ));
    }
    for row in 0..=GRID_ROWS {
        let sy = GRID_ORIGIN_Y + row as f32 * CELL_HEIGHT;
        let center = screen_to_world(GRID_ORIGIN_X + grid_w / 2.0, sy);
        commands.spawn((
            Sprite::from_color(color, Vec2::new(grid_w, 1.0)),
            Transform::from_translation(center.extend(5.0)),
            crate::state::GameplayEntity,
        ));
    }
}
