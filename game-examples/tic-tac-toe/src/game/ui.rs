use bevy::prelude::*;

use super::grid::{Cell, Dimension, GameState, GRID_SPACING};
use super::state::StateInfo;

#[derive(Component)]
pub enum GameOverButton {
    PlayAgain,
    BackToMenu,
}

#[derive(Component)]
pub struct GameOverOverlay {}

pub fn start_game(mut commands: Commands, mut next_game_state: ResMut<NextState<GameState>>) {
    next_game_state.set(GameState::XTurn);

    fn cell<'a>(
        parent: &'a mut bevy::ecs::hierarchy::ChildSpawnerCommands,
        cell: Cell,
        border: UiRect,
    ) -> EntityCommands<'a> {
        parent.spawn((
            Node {
                display: Display::Grid,
                grid_row: GridPlacement::start((-cell.row().position() + 2) as i16),
                grid_column: GridPlacement::start((cell.column().position() + 2) as i16),
                justify_items: JustifyItems::Center,
                align_items: AlignItems::Center,
                border,
                ..default()
            },
            BorderColor::all(Color::BLACK),
            cell,
        ))
    }

    crate::draw_screen(&mut commands, crate::AppState::Game).with_children(|parent| {
        parent
            .spawn((Node {
                display: Display::Grid,
                grid_template_rows: vec![
                    GridTrack::flex(1.0),
                    GridTrack::flex(1.0),
                    GridTrack::flex(1.0),
                ],
                grid_template_columns: vec![
                    GridTrack::flex(1.0),
                    GridTrack::flex(1.0),
                    GridTrack::flex(1.0),
                ],
                width: Val::Px(3.0 * GRID_SPACING),
                height: Val::Px(3.0 * GRID_SPACING),
                ..default()
            },))
            .with_children(|parent| {
                const NONE: Val = Val::ZERO;
                const THIN: Val = Val::Px(6.0);

                // Add bottom borders to bottom row cells to ensure bottom edge of grid is visible
                cell(parent, Cell::TopLeft, UiRect::new(NONE, THIN, THIN, NONE));     // right, top (no left, no bottom)
                cell(parent, Cell::TopMiddle, UiRect::new(NONE, THIN, THIN, NONE));   // right, top (no bottom)
                cell(parent, Cell::TopRight, UiRect::new(NONE, NONE, THIN, NONE));    // top (no right, no bottom)

                cell(
                    parent,
                    Cell::MiddleLeft,
                    UiRect::new(NONE, THIN, NONE, NONE),    // right only (no left, no bottom)
                );
                cell(
                    parent,
                    Cell::MiddleMiddle,
                    UiRect::new(NONE, THIN, NONE, NONE),    // right only (no bottom)
                );
                cell(
                    parent,
                    Cell::MiddleRight,
                    UiRect::new(NONE, NONE, NONE, NONE),    // no borders
                );

                cell(
                    parent,
                    Cell::BottomLeft,
                    UiRect::new(NONE, THIN, NONE, THIN),
                );
                cell(
                    parent,
                    Cell::BottomMiddle,
                    UiRect::new(NONE, THIN, NONE, THIN),
                );
                cell(
                    parent,
                    Cell::BottomRight,
                    UiRect::new(NONE, NONE, NONE, THIN),
                );
            });
    });
}

pub fn game_over(mut commands: Commands, info: Res<StateInfo>, _asset_server: Res<AssetServer>) {
    let font: Handle<Font> = Default::default();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                top: Val::ZERO,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            GlobalZIndex(1),
            GameOverOverlay {},
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(61.8),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                ))
                .with_children(|parent| {
                    parent
                        .spawn((Node {
                            justify_content: JustifyContent::Center,
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },))
                        .with_children(|parent| {
                            fn spawn_text(
                                parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
                                text: impl Into<String>,
                                font: Handle<Font>,
                                text_color: Color,
                            ) {
                                parent.spawn((
                                    Text::new(text),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 75.0,
                                        ..default()
                                    },
                                    TextColor(text_color),
                                ));
                            }

                            match info.game.winner() {
                                None => {
                                    spawn_text(parent, "It's a tie!", font.clone(), Color::BLACK);
                                }
                                Some((winner, _)) => {
                                    spawn_text(
                                        parent,
                                        format!("{}", winner.to_string()),
                                        font.clone(),
                                        winner.color(),
                                    );
                                    spawn_text(parent, " wins!", font.clone(), Color::BLACK);
                                }
                            }
                        });

                    fn button(
                        parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
                        text: impl Into<String>,
                        color: Color,
                        marker: GameOverButton,
                        font: Handle<Font>,
                    ) {
                        parent
                            .spawn((
                                Button,
                                Node {
                                    justify_content: JustifyContent::Center,
                                    padding: UiRect::all(Val::Px(10.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                                marker,
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(text),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 60.0,
                                        ..default()
                                    },
                                    TextColor(color),
                                ));
                            });
                    }

                    button(
                        parent,
                        "play again",
                        Color::srgb(0.0, 0.0, 1.0),
                        GameOverButton::PlayAgain,
                        font.clone(),
                    );
                    button(
                        parent,
                        "back to menu",
                        Color::srgb(1.0, 0.0, 0.0),
                        GameOverButton::BackToMenu,
                        font.clone(),
                    );
                });
        });
}

pub fn game_over_buttons(
    buttons: Query<(&Interaction, &GameOverButton), (Changed<Interaction>, With<Button>)>,
    mut next_app_state: ResMut<NextState<crate::AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut info: ResMut<StateInfo>,
) {
    for (interaction, button) in buttons.iter() {
        if let Interaction::Pressed = interaction {
            match button {
                GameOverButton::PlayAgain => {
                    *info = StateInfo::default();
                    next_game_state.set(GameState::XTurn);
                }
                GameOverButton::BackToMenu => {
                    *info = StateInfo::default();
                    next_game_state.set(GameState::GameNotInProgress);
                    next_app_state.set(crate::AppState::Menu);
                }
            }
        }
    }
}
