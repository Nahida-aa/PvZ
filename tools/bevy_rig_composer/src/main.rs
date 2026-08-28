use bevy::prelude::*;
use bevy::camera::RenderTarget;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_capture::{Capture, CaptureBundle, RenderTargetHeadless};
use bevy::winit::WinitPlugin;
use bevy::app::{RunMode, ScheduleRunnerPlugin};
use bevy::time::TimeUpdateStrategy;
use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "bevy_rig_composer")]
struct Cli {
    #[arg(short, long)] input: PathBuf,
    #[arg(short, long)] output: PathBuf,
    #[arg(short = 'W', long, default_value = "200")] width: u32,
    #[arg(short = 'H', long, default_value = "200")] height: u32,
    #[arg(long, default_value = "0,0,0,0")] bg: String,
    #[arg(short = 'r', long)] rig: Option<PathBuf>,
}

#[derive(Deserialize, Clone)]
struct RigConfig { parts: Vec<PartDef> }

#[derive(Deserialize, Clone)]
struct PartDef {
    image: String, x: f32, y: f32, z: f32,
    #[serde(default)] scale: Option<f32>,
    #[serde(default)] scale_x: Option<f32>,
    #[serde(default)] scale_y: Option<f32>,
    #[serde(default)] rotation: Option<f32>,
    #[serde(default = "default_visible")] visible: bool,
}
fn default_visible() -> bool { true }

fn load_png(path: &std::path::Path) -> Option<Image> {
    let bytes = std::fs::read(path).ok()?;
    let mut decoder = png::Decoder::new(std::io::Cursor::new(&bytes));
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().ok()?;
    let info = reader.info().clone();
    let w = info.width;
    let h = info.height;
    let mut pixels = vec![0u8; (w * h * 4) as usize];
    reader.next_frame(&mut pixels).ok()?;
    Some(Image::new(
        Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        Default::default(),
    ))
}

#[derive(Resource, Clone)]
struct Rigs(pub Vec<RigConfig>);

#[derive(Resource, Clone)]
struct PartsDir(pub PathBuf);

#[derive(Resource, Clone)]
struct CliOutput(pub PathBuf);

#[derive(Resource, Clone)]
struct CliWidth(pub u32);

#[derive(Resource, Clone)]
struct CliHeight(pub u32);

#[derive(Resource)]
struct TargetFrameCount(pub u32);

#[derive(Resource, Default)]
struct CurrentFrame(pub u32);

#[derive(Component, Clone)]
struct SpriteMeta { name: String, tw: f32, th: f32 }

fn main() {
    let cli = Cli::parse();
    let bg: Vec<u32> = cli.bg.split(',').map(|s| s.parse().unwrap_or(0)).collect();
    let background_color = Color::srgba(
        bg[0] as f32 / 255.0, bg[1] as f32 / 255.0, bg[2] as f32 / 255.0, bg[3] as f32 / 255.0,
    );
    let parts_dir = if cli.input.is_dir() { cli.input.clone() } else { cli.input.parent().unwrap().to_path_buf() };
    let rig_dir = cli.rig.clone().map_or(parts_dir.clone(), |p| p.parent().unwrap().to_path_buf());
    
    let mut rigs: Vec<RigConfig> = Vec::new();
    for i in 0..24 {
        let path = rig_dir.join(format!("_rig_{:03}.jsonc", i));
        if path.exists() {
            let json = std::fs::read_to_string(&path).expect("failed to read rig");
            rigs.push(serde_json::from_str(&json).expect("invalid rig"));
        }
    }
    if rigs.is_empty() {
        eprintln!("No rig files found in {}", rig_dir.display());
        std::process::exit(1);
    }
    std::fs::create_dir_all(&cli.output).expect("cannot create output dir");

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin { primary_window: None, exit_condition: bevy::window::ExitCondition::DontExit, ..default() })
            .disable::<WinitPlugin>()
            .set(bevy::render::RenderPlugin { synchronous_pipeline_compilation: true, ..default() }),
        ScheduleRunnerPlugin { run_mode: RunMode::Loop { wait: None } },
        bevy_capture::CapturePlugin,
    ));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0)));
    app.insert_resource(ClearColor(background_color));
    app.insert_resource(Rigs(rigs));
    app.insert_resource(PartsDir(parts_dir));
    app.insert_resource(CliOutput(cli.output.clone()));
    app.insert_resource(CliWidth(cli.width));
    app.insert_resource(CliHeight(cli.height));
    app.insert_resource(TargetFrameCount(24));
    app.insert_resource(CurrentFrame::default());
    app.add_systems(Startup, setup)
       .add_systems(Update, update_sprites)
       .add_systems(Update, capture_loop);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    cw: Res<CliWidth>,
    ch: Res<CliHeight>,
    rigs: Res<Rigs>,
    parts_dir: Res<PartsDir>,
) {
    commands.spawn((
        Camera2d,
        RenderTarget::target_headless(cw.0, ch.0, &mut images),
        CaptureBundle::default(),
    ));
    // Spawn sprites for first rig directly (Startup: flushed before first Update)
    let first_rig = &rigs.0[0];
    for part in &first_rig.parts {
        if !part.visible { continue; }
        let path = parts_dir.0.join("parts").join(&part.image);
        let path = if path.exists() {
            path
        } else {
            let alt = parts_dir.0.join(&part.image);
            if alt.exists() { alt } else { continue; }
        };
        let img = match load_png(&path) {
            Some(i) => i,
            None => continue,
        };
        let tw = img.width() as f32;
        let th = img.height() as f32;
        let handle = images.add(img);
        let sx = part.scale_x.or(part.scale).unwrap_or(1.0);
        let sy = part.scale_y.or(part.scale).unwrap_or(1.0);
        let rot = part.rotation.unwrap_or(0.0);
        let bevy_x = part.x - 100.0;
        let bevy_y = 100.0 - part.y;
        let bevy_z = part.z + 1.0;
        commands.spawn((
            Sprite { image: handle.clone(), custom_size: Some(Vec2::new(tw * sx, th * sy)), ..default() },
            Transform::from_translation(Vec3::new(bevy_x, bevy_y, bevy_z)).with_rotation(Quat::from_rotation_z(rot)),
            SpriteMeta { name: part.image.clone(), tw, th },
        ));
    }
}

fn update_sprites(
    mut sprites: Query<(&mut Transform, &mut Sprite, &SpriteMeta)>,
    rigs: Res<Rigs>,
    frame: Res<CurrentFrame>,
) {
    let frame_idx = (frame.0 as usize) % rigs.0.len();
    let frame_rig = &rigs.0[frame_idx];
    for (mut tr, mut sp, meta) in sprites.iter_mut() {
        if let Some(part) = frame_rig.parts.iter().find(|p| p.image == meta.name) {
            let sx = part.scale_x.or(part.scale).unwrap_or(1.0);
            let sy = part.scale_y.or(part.scale).unwrap_or(1.0);
            let rot = part.rotation.unwrap_or(0.0);
            let bevy_x = part.x - 100.0;
            let bevy_y = 100.0 - part.y;
            tr.translation = Vec3::new(bevy_x, bevy_y, part.z + 1.0);
            tr.rotation = Quat::from_rotation_z(rot);
            tr.scale = Vec3::splat(1.0);
            sp.custom_size = Some(Vec2::new(meta.tw * sx, meta.th * sy));
        }
    }
}

fn capture_loop(
    mut done: Local<bool>,
    mut warmed_up: Local<bool>,
    mut captures: Query<&mut Capture>,
    cli_output: Res<CliOutput>,
    target: Res<TargetFrameCount>,
    mut frame: ResMut<CurrentFrame>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if *done { return; }
    // Warm-up frame: GPU textures aren't uploaded until the first render,
    // so skip capturing frame 0 (render once to warm up the pipeline).
    if !*warmed_up {
        *warmed_up = true;
        return;
    }
    let mut capture = match captures.single_mut() {
        Ok(c) => c,
        Err(_) => return,
    };
    if !capture.is_capturing() {
        use bevy_capture::encoder::frames::FramesEncoder;
        capture.start((FramesEncoder::new(cli_output.0.clone()),));
    }
    frame.0 += 1;
    if frame.0 >= target.0 {
        println!("Captured {} frames to {:?}", frame.0, cli_output.0);
        *done = true;
        app_exit.write(AppExit::Success);
    }
}

