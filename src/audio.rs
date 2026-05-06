use bevy::{audio::Volume, prelude::*};

pub fn setup_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    let source = asset_server.load("tone.wav");

    commands.spawn((
        AudioPlayer::new(source),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: Volume::Decibels(-30.0),
            paused: true,
            ..default()
        },
    ));
}

// TODO: play sounds out of the audio pattern buffer rather than a single tone file
