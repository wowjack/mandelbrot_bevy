use bevy::asset::RenderAssetUsages;
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::math::DVec2;
use bevy::{prelude::*};
use bevy::render::render_resource::*;
use bevy::window::{PrimaryWindow, WindowResized};

#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FpsOverlayPlugin::default()))
        .add_systems(Startup, init)
        .add_systems(Update, (window_resized, handle_keys, update_cursor_text, update_depth_text))
        .add_observer(handle_drag)
        .add_observer(handle_scroll)
        .add_observer(handle_redraw)
        .run();
}



#[derive(Component)]
struct MandelbrotRender {
    pub image_handle: Handle<Image>,
    pub depth: u32,
    pub scale: f64,
    pub center: DVec2
}

fn init(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2d);

    let size = Extent3d {
        width: 800,
        height: 600,
        depth_or_array_layers: 1,
    };

    let image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[255, 0, 0, 255], // RGBA
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );

    let handle = images.add(image);

    commands.spawn((
        Sprite::from_image(handle.clone()),
        Transform::from_scale(Vec3::splat(1.)),
        MandelbrotRender {
            image_handle: handle,
            depth: 10,
            scale: 0.01,
            center: DVec2::new(0., 0.)
        }
    ));

    commands.trigger(RedrawEvent);


    commands.spawn(Node {
        width: Val::Percent(100.),
        flex_direction: FlexDirection::RowReverse,
        column_gap: Val::Px(50.),
        padding: UiRect::axes(Val::Px(20.), Val::Px(5.)),
        ..default()
    }).with_children(|builder| {
        builder.spawn(CursorText);
        builder.spawn(DepthText);
    });
}


#[derive(Component)]
#[require(Node, TextFont = TextFont::from_font_size(20.), Text = Text::new("CURSOR POS: "))]
struct CursorText;

fn update_cursor_text(
    mut text_query: Query<&mut Text, With<CursorText>>,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    surface_query: Query<&MandelbrotRender>
) {
    let Ok((camera, gtrans)) = camera_query.single() else { return };

    for mut text in text_query.iter_mut() {
        let Ok(surface) = surface_query.single() else { return };
        let Ok(window) = window_query.single() else { return };
        let Some(cursor_pos) = window.cursor_position() else { return };
        let Ok(world_pos) = camera.viewport_to_world_2d(gtrans, cursor_pos) else { return };
        let image_pos = world_pos.as_dvec2() * surface.scale + surface.center * DVec2::new(1., -1.);
        text.0 = format!("CURSOR POS: {:.5} {:.5}", image_pos.x, image_pos.y);
    }
}

#[derive(Component)]
#[require(Node, TextFont = TextFont::from_font_size(20.), Text = Text::new("MAX ITERATIONS: "))]
struct DepthText;

fn update_depth_text(
    mut text_query: Query<&mut Text, With<DepthText>>,
    surface_query: Query<&MandelbrotRender>
) {
    for mut text in text_query.iter_mut() {
        let Ok(surface) = surface_query.single() else { return };
        text.0 = format!("MAX ITERATIONS: {}", surface.depth);
    }
}



fn window_resized(
    mut messages: MessageReader<WindowResized>,
    mut surface_query: Query<&mut MandelbrotRender>,
    mut assets: ResMut<Assets<Image>>,
    mut commands: Commands
) {
    for message in messages.read() {
        let Ok(surface) = surface_query.single_mut() else { continue };
        let Some(img) = assets.get_mut(surface.image_handle.id()) else { continue };

        img.resize(Extent3d { width: message.width as u32, height: message.height as u32, depth_or_array_layers: 1 } );
        commands.trigger(RedrawEvent);
    }
}

fn handle_drag(
    on: On<Pointer<Drag>>,
    mut surface_query: Query<&mut MandelbrotRender>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands
) {
    return;
    let Ok((camera, camera_gtrans)) = camera_query.single() else { return };
    let Ok(delta_pos) = camera.viewport_to_world_2d(camera_gtrans, on.delta) else { return };
    let Ok(zero_pos) = camera.viewport_to_world_2d(camera_gtrans, Vec2::ZERO) else { return };
    let delta = delta_pos - zero_pos;
    
    let Ok(mut surface) = surface_query.single_mut() else { return };
    let scale = surface.scale;
    surface.center.x -= delta.x as f64 * scale;
    surface.center.y += delta.y as f64 * scale;

    commands.trigger(RedrawEvent);
}

fn handle_scroll(
    on: On<Pointer<Scroll>>,
    mut surface_query: Query<&mut MandelbrotRender>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands
) {
    let Ok(window) = window_query.single() else { return };
    let Ok(mut surface) = surface_query.single_mut() else { return };
    let Ok((camera, camera_gtrans)) = camera_query.single() else { return };

    use bevy::input::mouse::MouseScrollUnit;
    match on.unit {
        MouseScrollUnit::Line => {
            let Some(mouse_pos) = window.cursor_position() else { return };

            let Ok(unscaled_cursor_pos) = camera.viewport_to_world_2d(camera_gtrans, mouse_pos) else { return };
            let old_pos = unscaled_cursor_pos.as_dvec2() * surface.scale;
            surface.scale *= 1. - (on.y as f64)*0.2;
            let new_pos = unscaled_cursor_pos.as_dvec2() * surface.scale;
            let delta = new_pos - old_pos;
            surface.center.x -= delta.x;
            surface.center.y += delta.y;

            commands.trigger(RedrawEvent);
        },
        MouseScrollUnit::Pixel => {
            let Some(mouse_pos) = window.cursor_position() else { return };
            let Ok(unscaled_cursor_pos) = camera.viewport_to_world_2d(camera_gtrans, mouse_pos) else { return };
            let old_pos = unscaled_cursor_pos.as_dvec2() * surface.scale;
            surface.scale *= 1. - (on.y as f64) * 0.002; // much smaller factor than 0.2
            let new_pos = unscaled_cursor_pos.as_dvec2() * surface.scale;
            let delta = new_pos - old_pos;
            surface.center.x -= delta.x;
            surface.center.y += delta.y;
            commands.trigger(RedrawEvent);
        }
    }
}

fn handle_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut surface_query: Query<&mut MandelbrotRender>,
    mut commands: Commands
) {
    let Ok(mut surface) = surface_query.single_mut() else { return };
    let mut redraw = false;

    if keys.pressed(KeyCode::ArrowUp) {
        surface.depth = (surface.depth+1).min(500);
        redraw = true;
    } else if keys.pressed(KeyCode::ArrowDown) {
        surface.depth = (surface.depth-1).max(1);
        redraw = true;
    }

    if keys.pressed(KeyCode::KeyW) {
        surface.center.y -= surface.scale;
        redraw = true;
    }
    if keys.pressed(KeyCode::KeyA) {
        surface.center.x -= surface.scale;
        redraw = true;
    }
    if keys.pressed(KeyCode::KeyS) {
        surface.center.y += surface.scale;
        redraw = true;
    }
    if keys.pressed(KeyCode::KeyD) {
        surface.center.x += surface.scale;
        redraw = true;
    }

    if redraw {
        commands.trigger(RedrawEvent);
    }
}




#[derive(Event)]
struct RedrawEvent;

fn handle_redraw(
    _: On<RedrawEvent>,
    query: Query<&MandelbrotRender>,
    mut assets: ResMut<Assets<Image>>
) {
    let mr = query.single().unwrap();
    let image = assets.get_mut(mr.image_handle.id()).unwrap();

    draw_image(image, mr);
}

fn draw_image(
    img: &mut Image,
    surface: &MandelbrotRender
) {
    let img_size = img.size().as_dvec2();
    let x_min = surface.center.x - surface.scale * img_size.x / 2.;
    let y_min = surface.center.y - surface.scale * img_size.y / 2.;

    let Some(data) = img.data.as_mut() else { return };
    
    // RGBA pixels
    #[cfg(not(target_arch = "wasm32"))]
    data.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
        let x = x_min + (i % img_size.x as usize) as f64*surface.scale;
        let y = y_min + (i / img_size.x as usize) as f64*surface.scale;

        let color = get_color(x, y, surface.depth);
        
        pixel.copy_from_slice(&[
            color.0,
            color.1,
            color.2,
            color.3
        ]);

    });

    #[cfg(target_arch = "wasm32")]
    data.chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
        let x = x_min + (i % img_size.x as usize) as f64*surface.scale;
        let y = y_min + (i / img_size.x as usize) as f64*surface.scale;

        let color = get_color(x, y, surface.depth);
        
        pixel.copy_from_slice(&[
            color.0,
            color.1,
            color.2,
            color.3
        ]);

    });

    

}

#[inline(always)]
fn get_color(cx: f64, cy: f64, max_depth: u32) -> (u8, u8, u8, u8) {
    let mut zx = 0.0;
    let mut zy = 0.0;

    let mut zx2 = 0.0;
    let mut zy2 = 0.0;

    let mut iter = max_depth;

    while zx2 + zy2 <= 4.0 && iter > 0 {
        zy = 2.0 * zx * zy + cy;
        zx = zx2 - zy2 + cx;

        zx2 = zx * zx;
        zy2 = zy * zy;

        iter -= 1;
    }
    
    return match iter {
        0 => (0, 0, 0, 255),
        other => {
            let num: u32 = other * 256*256*256/300;
            ((num/(256*256)) as u8, ((num/256)%256) as u8, (num%256) as u8, 255)
        }
    }
}