use bevy::{input::system::exit_on_esc_system, prelude::*};

use bevy_inspector_egui::{
    Inspectable, InspectorPlugin, RegisterInspectable, WorldInspectorPlugin,
};
use bevy_prototype_lyon::plugin::ShapePlugin;
use rand::{thread_rng, Rng};

#[derive(Component, Reflect, Inspectable)]
struct Velocity(Vec2);

#[derive(Component)]
struct Food;

#[derive(Component, Reflect, Inspectable, Default)]
struct Controller {
    p: f32,
    i_acc: f32,
    i: f32,
    d: f32,
    prev_err: Option<f32>,
}

struct GameState {
    score: u32,
    food_pos: Option<Vec2>,
}

struct EatEvent;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(ShapePlugin)
        .add_system(exit_on_esc_system)
        .insert_resource(ClearColor(Color::rgb(1., 1., 1.)))
        .add_system(update_food)
        .add_startup_system(init_system)
        .add_startup_system(spawn_food)
        .add_system(update_motion)
        .insert_resource(GameState {
            score: 0,
            food_pos: None,
        })
        .add_system(controller.after("eat"))
        .add_system(update_score.after("eat"))
        .add_system(eat_food.label("eat"))
        .add_system(listen_eat.after("eat"))
        .register_inspectable::<Velocity>()
        .register_inspectable::<Controller>()
        .add_event::<EatEvent>()
        .run();
}

fn init_system(mut commands: Commands, asset_server: Res<AssetServer>, game_state: ResMut<GameState>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle(TextBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: Rect {
                bottom: Val::Px(5.0),
                right: Val::Px(15.0),
                ..Default::default()
            },
            ..Default::default()
        },
        text: Text::with_section(
            "Hello, world!",
            TextStyle {
                font: asset_server.load("fonts/ShareTechMono-Regular.ttf"),
                font_size: 40.0,
                color: Color::GRAY,
            },
            TextAlignment {
                horizontal: HorizontalAlign::Center,
                ..Default::default()
            },
        ),
        ..Default::default()
    });

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                //color: Color::BLACK,
                custom_size: Some(Vec2::splat(80.0)),
                ..Default::default()
            },
            texture: asset_server.load("images/russia.png"),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..Default::default()
        })
        .insert(Velocity(Vec2::splat(0.0)))
        .insert(Controller {
            p: 0.06,
            i: 1.0,
            i_acc: 0.0,
            d: 0.8,
            prev_err: None,
        });
}

fn update_motion(mut query: Query<(&mut Velocity, &mut Transform)>) {
    for (v, mut t) in query.iter_mut() {
        let v = v.0;
        t.translation.x += v.x;
        t.translation.y += v.y;
    }
}

fn spawn_food(mut commands: Commands, mut game_state: ResMut<GameState>, asset_server: Res<AssetServer>) {
    let mut rand = thread_rng();
    let (x, y) = (rand.gen_range(-300.0..300.0), rand.gen_range(-300.0..300.0));
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                //color: Color::RED,
                custom_size: Some(Vec2::splat(55.0)),
                ..Default::default()
            },
            texture: asset_server.load("images/ukraine.png"),
            transform: Transform::from_xyz(x, y, 0.0),
            ..Default::default()
        })
        .insert(Food);

    game_state.food_pos = Some(Vec2::new(x, y));
}

fn update_food(
    query: Query<&Transform, (With<Food>, Changed<Transform>)>,
    mut game_state: ResMut<GameState>,
) {
    if let Some(f) = query.iter().last() {
        game_state.food_pos = Some(Vec2::new(f.translation.x, f.translation.y));
    } else {
        //game_state.food_pos = None;
    }
}

fn controller(
    mut query: Query<(&mut Velocity, &mut Transform, &mut Controller)>,
    game_state: Res<GameState>,
    mut eaten_event: EventReader<EatEvent>,
) {
    let mut agent = query.iter_mut().last().unwrap();

    agent.0 .0.x *= 0.9;
    agent.0 .0.y *= 0.9;

    if let Some(_) = eaten_event.iter().last() {
        agent.2.prev_err = None;
    }

    if let Some(food) = game_state.food_pos {
        let distx = agent.1.translation.x - food.x;
        let disty = agent.1.translation.y - food.y;

        let dist = (distx.powi(2) + disty.powi(2)).sqrt();

        let dir = Vec2::new(distx, disty).normalize() * 0.05;

        let mut d_resp = 0.0;
        let p_resp = dist * agent.2.p;
        if agent.2.prev_err.is_some() {
            let delta = dist - agent.2.prev_err.unwrap();

            d_resp = delta * agent.2.d;
        }

        //agent.0 .0.x = (1.0 / distx) * -10.0;
        //agent.0 .0.y = (1.0 / disty) * -10.0;

        agent.2.prev_err = Some(dist);

        let resp = d_resp + p_resp;

        agent.0 .0.x -= dir.x * resp;
        agent.0 .0.y -= dir.y * resp;

        agent.0 .0.x = agent.0 .0.x.min(10.0);
        agent.0 .0.y = agent.0 .0.y.min(10.0);
        //agent.0 .0 = agent.0 .0.normalize();
    } else {
        let v = agent.0 .0;
    }
}

fn eat_food(
    query: Query<&Transform, With<Controller>>,
    mut game_state: ResMut<GameState>,
    mut writer: EventWriter<EatEvent>,
) {
    let player = query.iter().last().unwrap();

    let player = Vec2::new(player.translation.x, player.translation.y);

    if let Some(pos) = game_state.food_pos {
        if pos.distance(player) < 10.0 {
            game_state.score += 1;
            writer.send(EatEvent);
        }
    }
}

fn update_score(mut query: Query<&mut Text>, state: Res<GameState>) {
    for mut text in query.iter_mut() {
        text.sections[0].value = format!("Food eaten {:?}", state.score);
    }
}

fn listen_eat(mut reader: EventReader<EatEvent>, mut query: Query<&mut Transform, With<Food>>) {
    if let Some(_) = reader.iter().last() {
        let mut rand = thread_rng();
        let pos = &mut query.iter_mut().last().unwrap();
        pos.translation.x = rand.gen_range(-200.0..200.0);
        pos.translation.y = rand.gen_range(-200.0..200.0);
    }
}
