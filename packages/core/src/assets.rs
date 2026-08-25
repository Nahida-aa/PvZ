use bevy::prelude::*;

/// 背景音乐实体标记。
///
/// 附加到由 `start_music` 创建的音乐实体上，供 `pause_menu.rs`
/// 中的 `pause_bgm` / `resume_bgm` 系统通过 `Query<&AudioSink, With<BgmMusic>>`
/// 查询并控制播放状态。
#[derive(Component)]
pub struct BgmMusic;

#[derive(Resource)]
pub struct GameAssets {
    pub font: Handle<Font>,
    pub peashooter_frames: Vec<Handle<Image>>,
    pub sunflower_frames: Vec<Handle<Image>>,
    pub normal_zombie_frames: Vec<Handle<Image>>,
    pub sun_frames: Vec<Handle<Image>>,
    pub pea_normal: Handle<Image>,
    pub pea_normal_explode: Handle<Image>,
    pub card_peashooter: Handle<Image>,
    pub card_sunflower: Handle<Image>,
    pub shoot_sound: Handle<AudioSource>,
    pub bullet_explode_sound: Handle<AudioSource>,
    pub cannot_choose_sound: Handle<AudioSource>,
    pub background: Handle<Image>,
    pub chooser_bg: Handle<Image>,
    pub pause_menu_bg: Handle<Image>,
    pub pause_return_button: Handle<Image>,
    pub small_button_bg: Handle<Image>,
    pub lawn_mower: Handle<Image>,
    pub lawn_mower_sound: Handle<AudioSource>,
    pub mower_body: Handle<Image>,       // 车身主体（割草机外壳）
    pub mower_wheelpiece: Handle<Image>, // 轮罩/挡泥板**（每侧前后轮各一片，共4片） | 27×26
    pub mower_wheel1: Handle<Image>,     // 前轮**（左侧前轮，大轮
    pub mower_wheel2: Handle<Image>,     //后轮**（两个后轮，小轮
    pub mower_wheelshine: Handle<Image>, //轮子高光**（每个轮子上的反光点，共4个
    pub mower_pull: Handle<Image>,       //牵引杆/启动绳手柄**（车头前方的小把手）
    pub mower_engine: Handle<Image>,     //引擎**（车身中部上方的发动机）
    pub mower_exhaust: Handle<Image>,    //排气管**（引擎旁的小排气口）
}

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        let server = app.world().resource::<AssetServer>().clone();
        let assets = GameAssets {
            font: server.load("DroidSansFallback.ttf"),
            peashooter_frames: (0..13)
                .map(|i| server.load(format!("graphics/Plants/Peashooter/Peashooter_{i}.png")))
                .collect(),
            sunflower_frames: (0..18)
                .map(|i| server.load(format!("graphics/Plants/SunFlower/SunFlower_{i}.png")))
                .collect(),
            normal_zombie_frames: (0..22)
                .map(|i| {
                    server.load(format!(
                        "graphics/Zombies/NormalZombie/Zombie/Zombie_{i}.png"
                    ))
                })
                .collect(),
            sun_frames: (0..22)
                .map(|i| server.load(format!("graphics/Plants/Sun/Sun_{i}.png")))
                .collect(),
            pea_normal: server.load("graphics/Bullets/PeaNormal/PeaNormal_0.png"),
            pea_normal_explode: server
                .load("graphics/Bullets/PeaNormalExplode/PeaNormalExplode_0.png"),
            card_peashooter: server.load("graphics/Cards/card_peashooter.png"),
            card_sunflower: server.load("graphics/Cards/card_sunflower.png"),
            shoot_sound: server.load("sound/shoot.ogg"),
            bullet_explode_sound: server.load("sound/bulletExplode.ogg"),
            cannot_choose_sound: server.load("sound/cannotChooseWarning.ogg"),
            background: server.load("graphics/Items/Background/Background_0.jpg"),
            chooser_bg: server.load("graphics/Screen/ChooserBackground.png"),
            pause_menu_bg: server.load("graphics/Screen/option_dialog.png"),
            pause_return_button: server.load("graphics/Screen/btn_dialog_back_2.png"),
            small_button_bg: server.load("graphics/Screen/button_BG.png"),
            lawn_mower: server.load("graphics/Items/LawnMower_body.png"),
            lawn_mower_sound: server.load("sound/lawnmower.ogg"),
            mower_body: server.load("graphics/Items/LawnMower_body.png"),
            mower_wheelpiece: server.load("graphics/Items/LawnMower_wheelpiece.png"),
            mower_wheel1: server.load("graphics/Items/LawnMower_wheel1.png"),
            mower_wheel2: server.load("graphics/Items/LawnMower_wheel2.png"),
            mower_wheelshine: server.load("graphics/Items/LawnMower_wheelshine.png"),
            mower_pull: server.load("graphics/Items/LawnMower_pull.png"),
            mower_engine: server.load("graphics/Items/LawnMower_engine.png"),
            mower_exhaust: server.load("graphics/Items/LawnMower_exhaust.png"),
        };
        app.insert_resource(assets);
    }
}
