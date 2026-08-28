use bevy::prelude::*;
use bevy::camera::RenderTarget;
use bevy::mesh::{Indices, PrimitiveTopology};
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
    #[serde(default)] skew: f32,
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
struct SpriteMeta { name: String, tw: f32, th: f32, part_index: usize }

/// Per-(part, frame) mesh cache. Each rig frame maps to its own mesh because the
/// Godot basis (rotation + skew + scale) is baked directly into the vertices.
#[derive(Default, Resource)]
struct MeshCache {
    data: Vec<Vec<Option<Handle<Mesh>>>>,
}

impl MeshCache {
    fn get_or_insert(
        &mut self,
        part: usize,
        frame: usize,
        meshes: &mut Assets<Mesh>,
        a: f32, b: f32, c: f32, d: f32,
        tw: f32, th: f32,
    ) -> Handle<Mesh> {
        if self.data.len() <= part {
            self.data.resize(part + 1, Vec::new());
        }
        if self.data[part].len() <= frame {
            self.data[part].resize(frame + 1, None);
        }
        if let Some(h) = &self.data[part][frame] {
            return h.clone();
        }
        let m = make_skewed_mesh(a, b, c, d, tw, th);
        let h = meshes.add(m);
        self.data[part][frame] = Some(h.clone());
        h
    }
}

/// Build a quad whose vertices encode the full Godot `Transform2D` basis
/// (including skew), so Bevy renders a sheared parallelogram instead of an
/// axis-aligned rectangle. Coordinates are centered at the origin; the entity
/// `Transform` carries the world translation.
///
/// Godot basis (y-down canvas):
///   x' = a*(u-hw) + b*(v-hh) + cx
///   y' = c*(u-hw) + d*(v-hh) + cy
/// with a=cos(r)*sx, b=-sin(r+skew)*sy, c=sin(r)*sx, d=cos(r+skew)*sy.
///
/// Bevy world (y-up, so y is flipped):
///   bevy_x = a*lx + b*ly + (cx - 100)
///   bevy_y = -(c*lx + d*ly) + (100 - cy)
/// where lx = (u - hw) = qx*tw, ly = (v - hh) = qy*th, qx,qy in {-0.5, 0.5}.
fn make_skewed_mesh(a: f32, b: f32, c: f32, d: f32, tw: f32, th: f32) -> Mesh {
    let hw = tw * 0.5;
    let hh = th * 0.5;
    let positions = vec![
        // qx=-0.5, qy=-0.5  (texture top-left,    uv 0,0)
        [(-a * hw - b * hh), (c * hw + d * hh), 0.0],
        // qx= 0.5, qy=-0.5  (texture top-right,   uv 1,0)
        [( a * hw - b * hh), (-c * hw + d * hh), 0.0],
        // qx= 0.5, qy= 0.5  (texture bottom-right, uv 1,1)
        [( a * hw + b * hh), (-c * hw - d * hh), 0.0],
        // qx=-0.5, qy= 0.5  (texture bottom-left,  uv 0,1)
        [(-a * hw + b * hh), (c * hw - d * hh), 0.0],
    ];
    let uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let normals = vec![[0.0, 0.0, 1.0]; 4];
    let indices = Indices::U32(vec![0, 1, 2, 0, 2, 3]);
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, Default::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(indices);
    mesh
}

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
    app.insert_resource(MeshCache::default());
    app.add_systems(Startup, setup)
       .add_systems(Update, update_sprites)
       .add_systems(Update, capture_loop);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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
    // Spawn meshes for first rig directly (Startup: flushed before first Update)
    let first_rig = &rigs.0[0];
    for (part_index, part) in first_rig.parts.iter().enumerate() {
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
        let image_handle = images.add(img);
        let material = materials.add(ColorMaterial {
            texture: Some(image_handle),
            ..Default::default()
        });
        let sx = part.scale_x.or(part.scale).unwrap_or(1.0);
        let sy = part.scale_y.or(part.scale).unwrap_or(1.0);
        let rot = part.rotation.unwrap_or(0.0);
        let skew = part.skew;
        let cos_r = rot.cos();
        let sin_r = rot.sin();
        let cos_r_sk = (rot + skew).cos();
        let sin_r_sk = (rot + skew).sin();
        let a = cos_r * sx;
        let b = -sin_r_sk * sy;
        let c = sin_r * sx;
        let d = cos_r_sk * sy;
        let mesh_handle = meshes.add(make_skewed_mesh(a, b, c, d, tw, th));
        let bevy_x = part.x - 100.0;
        let bevy_y = 100.0 - part.y;
        let bevy_z = part.z + 1.0;
        commands.spawn((
            Mesh2d(mesh_handle),
            MeshMaterial2d(material),
            Transform::from_translation(Vec3::new(bevy_x, bevy_y, bevy_z)),
            SpriteMeta { name: part.image.clone(), tw, th, part_index },
        ));
    }
}

fn update_sprites(
    mut sprites: Query<(&mut Transform, &mut Mesh2d, &SpriteMeta)>,
    rigs: Res<Rigs>,
    frame: Res<CurrentFrame>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cache: ResMut<MeshCache>,
) {
    let frame_idx = (frame.0 as usize) % rigs.0.len();
    let frame_rig = &rigs.0[frame_idx];
    for (mut tr, mut mesh2d, meta) in sprites.iter_mut() {
        if let Some(part) = frame_rig.parts.iter().find(|p| p.image == meta.name) {
            let sx = part.scale_x.or(part.scale).unwrap_or(1.0);
            let sy = part.scale_y.or(part.scale).unwrap_or(1.0);
            let rot = part.rotation.unwrap_or(0.0);
            let skew = part.skew;
            let cos_r = rot.cos();
            let sin_r = rot.sin();
            let cos_r_sk = (rot + skew).cos();
            let sin_r_sk = (rot + skew).sin();
            let a = cos_r * sx;
            let b = -sin_r_sk * sy;
            let c = sin_r * sx;
            let d = cos_r_sk * sy;
            let handle = cache.get_or_insert(
                meta.part_index, frame_idx, &mut meshes, a, b, c, d, meta.tw, meta.th,
            );
            mesh2d.0 = handle;
            tr.translation = Vec3::new(part.x - 100.0, 100.0 - part.y, part.z + 1.0);
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
