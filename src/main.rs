use bevy::{prelude::*, render::{texture::BevyDefault}, diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin}};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(init)
        .add_system(edit_image)
        .run();
}

#[derive(Component)]
struct DrawSurface {
    pub handle: bevy::asset::HandleId
}

fn init(mut assets: ResMut<Assets<Image>>, mut commands: Commands, windows: Res<Windows>) {
    let window = windows.get_primary().unwrap();
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let handle = assets.add(create_blank_image(window.width() as u32, window.height() as u32));
    let id = handle.id;
    commands.spawn_bundle(SpriteBundle {
        texture: handle,
        ..default()
    }).insert(DrawSurface{ handle: id });
}

fn create_blank_image(width: u32, height: u32) -> Image {
    let e = bevy::render::render_resource::Extent3d {width: width, height: height, depth_or_array_layers: 1};
    let t = bevy::render::render_resource::TextureDimension::D2;
    let p = &[100, 150, 200, 255];
    let f = bevy::render::render_resource::TextureFormat::bevy_default();
    Image::new_fill(e, t, p, f)
}



fn edit_image(mut assets: ResMut<Assets<Image>>, query: Query<&DrawSurface>) {
    let mut id = bevy::asset::HandleId::default::<Image>();
    for d in query.iter() { id = d.handle; }
    let img = assets.get_mut(id).unwrap();
    let img_size = img.size();

    for i in 0..img_size[0] as i32 {
        for j in 0..img_size[1] as i32 {
            let x_coord = i as f32/img.size()[0]*3. - 2.3;
            let y_coord = j as f32/img.size()[1]*2. - 1.;
            let pix = get_pixel(i, j, img);
            (pix[0],pix[1],pix[2],pix[3]) = get_color(x_coord, y_coord);
        }
    }

}

fn get_pixel<'a>(x: i32, y: i32, img: &'a mut Image) -> &'a mut [u8] {
    if x > img.size()[0] as i32 || y > img.size()[1] as i32 { panic!("Referenced pixel outside image.")}
    let global_ind = (x + y * img.size()[0] as i32) * 4;
    return &mut img.data[global_ind as usize..(global_ind+4) as usize]
}

fn get_color(mut x: f32, mut y: f32) -> (u8, u8, u8, u8) {
    let (cx, cy) = (x, y);
    let (mut x2, mut y2) = (x*x, y*y);
    let mut depth = 255;
    while depth > 0 && x2 + y2 < 5. {
        x2 = x*x; y2 = y*y;
        y = 2. * x * y + cy;
        x = x2 - y2 + cx;
        depth -= 1;
    }

    (depth as u8, depth as u8, depth as u8, 255)
}