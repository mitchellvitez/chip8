use crate::instruction::RecentInstructions;
use crate::machine::Machine;
use crate::{constant::*, SimState};
use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

#[derive(Component)]
pub struct ErrorText;

#[derive(Component)]
pub struct Background;

#[derive(Component)]
pub struct RecentInstructionsMarker;

#[derive(Component)]
pub struct ExecutionModeMarker;

#[derive(Component)]
pub struct CyclesMarker;

#[derive(Component)]
pub struct RegistersMarker;

#[derive(Component)]
pub struct TimersMarker;

#[derive(Component)]
pub struct PseudoRegistersMarker;

#[derive(Component)]
pub struct StackMarker;

#[derive(Resource, Default)]
pub struct Display {
    pub handle: Handle<Image>,
}

#[derive(Resource, Default)]
pub struct RamVisualizer {
    pub handle: Handle<Image>,
}

pub fn update_ui(
    machine: Res<Machine>,
    mut images: ResMut<Assets<Image>>,
    display: Res<Display>,
    ram_visualizer: Res<RamVisualizer>,
    recent_instructions_queue: Res<RecentInstructions>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<RecentInstructionsMarker>>,
        Query<&mut Text, With<ExecutionModeMarker>>,
        Query<&mut Text, With<CyclesMarker>>,
        Query<&mut Text, With<RegistersMarker>>,
        Query<&mut Text, With<StackMarker>>,
        Query<&mut Text, With<TimersMarker>>,
        Query<&mut Text, With<PseudoRegistersMarker>>,
    )>,
    state: Res<State<SimState>>,
) {
    if !machine.is_changed() {
        return;
    }

    // update display from machine.display data
    let Some(display_image) = images.get_mut(&display.handle) else {
        return;
    };
    let Some(ref mut display_image_data) = display_image.data else {
        return;
    };
    for (i, pixel) in display_image_data.chunks_exact_mut(4).enumerate() {
        const BLACK: &[u8; 4] = &[0, 0, 0, 255];
        const WHITE: &[u8; 4] = &[255, 255, 255, 255];
        let pixel_color = if !machine.display[i] { BLACK } else { WHITE };
        pixel.copy_from_slice(pixel_color);
    }

    // update RAM visualizer from machine.memory data
    let Some(ram_visualizer_image) = images.get_mut(&ram_visualizer.handle) else {
        return;
    };
    let Some(ref mut ram_visualizer_image_data) = ram_visualizer_image.data else {
        return;
    };
    for (i, pixel) in ram_visualizer_image_data.chunks_exact_mut(4).enumerate() {
        let pixel_color: &[u8; 4] = if machine.memory[i] == 0 {
            &[0, 0, 0, 255]
        } else {
            // convert non-zero RAM bytes to a color for the RAM visualization
            let hue = (machine.memory[i] as f32 / 255.0) * 360.0;
            let color = Color::hsl(hue, 1.0, 0.5).to_srgba();
            let r = (color.red * 255.0) as u8;
            let g = (color.green * 255.0) as u8;
            let b = (color.blue * 255.0) as u8;
            &[r, g, b, 255]
        };
        pixel.copy_from_slice(pixel_color);
    }

    if let Ok(mut text) = text_queries.p0().single_mut() {
        let message = recent_instructions_queue
            .recent_instructions
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        **text = format!("{}", message);
    }

    if let Ok(mut text) = text_queries.p1().single_mut() {
        **text = match **state {
            SimState::Stepping => {
                format!("single step\n\npress [P] for real time\npress [SPACE] to step")
            }
            SimState::Executing => {
                format!("real time\n\npress [P] for single step\n")
            }
            SimState::Errored => {
                format!("encountered fatal error\n\n\n")
            }
            SimState::WaitingForKey => {
                format!("waiting for keyboard input\n\n\n")
            }
        };
    }

    if let Ok(mut text) = text_queries.p2().single_mut() {
        **text = format!("{}", machine.cycles);
    }

    if let Ok(mut text) = text_queries.p3().single_mut() {
        let mut s = String::new();
        for (i, &v) in machine.registers.iter().enumerate() {
            s.push_str(&format!("V{:X}=0x{:02X}\n", i, v));
        }
        s.push_str(&format!("\nI=0x{:03X}", machine.i));
        **text = s;
    }

    if let Ok(mut text) = text_queries.p4().single_mut() {
        let mut s = String::new();
        for v in machine.stack.iter().rev() {
            s.push_str(&format!("0x{:03X}\n", v));
        }
        s.push_str(&format!("\nSP=0x{:01X}", machine.sp));
        **text = s;
    }

    if let Ok(mut text) = text_queries.p5().single_mut() {
        **text = format!("DT=0x{:02X}\nST=0x{:02X}", machine.dt, machine.st);
    }

    if let Ok(mut text) = text_queries.p6().single_mut() {
        **text = format!("PC=0x{:03X}\n ", machine.pc);
    }
}

pub fn create_display_image(images: &mut Assets<Image>) -> Handle<Image> {
    let mut image = Image::new_fill(
        Extent3d {
            width: DISPLAY_WIDTH as u32,
            height: DISPLAY_HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    images.add(image)
}

pub fn create_ram_visualizer_image(images: &mut Assets<Image>) -> Handle<Image> {
    let mut image = Image::new_fill(
        Extent3d {
            width: 128,
            height: RAM_SIZE as u32 / 128,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    images.add(image)
}

pub fn setup_ui(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let display_handle = create_display_image(&mut images);
    commands.insert_resource(Display {
        handle: display_handle.clone(),
    });

    let ram_visualizer_handle = create_ram_visualizer_image(&mut images);
    commands.insert_resource(RamVisualizer {
        handle: ram_visualizer_handle.clone(),
    });

    commands.spawn(Camera2d);
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            Background,
        ))
        .with_children(|parent| {
            // top bar
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                ))
                .with_children(|top_bar| {
                    // attribution
                    top_bar.spawn((
                        Text::new("Chip-8 by Mitchell Vitez"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ));
                    // error message text
                    top_bar.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(COLOR_RED),
                        ErrorText,
                    ));
                    // website
                    top_bar.spawn((
                        Text::new("vitez.me"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ));
                });
            // left/right panel layout
            parent
                .spawn(Node {
                    flex_grow: 1.0,
                    column_gap: Val::Px(6.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    flex_direction: FlexDirection::Row,
                    ..default()
                })
                .with_children(|parent| {
                    // main area (left panel)
                    parent
                        // vertical layout of left panel
                        .spawn((
                            Node {
                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                flex_direction: FlexDirection::Column,
                                flex_grow: 1.0,
                                justify_content: JustifyContent::FlexStart,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Node {
                                    margin: UiRect::left(Val::Px(12.0)).with_top(Val::Px(12.0)),
                                    ..default()
                                },
                                Text::new("Display"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.75, 0.75, 0.75)),
                            ));
                            parent
                                .spawn((
                                    // display container
                                    Node {
                                        border_radius: BorderRadius::all(Val::Px(8.0)),
                                        align_items: AlignItems::FlexStart,
                                        ..default()
                                    },
                                ))
                                .with_children(|parent| {
                                    // display
                                    parent.spawn((
                                        Node {
                                            // width: Val::Percent(100.0),
                                            // width: Val::Px(1024.0),
                                            // height: Val::Px(512.0),
                                            flex_grow: 1.0,
                                            margin: UiRect::all(Val::Px(12.0)),
                                            aspect_ratio: Some(2.0),
                                            ..default()
                                        },
                                        ImageNode::new(display_handle),
                                    ));
                                });
                            parent.spawn((
                                Node {
                                    margin: UiRect::left(Val::Px(12.0)).with_top(Val::Px(12.0)),
                                    ..default()
                                },
                                Text::new("RAM Visualizer"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.75, 0.75, 0.75)),
                            ));
                            parent
                                .spawn((
                                    // RAM visualizer container
                                    Node {
                                        border_radius: BorderRadius::all(Val::Px(8.0)),
                                        align_items: AlignItems::FlexStart,
                                        flex_grow: 1.0,
                                        ..default()
                                    },
                                ))
                                .with_children(|parent| {
                                    // display
                                    parent.spawn((
                                        Node {
                                            // width: Val::Percent(100.0),
                                            // width: Val::Px(1024.0),
                                            // height: Val::Px(512.0),
                                            flex_grow: 1.0,
                                            margin: UiRect::all(Val::Px(12.0)),
                                            aspect_ratio: Some(4.0),
                                            ..default()
                                        },
                                        ImageNode::new(ram_visualizer_handle),
                                    ));
                                });
                        });

                    // right side panel
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(33.0),
                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent
                                        .spawn(Node {
                                            flex_direction: FlexDirection::Column,
                                            width: Val::Percent(50.0),
                                            ..default()
                                        })
                                        .with_children(|parent| {
                                            // execution mode
                                            parent.spawn((
                                                Node {
                                                    margin: UiRect::left(Val::Px(12.0))
                                                        .with_top(Val::Px(12.0)),
                                                    ..default()
                                                },
                                                Text::new("Execution Mode"),
                                                TextFont {
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.75, 0.75, 0.75)),
                                            ));
                                            parent.spawn((
                                                Node {
                                                    margin: UiRect::left(Val::Px(12.0))
                                                        .with_top(Val::Px(12.0)),
                                                    ..default()
                                                },
                                                Text::new(""),
                                                TextFont {
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                                                ExecutionModeMarker,
                                            ));
                                        });

                                    parent
                                        .spawn(Node {
                                            flex_direction: FlexDirection::Column,
                                            width: Val::Percent(50.0),
                                            ..default()
                                        })
                                        .with_children(|parent| {
                                            // cycles
                                            parent.spawn((
                                                Node {
                                                    margin: UiRect::left(Val::Px(12.0))
                                                        .with_top(Val::Px(12.0)),
                                                    ..default()
                                                },
                                                Text::new("Cycles"),
                                                TextFont {
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.75, 0.75, 0.75)),
                                            ));
                                            parent.spawn((
                                                Node {
                                                    margin: UiRect::left(Val::Px(12.0))
                                                        .with_top(Val::Px(12.0)),
                                                    ..default()
                                                },
                                                Text::new(""),
                                                TextFont {
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                                                CyclesMarker,
                                            ));
                                        });
                                });

                            // registers alongside stack
                            parent
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent
                                        .spawn(Node {
                                            flex_direction: FlexDirection::Column,
                                            width: Val::Percent(50.0),
                                            ..default()
                                        })
                                        .with_children(|parent| {
                                            // registers
                                            parent.spawn((
                                                Node {
                                                    margin: UiRect::left(Val::Px(12.0))
                                                        .with_top(Val::Px(12.0)),
                                                    ..default()
                                                },
                                                Text::new("Registers"),
                                                TextFont {
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.75, 0.75, 0.75)),
                                            ));
                                            parent.spawn((
                                                Node {
                                                    margin: UiRect::left(Val::Px(12.0))
                                                        .with_top(Val::Px(12.0)),
                                                    ..default()
                                                },
                                                Text::new(""),
                                                TextFont {
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                                                RegistersMarker,
                                            ));
                                        });

                                    parent
                                        .spawn(Node {
                                            flex_direction: FlexDirection::Column,
                                            width: Val::Percent(50.0),
                                            ..default()
                                        })
                                        .with_children(|parent| {
                                            // stack
                                            parent.spawn((
                                                Node {
                                                    margin: UiRect::left(Val::Px(12.0))
                                                        .with_top(Val::Px(12.0)),
                                                    ..default()
                                                },
                                                Text::new("Stack"),
                                                TextFont {
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.75, 0.75, 0.75)),
                                            ));
                                            parent.spawn((
                                                Node {
                                                    margin: UiRect::left(Val::Px(12.0))
                                                        .with_top(Val::Px(12.0)),
                                                    ..default()
                                                },
                                                Text::new(""),
                                                TextFont {
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                                                StackMarker,
                                            ));
                                        });
                                });

                            // timers alongside pseudo-registers
                            parent
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent
                                        .spawn(Node {
                                            flex_direction: FlexDirection::Column,
                                            width: Val::Percent(50.0),
                                            ..default()
                                        })
                                        .with_children(|parent| {
                                            // registers
                                            parent.spawn((
                                                Node {
                                                    margin: UiRect::left(Val::Px(12.0))
                                                        .with_top(Val::Px(12.0)),
                                                    ..default()
                                                },
                                                Text::new("Timers"),
                                                TextFont {
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.75, 0.75, 0.75)),
                                            ));
                                            parent.spawn((
                                                Node {
                                                    margin: UiRect::left(Val::Px(12.0))
                                                        .with_top(Val::Px(12.0)),
                                                    ..default()
                                                },
                                                Text::new(""),
                                                TextFont {
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                                                TimersMarker,
                                            ));
                                        });

                                    parent
                                        .spawn(Node {
                                            flex_direction: FlexDirection::Column,
                                            width: Val::Percent(50.0),
                                            ..default()
                                        })
                                        .with_children(|parent| {
                                            // stack
                                            parent.spawn((
                                                Node {
                                                    margin: UiRect::left(Val::Px(12.0))
                                                        .with_top(Val::Px(12.0)),
                                                    ..default()
                                                },
                                                Text::new("Pseudo-registers"),
                                                TextFont {
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.75, 0.75, 0.75)),
                                            ));
                                            parent.spawn((
                                                Node {
                                                    margin: UiRect::left(Val::Px(12.0))
                                                        .with_top(Val::Px(12.0)),
                                                    ..default()
                                                },
                                                Text::new(""),
                                                TextFont {
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                                                PseudoRegistersMarker,
                                            ));
                                        });
                                });

                            // recent instructions
                            parent.spawn((
                                Node {
                                    margin: UiRect::left(Val::Px(12.0)).with_top(Val::Px(12.0)),
                                    ..default()
                                },
                                Text::new("Instructions"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.75, 0.75, 0.75)),
                            ));
                            parent.spawn((
                                Node {
                                    margin: UiRect::left(Val::Px(12.0)).with_top(Val::Px(12.0)),
                                    ..default()
                                },
                                Text::new(""),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                                RecentInstructionsMarker,
                            ));

                            // keyboard
                            parent.spawn((
                                Node {
                                    margin: UiRect::left(Val::Px(12.0)).with_top(Val::Px(12.0)),
                                    ..default()
                                },
                                Text::new("Key Map"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.75, 0.75, 0.75)),
                            ));
                            parent.spawn((
                                Node {
                                    margin: UiRect::left(Val::Px(12.0)).with_top(Val::Px(12.0)),
                                    ..default()
                                },
                                Text::new("chip-8    qwerty\n1 2 3 C   1 2 3 4\n4 5 6 D   Q W E R\n7 8 9 E   A S D F\nA 0 B F   Z X C V"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                            ));
                        });
                });
        });
}
