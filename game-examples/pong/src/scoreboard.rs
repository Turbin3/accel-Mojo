use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Score {
    pub player: u32,
    pub computer: u32,
}

#[derive(Component)]
pub struct PlayerScore;

#[derive(Component)]
pub struct ComputerScore;

pub fn spawn_scoreboard(mut commands: Commands) {
    let container = Node {
        width: percent(100.0),
        height: percent(100.0),
        justify_content: JustifyContent::Center,
        ..default()
    };

    let header = Node {
        width: px(200.),
        height: px(100.),
        ..default()
    };

    let player_score = (
        PlayerScore,
        Text::new("0"),
        TextFont::from_font_size(72.0),
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            top: px(5.0),
            left: px(5.0),
            ..default()
        },
    );

    let computer_score = (
        ComputerScore,
        Text::new("0"),
        TextFont::from_font_size(72.0),
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            top: px(5.0),
            right: px(25.0),
            ..default()
        },
    );

    commands.spawn((
        container,
        children![(header, children![player_score, computer_score])],
    ));
}

pub fn update_scoreboard(
    mut player_score: Single<&mut Text, (With<PlayerScore>, Without<ComputerScore>)>,
    mut computer_score: Single<&mut Text, (With<ComputerScore>, Without<PlayerScore>)>,
    score: Res<Score>,
) {
    if score.is_changed() {
        player_score.0 = score.player.to_string();
        computer_score.0 = score.computer.to_string();
    }
}
