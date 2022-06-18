use bevy::{prelude::*, render::{texture::BevyDefault}, diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin}};
use rug::Float;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(init)
        //.add_system(edit_image)
        .run();
}

#[derive(Component, Clone)]
struct MandelbrotRender {
    pub image_handle: bevy::asset::HandleId,
    pub depth: u32,
    pub width: f64,
    pub height: f64,
    pub center: (f64, f64)
}

fn init(mut assets: ResMut<Assets<Image>>, mut commands: Commands, windows: Res<Windows>) {
    let window = windows.get_primary().unwrap();
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let mut image = create_blank_image(window.width() as u32, window.height() as u32);
    let mut surface = MandelbrotRender {
        image_handle: bevy::asset::HandleId::default::<Image>(),
        depth: 200,
        width: 2. * (window.width()/window.height()) as f64,
        height: 2.,
        center: (-0.5, 0.)
    };
    draw_image(&mut image, &surface);

    let handle = assets.add(image);
    let id = handle.id;
    surface.image_handle = id;
    commands.spawn_bundle(SpriteBundle {
        texture: handle,
        ..default()
    }).insert(surface);
}

fn create_blank_image(width: u32, height: u32) -> Image {
    let e = bevy::render::render_resource::Extent3d {width: width, height: height, depth_or_array_layers: 1};
    let t = bevy::render::render_resource::TextureDimension::D2;
    let p = &[100, 150, 200, 255];
    let f = bevy::render::render_resource::TextureFormat::bevy_default();
    Image::new_fill(e, t, p, f)
}

fn draw_image(img: &mut Image, surface: &MandelbrotRender) {
    let img_size = img.size();
    let x_min = &surface.center.0 - &surface.width / 2.;
    let y_min = &surface.center.1 - &surface.height / 2.;

    let x_step = &surface.width / img_size[0] as f64;
    let y_step = &surface.height / img_size[1] as f64;

    for i in 0..img_size[0] as i32 {
        for j in 0..img_size[1] as i32 {
            //println!("{i}, {j}");
            let x_coord = x_min + i as f64 * x_step;
            let y_coord = y_min + j as f64 * y_step;
            let pix = get_pixel(i, j, img);
            (pix[0],pix[1],pix[2],pix[3]) = get_color(x_coord, y_coord);
        }
    }
}

/*
fn edit_image(mut assets: ResMut<Assets<Image>>, query: Query<&MandelbrotRender>) {
    let mut id = bevy::asset::HandleId::default::<Image>();
    for d in query.iter() { id = d.image_handle; }
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
*/

fn get_pixel<'a>(x: i32, y: i32, img: &'a mut Image) -> &'a mut [u8] {
    if x > img.size()[0] as i32 || y > img.size()[1] as i32 { panic!("Referenced pixel outside image.")}
    let global_ind = (x + y * img.size()[0] as i32) * 4;
    return &mut img.data[global_ind as usize..(global_ind+4) as usize]
}

fn get_color(mut a: f64, mut b: f64) -> (u8, u8, u8, u8) {
    let x0 = a; let y0 = b;
    let mut x: f64 = 0.; let mut y: f64 = 0.;
    let mut depth = 300;
    while x*x + y*y <= 4. && depth > 0 {
        let xtmp = x*x - y*y + x0;
        y = 2.*x*y + y0;
        x = xtmp;
        depth -= 1;
    }
    

    match depth {
        0 => (0, 0, 0, 255),
        other => {
            let num: u32 = other * 256*256*256/300;
            ((num/(256*256)) as u8, ((num/256)%256) as u8, (num%256) as u8, 255)
        }
    }
}