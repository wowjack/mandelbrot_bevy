use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(init)
        .add_system(move_all)
        .run();
}

#[derive(Component)]
struct Square;

fn init(windows: Res<Windows>, mut commands: Commands) {
    let window = windows.get_primary().unwrap();

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    for x in 0..window.width() as i32 {
        for y in 0..window.height() as i32 {
            if x % 10 == 0 && y % 10 == 0 {
                commands.spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(1.,1.,1.),
                        ..default()
                    },
                    transform: Transform {
                        scale: Vec3::new(8., 8., 8.),
                        translation: Vec3::new(x as f32 - window.width()/2., y as f32 - window.height()/2., 0.),
                        ..default()
                    },
                    ..default()
                }).insert(Square);
            }
        }
    }
}

fn move_all(mut query: Query<&mut Transform, With<Square>>) {
    for mut t in query.iter_mut() {
        t.translation.y += 1.;
    }
}