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
    pub sun_font: Handle<Font>,
    pub peashooter_frames: Vec<Handle<Image>>,
    pub sunflower_frames: Vec<Handle<Image>>,
    pub normal_zombie_frames: Vec<Handle<Image>>,
    pub sun_frames: Vec<Handle<Image>>,
    pub pea_normal: Handle<Image>,
    pub pea_normal_explode: Handle<Image>,
    pub card_peashooter: Handle<Image>,
    pub card_sunflower: Handle<Image>,
    pub card_cherrybomb: Handle<Image>,
    pub card_wallnut: Handle<Image>,
    pub card_potatomine: Handle<Image>,
    pub card_snowpea: Handle<Image>,
    pub card_chomper: Handle<Image>,
    pub card_repeaterpea: Handle<Image>,
    pub card_puffshroom: Handle<Image>,
    pub card_sunshroom: Handle<Image>,
    pub card_fumeshroom: Handle<Image>,
    pub card_gravebuster: Handle<Image>,
    pub card_hypnoshroom: Handle<Image>,
    pub card_iceshroom: Handle<Image>,
    pub card_doomshroom: Handle<Image>,
    pub card_lilypad: Handle<Image>,
    pub card_squash: Handle<Image>,
    pub card_threepeashooter: Handle<Image>,
    pub card_tangleklep: Handle<Image>,
    pub card_jalapeno: Handle<Image>,
    pub card_spikeweed: Handle<Image>,
    pub card_torchwood: Handle<Image>,
    pub card_tallnut: Handle<Image>,
    pub card_seashroom: Handle<Image>,
    pub card_starfruit: Handle<Image>,
    pub card_pumpkinhead: Handle<Image>,
    pub card_garlic: Handle<Image>,
    pub card_giantwallnut: Handle<Image>,
    pub shoot_sound: Handle<AudioSource>,
    pub bullet_explode_sound: Handle<AudioSource>,
    pub cannot_choose_sound: Handle<AudioSource>,
    pub seed_packet_silhouette: Handle<Image>,
    pub seed_chooser_background: Handle<Image>,
    pub seed_packet_larger: Handle<Image>,
    pub start_button_glow: Handle<Image>,
    pub background: Handle<Image>,
    pub seed_bank: Handle<Image>,
    pub pause_return_button: Handle<Image>,
    pub small_button_bg: Handle<Image>,
    pub slider_slot: Handle<Image>,
    pub slider_knob: Handle<Image>,
    pub checkbox_off: Handle<Image>,
    pub checkbox_on: Handle<Image>,
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
    pub gravebutton_sound: Handle<AudioSource>,
    pub pause_sound: Handle<AudioSource>,
    pub seed_chooser_button: Handle<Image>,
    pub seed_chooser_button_glow: Handle<Image>,
    pub seed_chooser_button2: Handle<Image>,
    pub seed_chooser_button2_glow: Handle<Image>,
    pub almanac_index_back: Handle<Image>,
    pub almanac_plant_back: Handle<Image>,
    pub almanac_close_button: Handle<Image>,
    pub almanac_close_button_highlight: Handle<Image>,
    pub almanac_index_button: Handle<Image>,
    pub almanac_index_button_highlight: Handle<Image>,
    pub almanac_plant_card: Handle<Image>,
    pub almanac_ground_day: Handle<Image>,
    pub almanac_ground_night: Handle<Image>,
    pub almanac_ground_pool: Handle<Image>,
    pub almanac_ground_fog: Handle<Image>,
    pub almanac_ground_ice: Handle<Image>,
    pub almanac_ground_roof: Handle<Image>,
    pub almanac_close_button_mask: Handle<Image>,
    pub almanac_index_button_mask: Handle<Image>,
}

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        let server = app.world().resource::<AssetServer>().clone();
        let assets = GameAssets {
            font: server.load("fonts/汉仪夏日体W.ttf"),
            sun_font: server.load("fonts/汉仪夏日体W.ttf"),
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
            card_cherrybomb: server.load("graphics/Cards/card_cherrybomb.png"),
            card_wallnut: server.load("graphics/Cards/card_wallnut.png"),
            card_potatomine: server.load("graphics/Cards/card_potatomine.png"),
            card_snowpea: server.load("graphics/Cards/card_snowpea.png"),
            card_chomper: server.load("graphics/Cards/card_chomper.png"),
            card_repeaterpea: server.load("graphics/Cards/card_repeaterpea.png"),
            card_puffshroom: server.load("graphics/Cards/card_puffshroom.png"),
            card_sunshroom: server.load("graphics/Cards/card_sunshroom.png"),
            card_fumeshroom: server.load("graphics/Cards/card_fumeshroom.png"),
            card_gravebuster: server.load("graphics/Cards/card_gravebuster.png"),
            card_hypnoshroom: server.load("graphics/Cards/card_hypnoshroom.png"),
            card_iceshroom: server.load("graphics/Cards/card_iceshroom.png"),
            card_doomshroom: server.load("graphics/Cards/card_doomshroom.png"),
            card_lilypad: server.load("graphics/Cards/card_lilypad.png"),
            card_squash: server.load("graphics/Cards/card_squash.png"),
            card_threepeashooter: server.load("graphics/Cards/card_threepeashooter.png"),
            card_tangleklep: server.load("graphics/Cards/card_tangleklep.png"),
            card_jalapeno: server.load("graphics/Cards/card_jalapeno.png"),
            card_spikeweed: server.load("graphics/Cards/card_spikeweed.png"),
            card_torchwood: server.load("graphics/Cards/card_torchwood.png"),
            card_tallnut: server.load("graphics/Cards/card_tallnut.png"),
            card_seashroom: server.load("graphics/Cards/card_seashroom.png"),
            card_starfruit: server.load("graphics/Cards/card_starfruit.png"),
            card_pumpkinhead: server.load("graphics/Cards/card_pumpkinhead.png"),
            card_garlic: server.load("graphics/Cards/card_garlic.png"),
            card_giantwallnut: server.load("graphics/Cards/card_giantwallnut.png"),
            shoot_sound: server.load("sound/shoot.ogg"),
            bullet_explode_sound: server.load("sound/bulletExplode.ogg"),
            cannot_choose_sound: server.load("sound/cannotChooseWarning.ogg"),
            seed_packet_silhouette: server.load("image/ui/ui_card/SeedPacketSilhouette.png"),
            seed_chooser_background: server.load("image/ui/ui_card/SeedChooser_Background.png"),
            seed_packet_larger: server.load("image/ui/ui_card/SeedPacket_Larger.png"),
            start_button_glow: server.load("image/ui/ui_card/all_card/SeedChooser_Button_Glow.png"),
            background: server.load("graphics/Items/Background/Background_0.jpg"),
            seed_bank: server.load("image/ui/ui_card/SeedBank.png"),
            pause_return_button: server.load("graphics/Screen/btn_dialog_back_2.png"),
            small_button_bg: server
                .load("image/ui/ui_main_game_menu/UI_BG/button_BG.png"),
            slider_slot: server.load("image/ui/ui_main_game_menu/options_sliderslot.png"),
            slider_knob: server.load("image/ui/ui_main_game_menu/options_sliderknob2.png"),
            checkbox_off: server.load("image/ui/ui_main_game_menu/options_checkbox0.png"),
            checkbox_on: server.load("image/ui/ui_main_game_menu/options_checkbox1.png"),
            lawn_mower: server.load("items/lawnmower/parts/LawnMower_body.png"),
            lawn_mower_sound: server.load("sound/lawnmower.ogg"),
            mower_body: server.load("items/lawnmower/parts/LawnMower_body.png"),
            mower_wheelpiece: server.load("items/lawnmower/parts/LawnMower_wheelpiece.png"),
            mower_wheel1: server.load("items/lawnmower/parts/LawnMower_wheel1.png"),
            mower_wheel2: server.load("items/lawnmower/parts/LawnMower_wheel2.png"),
            mower_wheelshine: server.load("items/lawnmower/parts/LawnMower_wheelshine.png"),
            mower_pull: server.load("items/lawnmower/parts/LawnMower_pull.png"),
            mower_engine: server.load("items/lawnmower/parts/LawnMower_engine.png"),
            mower_exhaust: server.load("items/lawnmower/parts/LawnMower_exhaust.png"),
            gravebutton_sound: server.load("sound/gravebutton.ogg"),
            pause_sound: server.load("sound/pause.ogg"),
            seed_chooser_button: server.load("image/ui/ui_card/all_card/SeedChooser_Button.png"),
            seed_chooser_button_glow: server.load("image/ui/ui_card/all_card/SeedChooser_Button_Glow.png"),
            seed_chooser_button2: server.load("image/ui/ui_card/SeedChooser_Button2.png"),
            seed_chooser_button2_glow: server.load("image/ui/ui_card/SeedChooser_Button2_Glow.png"),
            almanac_index_back: server.load("image/Almanac/Almanac_IndexBack.jpg"),
            almanac_plant_back: server.load("image/Almanac/Almanac_PlantBack_New.png"),
            almanac_close_button: server.load("image/Almanac/Almanac_CloseButton.png"),
            almanac_close_button_highlight: server.load("image/Almanac/Almanac_CloseButtonHighlight.png"),
            almanac_index_button: server.load("image/Almanac/Almanac_IndexButton.png"),
            almanac_index_button_highlight: server.load("image/Almanac/Almanac_IndexButtonHighlight.png"),
            almanac_plant_card: server.load("image/Almanac/Almanac_PlantCard.png"),
            almanac_ground_day: server.load("image/Almanac/Almanac_GroundDay.jpg"),
            almanac_ground_night: server.load("image/Almanac/Almanac_GroundNight.jpg"),
            almanac_ground_pool: server.load("image/Almanac/Almanac_GroundPool.jpg"),
            almanac_ground_fog: server.load("image/Almanac/Almanac_GroundNightPool.jpg"),
            almanac_ground_ice: server.load("image/Almanac/Almanac_GroundIce.jpg"),
            almanac_ground_roof: server.load("image/Almanac/Almanac_GroundRoof.jpg"),
            almanac_close_button_mask: server.load("image/Almanac/Almanac_CloseButton_Mask.png"),
            almanac_index_button_mask: server.load("image/Almanac/Almanac_IndexButton_Mask.png"),
        };
        app.insert_resource(assets);
    }
}
