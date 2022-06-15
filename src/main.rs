use bevy::{prelude::*, render::{texture::BevyDefault}};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
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
    println!("Handle just created: {:?}", id);
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

    for i in img.size()[0] as i32/2-100..img.size()[0] as i32/2+100 {
        for j in img.size()[1] as i32/2-100..img.size()[1] as i32/2+100 {
            let pix = get_pixel(i, j, img);
            pix[0] = 0; pix[1] = 0; pix[2] = 0; pix[3] = 255;
        }
    }

}

fn get_pixel<'a>(x: i32, y: i32, img: &'a mut Image) -> &'a mut [u8] {
    if x > img.size()[0] as i32 || y > img.size()[1] as i32 { panic!("Referenced pixel outside image.")}
    let global_ind = (x + y * img.size()[0] as i32) * 4;
    return &mut img.data[global_ind as usize..(global_ind+4) as usize]
}