//! Bevy GUI example for the orchestrator
//!
//! This example demonstrates how to integrate the orchestrator with Bevy
//! for a visual representation of the galaxy simulation.

use bevy::prelude::*;
use bevy::log::LogPlugin;
use orchestrator::{Orchestrator, GuiEvent, GuiState};
use common_game::utils::ID;
use std::time::Duration;

/// Main Bevy app state
#[derive(Resource)]
struct AppState {
    orchestrator: Orchestrator,
    last_update: f32,
}

/// Marker component for planets
#[derive(Component)]
struct PlanetMarker {
    id: ID,
}

/// Marker component for explorers
#[derive(Component)]
struct ExplorerMarker {
    id: ID,
}

/// Main function
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::INFO,
            filter: "info,wgpu_core=warn,wgpu_hal=warn,orchestrator=debug".into(),
        }))
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.15)))
        .add_systems(Startup, (setup_camera, setup_orchestrator, setup_ui))
        .add_systems(Update, (
            update_simulation,
            update_visuals,
            handle_user_input,
            update_ui,
        ))
        .run();
}

/// Setup the camera
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_scale(Vec3::new(0.5, 0.5, 1.0)),
        ..default()
    });
}

/// Setup the orchestrator with dummy data
fn setup_orchestrator(mut commands: Commands) {
    // Create orchestrator
    let mut orchestrator = Orchestrator::new()
        .expect("Failed to create orchestrator");

    // Create 7 dummy planets (in a real app, these would be actual planet implementations)
    for planet_id in 1..=7 {
        // Note: In reality, you'd need actual planet channels here
        // This is just for demonstration
        println!("Planet {} would be created here", planet_id);
    }

    // Create a few explorers
    for explorer_id in 1..=3 {
        println!("Explorer {} would be created here", explorer_id);
    }

    commands.insert_resource(AppState {
        orchestrator,
        last_update: 0.0,
    });
}

/// Setup the UI
fn setup_ui(mut commands: Commands) {
    // Root node
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        ..default()
    })
        .with_children(|parent| {
            // Top bar with controls
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(50.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(),
                ..default()
            })
                .with_children(|parent| {
                    // Title
                    parent.spawn(TextBundle::from_section(
                        "Galaxy Orchestrator",
                        TextStyle {
                            font_size: 24.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));

                    // Controls
                    parent.spawn(NodeBundle {
                        style: Style {
                            gap: Size::width(Val::Px(10.0)),
                            ..default()
                        },
                        ..default()
                    })
                        .with_children(|parent| {
                            // Mood display
                            parent.spawn(TextBundle::from_section(
                                "😐",
                                TextStyle {
                                    font_size: 30.0,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            ));
                        });
                });

            // Stats panel (right side)
            parent.spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Px(10.0),
                    top: Val::Px(60.0),
                    width: Val::Px(200.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.5).into(),
                ..default()
            })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Game Stats",
                        TextStyle {
                            font_size: 18.0,
                            color: Color::YELLOW,
                            ..default()
                        },
                    ));

                    // Stats will be updated dynamically
                    parent.spawn((
                        TextBundle::from_section(
                            "",
                            TextStyle {
                                font_size: 14.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),
                        Name::new("stats_text"),
                    ));
                });
        });
}

/// Update the simulation
fn update_simulation(
    time: Res<Time>,
    mut app_state: ResMut<AppState>,
) {
    // Update every 0.1 seconds
    app_state.last_update += time.delta_seconds();
    if app_state.last_update >= 0.1 {
        app_state.last_update = 0.0;

        // In a real app, you would run the orchestrator async
        // For this example, we'll just simulate some events

        // Simulate occasional asteroids and sunrays
        let time = time.elapsed_seconds() as u64;
        if time % 5 == 0 {
            let planet_id = ((time / 5) % 7) as ID + 1;

            // Alternate between asteroids and sunrays
            if time % 10 == 0 {
                app_state.orchestrator.gui_sender.send(GuiEvent::AsteroidSent(planet_id))
                    .expect("Failed to send GUI event");
            } else {
                app_state.orchestrator.gui_sender.send(GuiEvent::SunraySent(planet_id))
                    .expect("Failed to send GUI event");
            }
        }
    }
}

/// Update visuals based on GUI state
fn update_visuals(
    app_state: Res<AppState>,
    mut commands: Commands,
    planet_query: Query<Entity, With<PlanetMarker>>,
    explorer_query: Query<Entity, With<ExplorerMarker>>,
) {
    // Get current GUI state
    let gui_state = app_state.orchestrator.get_gui_state();

    // Clear existing entities
    for entity in planet_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in explorer_query.iter() {
        commands.entity(entity).despawn();
    }

    // Create planet visuals
    for planet in &gui_state.planets {
        let color = match planet.planet_type.as_str() {
            "A" => Color::RED,
            "B" => Color::BLUE,
            "C" => Color::GREEN,
            "D" => Color::PURPLE,
            _ => Color::GRAY,
        };

        let mut transform = Transform::from_xyz(planet.x, planet.y, 0.0);
        transform.scale = Vec3::splat(20.0 + planet.energy_level * 30.0);

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(1.0, 1.0)),
                    ..default()
                },
                transform,
                ..default()
            },
            PlanetMarker { id: planet.id },
        ))
            .with_children(|parent| {
                // Planet ID label
                parent.spawn(Text2dBundle {
                    text: Text::from_section(
                        format!("P{}", planet.id),
                        TextStyle {
                            font_size: 12.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    transform: Transform::from_xyz(0.0, 0.0, 1.0),
                    ..default()
                });

                // Energy indicator
                if planet.charged_cells > 0 {
                    parent.spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::YELLOW,
                            custom_size: Some(Vec2::new(0.5, 0.5)),
                            ..default()
                        },
                        transform: Transform::from_xyz(0.0, -0.8, 0.0),
                        ..default()
                    });
                }

                // Rocket indicator
                if planet.has_rocket {
                    parent.spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::ORANGE,
                            custom_size: Some(Vec2::new(0.3, 0.6)),
                            ..default()
                        },
                        transform: Transform::from_xyz(0.8, 0.0, 0.0),
                        ..default()
                    });
                }
            });
    }

    // Create explorer visuals
    for explorer in &gui_state.explorers {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::CYAN,
                    custom_size: Some(Vec2::new(0.5, 0.5)),
                    ..default()
                },
                transform: Transform::from_xyz(explorer.x, explorer.y, 0.0),
                ..default()
            },
            ExplorerMarker { id: explorer.id },
        ))
            .with_children(|parent| {
                // Explorer ID label
                parent.spawn(Text2dBundle {
                    text: Text::from_section(
                        format!("E{}", explorer.id),
                        TextStyle {
                            font_size: 10.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    transform: Transform::from_xyz(0.0, 0.0, 1.0),
                    ..default()
                });
            });
    }
}

/// Handle user input
fn handle_user_input(
    keys: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    app_state: Res<AppState>,
) {
    // Left click to send asteroid
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some(cursor_position) = windows.single().cursor_position() {
            let (camera, camera_transform) = camera_query.single();

            if let Some(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
                // Find clicked planet
                let gui_state = app_state.orchestrator.get_gui_state();
                for planet in &gui_state.planets {
                    let distance = Vec2::new(planet.x, planet.y).distance(world_position);
                    if distance < 30.0 {
                        // Send asteroid to clicked planet
                        let _ = app_state.orchestrator.gui_sender.send(GuiEvent::SendAsteroid {
                            planet_id: planet.id,
                        });
                        break;
                    }
                }
            }
        }
    }

    // Right click to send sunray
    if mouse_buttons.just_pressed(MouseButton::Right) {
        if let Some(cursor_position) = windows.single().cursor_position() {
            let (camera, camera_transform) = camera_query.single();

            if let Some(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
                let gui_state = app_state.orchestrator.get_gui_state();
                for planet in &gui_state.planets {
                    let distance = Vec2::new(planet.x, planet.y).distance(world_position);
                    if distance < 30.0 {
                        // Send sunray to clicked planet
                        let _ = app_state.orchestrator.gui_sender.send(GuiEvent::SendSunray {
                            planet_id: planet.id,
                        });
                        break;
                    }
                }
            }
        }
    }

    // Space to toggle simulation
    if keys.just_pressed(KeyCode::Space) {
        println!("Space pressed - would toggle simulation");
    }
}

/// Update UI text
fn update_ui(
    app_state: Res<AppState>,
    mut text_query: Query<&mut Text, With<Name>>,
) {
    let gui_state = app_state.orchestrator.get_gui_state();

    for mut text in text_query.iter_mut() {
        if text.sections[0].value.is_empty() || text.sections[0].value.contains("Game Stats") {
            // Update stats text
            *text = Text::from_section(
                format!(
                    "Asteroids: {}\nSunrays: {}\nPlanets Alive: {}\nExplorers: {}\nResources: {}\nMood: {}",
                    gui_state.game_stats.asteroids_sent,
                    gui_state.game_stats.sunrays_sent,
                    gui_state.planets.len(),
                    gui_state.explorers.len(),
                    gui_state.game_stats.resources_generated,
                    gui_state.get_current_mood()
                ),
                TextStyle {
                    font_size: 14.0,
                    color: Color::WHITE,
                    ..default()
                },
            );
        }
    }
}