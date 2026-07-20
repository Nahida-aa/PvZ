use bevy::prelude::*;

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
                .map(|i| server.load(format!("graphics/Zombies/NormalZombie/Zombie/Zombie_{i}.png")))
                .collect(),
            sun_frames: (0..22)
                .map(|i| server.load(format!("graphics/Plants/Sun/Sun_{i}.png")))
                .collect(),
            pea_normal: server.load("graphics/Bullets/PeaNormal/PeaNormal_0.png"),
            pea_normal_explode: server.load("graphics/Bullets/PeaNormalExplode/PeaNormalExplode_0.png"),
            card_peashooter: server.load("graphics/Cards/card_peashooter.png"),
            card_sunflower: server.load("graphics/Cards/card_sunflower.png"),
            shoot_sound: server.load("sound/shoot.ogg"),
            bullet_explode_sound: server.load("sound/bulletExplode.ogg"),
            cannot_choose_sound: server.load("sound/cannotChooseWarning.ogg"),
            background: server.load("graphics/Items/Background/Background_0.jpg"),
            chooser_bg: server.load("graphics/Screen/ChooserBackground.png"),
        };
        app.insert_resource(assets);
    }
}
