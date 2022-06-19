use bevy::{prelude::*, render::{texture::BevyDefault}, diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin}, input::{mouse::{MouseWheel, MouseMotion}}};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(init)
        .add_system(handle_input)
        .run();
}

#[derive(Component)]
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
        depth: 50,
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

    let mut threads: Vec<std::thread::JoinHandle<()>> = Vec::new();
    for j in 0..img_size[1] as i32 {
        let d = surface.depth;
        let ptr_usize: usize = img.data.as_mut_ptr() as usize;
        threads.push(std::thread::spawn(move || unsafe {
            let mut ptr = ptr_usize as *mut u8;
            ptr = ptr.offset((j*img_size[0] as i32*4) as isize);
            for i in 0..img_size[0] as i32{
                let x_coord = x_min + i as f64 * x_step;
                let y_coord = y_min + j as f64 * y_step;
                let bptr = ptr; ptr = ptr.offset(1);
                let gptr = ptr; ptr = ptr.offset(1);
                let rptr = ptr; ptr = ptr.offset(1);
                let aptr = ptr; ptr = ptr.offset(1);
                (*bptr,*gptr,*rptr,*aptr) = get_color(x_coord, y_coord, d);
            }
        }));
    }
    for thread in threads {
        let _ = thread.join();
    }
}


fn handle_input(keys: Res<Input<KeyCode>>,click: Res<Input<MouseButton>>, motion: EventReader<MouseMotion>, scroll: EventReader<MouseWheel>, mut assets: ResMut<Assets<Image>>, mut query: Query<&mut MandelbrotRender>) {
    let mut surface: &mut MandelbrotRender = &mut query.get_single_mut().unwrap();
    let img = assets.get_mut(surface.image_handle).unwrap();

    let scrolled = handle_scroll(scroll, &mut surface);
    let mut dragged: bool = false;
    if click.pressed(MouseButton::Left) { dragged = handle_drag(motion, &mut surface, img.size()); }
    let increased = handle_keys(keys, surface);
    if dragged || scrolled || increased {
        draw_image(img, surface);
    }
}

fn handle_keys(keys: Res<Input<KeyCode>>, surface: &mut MandelbrotRender) -> bool {
    let mut ret = false;
    if keys.pressed(KeyCode::Up){
        surface.depth += 1;
        ret = true;
    }else if keys.pressed(KeyCode::Down) && surface.depth > 2{
        surface.depth -= 1;
        ret = true;
    }
    return ret;
}

//returns true if the surface was changed
fn handle_scroll(mut er: EventReader<MouseWheel>, mut surface: &mut MandelbrotRender) -> bool {
    let mut ret = false;
    use bevy::input::mouse::MouseScrollUnit;
    for e in er.iter() {
        match e.unit {
            MouseScrollUnit::Line => {
                ret = true;
                surface.height -= e.y as f64 * surface.height * 0.1;
                surface.width -= e.y as f64 * surface.width * 0.1;
            },
            MouseScrollUnit::Pixel => {
                panic!("Pixel scrolling not yet implemented");
            }
        }
    }
    return ret;
}
//returns true if the surface was changed
fn handle_drag(mut er: EventReader<MouseMotion>, mut surface: &mut MandelbrotRender, img_size: Vec2) -> bool {
    let mut ret = false;
    for e in er.iter() {
        ret = true;
        surface.center.0 -= e.delta[0] as f64 * (surface.width / img_size[0] as f64);
        surface.center.1 -= e.delta[1] as f64 * (surface.height / img_size[1] as f64);
    }
    return ret;
}

fn get_color(a: f64, b: f64, depth: u32) -> (u8, u8, u8, u8) {
    let x0 = a; let y0 = b;
    let mut x: f64 = 0.; let mut y: f64 = 0.;
    let mut depth = depth;
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