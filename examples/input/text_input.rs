//! Simple text input support
//!
//! Return creates a new line, backspace removes the last character.
//! Clicking toggle IME (Input Method Editor) support, but the font used as limited support of characters.
//! You should change the provided font with another one to test other languages input.

use std::mem;

use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                toggle_ime,
                listen_ime_events,
                listen_keyboard_input_events,
                bubbling_text,
            ),
        )
        .run();
}

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    // The default font has a limited number of glyphs, so use the full version for
    // sections that will hold text input.
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands.spawn(
        TextBundle::from_sections([
            TextSection::from("Click to toggle IME. Press return to start a new line.\n\n"),
            TextSection::from("IME Enabled: "),
            TextSection::from("false\n"),
            TextSection::from("IME Active:  "),
            TextSection::from("false\n"),
            TextSection::from("IME Buffer:  "),
            TextSection {
                value: "\n".to_string(),
                style: TextStyle {
                    font: font.clone(),
                    ..default()
                },
            },
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
    );

    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "".to_string(),
            TextStyle {
                font,
                font_size: 100.0,
                ..default()
            },
        ),
        ..default()
    });
}

fn toggle_ime(
    input: Res<ButtonInput<MouseButton>>,
    mut windows: Query<&mut Window>,
    mut text: Query<&mut Text, With<Node>>,
) {
    if input.just_pressed(MouseButton::Left) {
        let mut window = windows.single_mut();

        window.ime_position = window.cursor_position().unwrap();
        window.ime_enabled = !window.ime_enabled;

        let mut text = text.single_mut();
        text.sections[2].value = format!("{}\n", window.ime_enabled);
    }
}

#[derive(Component)]
struct Bubble {
    timer: Timer,
}

fn bubbling_text(
    mut commands: Commands,
    mut bubbles: Query<(Entity, &mut Transform, &mut Bubble)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut bubble) in bubbles.iter_mut() {
        if bubble.timer.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
        }
        transform.translation.y += time.delta_seconds() * 100.0;
    }
}

fn listen_ime_events(
    mut events: EventReader<Ime>,
    mut status_text: Query<&mut Text, With<Node>>,
    mut edit_text: Query<&mut Text, (Without<Node>, Without<Bubble>)>,
) {
    for event in events.read() {
        match event {
            Ime::Preedit { value, cursor, .. } if !cursor.is_none() => {
                status_text.single_mut().sections[6].value = format!("{value}\n");
            }
            Ime::Preedit { cursor, .. } if cursor.is_none() => {
                status_text.single_mut().sections[6].value = "\n".to_string();
            }
            Ime::Commit { value, .. } => {
                edit_text.single_mut().sections[0].value.push_str(value);
            }
            Ime::Enabled { .. } => {
                status_text.single_mut().sections[4].value = "true\n".to_string();
            }
            Ime::Disabled { .. } => {
                status_text.single_mut().sections[4].value = "false\n".to_string();
            }
            _ => (),
        }
    }
}

fn listen_keyboard_input_events(
    mut commands: Commands,
    mut events: EventReader<KeyboardInput>,
    mut edit_text: Query<&mut Text, (Without<Node>, Without<Bubble>)>,
) {
    for event in events.read() {
        // Only trigger changes when the key is first pressed.
        if !event.state.is_pressed() {
            continue;
        }

        match &event.logical_key {
            Key::Enter => {
                let mut text = edit_text.single_mut();
                if text.sections[0].value.is_empty() {
                    continue;
                }
                let old_value = mem::take(&mut text.sections[0].value);

                commands.spawn((
                    Text2dBundle {
                        text: Text::from_section(old_value, text.sections[0].style.clone()),
                        ..default()
                    },
                    Bubble {
                        timer: Timer::from_seconds(5.0, TimerMode::Once),
                    },
                ));
            }
            Key::Space => {
                edit_text.single_mut().sections[0].value.push(' ');
            }
            Key::Backspace => {
                edit_text.single_mut().sections[0].value.pop();
            }
            Key::Character(character) => {
                edit_text.single_mut().sections[0].value.push_str(character);
            }
            _ => continue,
        }
    }
}
