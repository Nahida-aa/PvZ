use bevy::prelude::*;

#[derive(Component)]
pub struct EncyclopediaRoot;

#[derive(Component)]
pub struct AlmanacIndexPage;

#[derive(Component)]
pub struct AlmanacPlantPage;

#[derive(Component)]
pub struct AlmanacPlantCard {
    pub card_name: String,
}

#[derive(Component)]
pub struct AlmanacPlantButton;

#[derive(Component)]
pub struct AlmanacZombieButton;

#[derive(Component)]
pub struct AlmanacReturnButton;

#[derive(Component)]
pub struct AlmanacCloseButton;

#[derive(Component)]
pub struct AlmanacCloseImage;

#[derive(Component)]
pub struct AlmanacDetailBg;

#[derive(Component)]
pub struct AlmanacDetailPreview;

#[derive(Component)]
pub struct AlmanacDetailText;

#[derive(Component)]
pub struct AlmanacNameText;

#[derive(Component)]
pub struct AlmanacDescText;

#[derive(Component)]
pub struct AlmanacParamsText;

#[derive(Component)]
pub struct AlmanacHintText;

#[derive(Component)]
pub struct AlmanacIntroText;

#[derive(Component)]
pub struct AlmanacCostText;

#[derive(Component)]
pub struct AlmanacCooltimeText;
