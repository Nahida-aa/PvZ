use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::config::{AppConfig, LevelDefinition};
use crate::state::GameState;

pub fn screen_to_world(sx: f32, sy: f32, app: &AppConfig) -> Vec2 {
    Vec2::new(sx - app.win_w() / 2.0, app.win_h() / 2.0 - sy)
}

pub fn world_to_screen(pos: Vec2, app: &AppConfig) -> Vec2 {
    Vec2::new(pos.x + app.win_w() / 2.0, app.win_h() / 2.0 - pos.y)
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

    pub fn world_pos(&self, level: &LevelDefinition, app: &AppConfig) -> Vec2 {
        let g = &level.grid;
        let sx = g.origin_x + self.col as f32 * g.cell_w + g.cell_w / 2.0;
        let sy = g.origin_y + self.row as f32 * g.cell_h + g.cell_h / 2.0;
        screen_to_world(sx, sy, app)
    }

    pub fn world_bottom(&self, level: &LevelDefinition, app: &AppConfig) -> Vec2 {
        let g = &level.grid;
        let sx = g.origin_x + self.col as f32 * g.cell_w + g.cell_w / 2.0;
        let sy = g.origin_y + self.row as f32 * g.cell_h + g.cell_h * 0.81;
        screen_to_world(sx, sy, app)
    }

    pub fn from_world(pos: Vec2, level: &LevelDefinition, app: &AppConfig) -> Option<Self> {
        let g = &level.grid;
        let sp = world_to_screen(pos, app);
        if sp.y < g.origin_y || sp.x < g.origin_x {
            return None;
        }
        let col = ((sp.x - g.origin_x) / g.cell_w) as i32;
        let row = ((sp.y - g.origin_y) / g.cell_h) as i32;
        if col >= 0 && col < g.cols as i32 && row >= 0 && row < g.rows as i32 {
            Some(Self::new(col as u32, row as u32))
        } else {
            None
        }
    }
}

#[derive(Resource, Debug)]
pub struct LawnOccupancy {
    cells: Vec<Vec<bool>>,
}

impl LawnOccupancy {
    pub fn from_level(level: &LevelDefinition) -> Self {
        let g = &level.grid;
        Self {
            cells: vec![vec![false; g.rows as usize]; g.cols as usize],
        }
    }

    pub fn is_free(&self, pos: GridPos) -> bool {
        if pos.col as usize >= self.cells.len() {
            return false;
        }
        let col = &self.cells[pos.col as usize];
        if pos.row as usize >= col.len() {
            return false;
        }
        !col[pos.row as usize]
    }

    pub fn occupy(&mut self, pos: GridPos) {
        if pos.col as usize >= self.cells.len() {
            return;
        }
        let col = &mut self.cells[pos.col as usize];
        if pos.row as usize >= col.len() {
            return;
        }
        col[pos.row as usize] = true;
    }

    pub fn free(&mut self, pos: GridPos) {
        if pos.col as usize >= self.cells.len() {
            return;
        }
        let col = &mut self.cells[pos.col as usize];
        if pos.row as usize >= col.len() {
            return;
        }
        col[pos.row as usize] = false;
    }
}

pub struct LawnPlugin;

impl Plugin for LawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            (setup_lawn_occupancy, draw_background),
        );
    }
}

/// 进入关卡时用当前关卡布局初始化草坪占用表。
fn setup_lawn_occupancy(mut commands: Commands, level: Res<LevelDefinition>) {
    commands.insert_resource(LawnOccupancy::from_level(&level));
}

fn draw_background(
    mut commands: Commands,
    assets: Res<GameAssets>,
    app: Res<AppConfig>,
    level: Res<LevelDefinition>,
) {
    let bg = &app.bg;

    // ox: 背景图左上角的世界坐标 X。
    // 公式把"背景图中心对齐到 (viewport_x, 0) 的屏幕点"换算成世界坐标。
    // viewport_x 变小 -> ox 变大 -> 背景整体右移。
    let ox = -app.win_w() / 2.0 - (bg.viewport_x - bg.img_w / 2.0);
    // oy: 背景图左上角的世界坐标 Y（背景高与窗口同高，故垂直居中）。
    let oy = -app.win_h() / 2.0 + bg.img_h / 2.0;
    commands.spawn((
        Sprite::from_image(assets.background.clone()),
        Transform::from_translation(Vec3::new(ox, oy, -10.0)),
        crate::state::GameplayEntity,
    ));

    let g = &level.grid;
    let grid_w = g.cols as f32 * g.cell_w;
    let grid_h = g.rows as f32 * g.cell_h;
    let color = Color::srgba(0.0, 1.0, 0.0, 0.3);
    for col in 0..=g.cols {
        let sx = g.origin_x + col as f32 * g.cell_w;
        let center = screen_to_world(sx, g.origin_y + grid_h / 2.0, &app);
        commands.spawn((
            Sprite::from_color(color, Vec2::new(1.0, grid_h)),
            Transform::from_translation(center.extend(5.0)),
            crate::state::GameplayEntity,
        ));
    }
    for row in 0..=g.rows {
        let sy = g.origin_y + row as f32 * g.cell_h;
        let center = screen_to_world(g.origin_x + grid_w / 2.0, sy, &app);
        commands.spawn((
            Sprite::from_color(color, Vec2::new(grid_w, 1.0)),
            Transform::from_translation(center.extend(5.0)),
            crate::state::GameplayEntity,
        ));
    }
}

