use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;

use crate::{clear_entities, draw_screen, AppState};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Menu), setup)
        .add_systems(Update, hover_start_button.run_if(in_state(AppState::Menu)))
        .add_systems(Update, start.run_if(in_state(AppState::Menu)))
        .add_systems(OnExit(AppState::Menu), clear_entities::<AppState>);
}
fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    let font: Handle<Font> = Default::default();

    fn word(parent: &mut ChildSpawnerCommands, word: [char; 3], font: Handle<Font>) {
        fn letter(parent: &mut ChildSpawnerCommands, letter: char, font: Handle<Font>) {
            parent
                .spawn((Node {
                    width: Val::Px(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(letter.to_string()),
                        TextFont {
                            font,
                            font_size: 100.0,
                            ..default()
                        },
                        TextColor(Color::BLACK),
                    ));
                });
        }

        parent
            .spawn((Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Px(300.0),
                ..default()
            },))
            .with_children(|parent| {
                letter(parent, word[0], font.clone());
                letter(parent, word[1], font.clone());
                letter(parent, word[2], font.clone());
            });
    }

    draw_screen(&mut commands, AppState::Menu).with_children(|parent| {
        parent
            .spawn((Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },))
            .with_children(|parent| {
                word(parent, ['T', 'I', 'C'], font.clone());
                word(parent, ['T', 'A', 'C'], font.clone());
                word(parent, ['T', 'O', 'E'], font.clone());

                parent
                    .spawn((Node {
                        height: Val::Px(300.0),
                        margin: UiRect::top(Val::Px(50.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceEvenly,
                        ..default()
                    },))
                    .with_children(|parent| {
                        parent.spawn((Node {
                            height: Val::Px(20.0),
                            ..default()
                        },));

                        parent
                            .spawn((
                                Button,
                                Node {
                                    border: UiRect::all(Val::Px(2.0)),
                                    padding: UiRect::all(Val::Px(5.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                                AppState::Menu,
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new("Start Game"),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 60.0,
                                        ..default()
                                    },
                                    TextColor(Color::BLACK),
                                ));
                            });
                    });
            });
    });
}

fn hover_start_button(mut buttons: Query<(&Interaction, &mut BorderColor), With<Button>>) {
    for (interaction, mut color) in buttons.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                *color = Color::srgba(0.0, 0.0, 0.0, 0.5).into();
            }
            _ => {
                *color = Color::srgba(0.0, 0.0, 0.0, 0.0).into();
            }
        }
    }
}

fn start(
    mut query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    for interaction in &mut query {
        if let Interaction::Pressed = interaction {
            app_state.set(AppState::Game)
        }
    }
}
