use crate::state::{ClientSession, FocusField};
use crate::ui::{palette, spawn_button, spawn_input, spawn_text, text_node, UiAction, UiAssets};
use bevy::prelude::*;

pub fn spawn(root: &mut ChildBuilder, session: &ClientSession, assets: &UiAssets) {
    root.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        column_gap: Val::Px(12.0),
        ..default()
    })
    .with_children(|header| {
        header.spawn(text_node("Лобби", 34.0, palette::accent(), assets));
        header
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row| {
                let who = session.display_name.as_deref().unwrap_or("игрок");
                spawn_text(row, format!("Вы: {who}"), 16.0, palette::text(), assets);
                spawn_button(row, "Выйти", UiAction::Logout, true, false, assets);
            });
    });

    root.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(16.0),
            row_gap: Val::Px(16.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
    ))
    .with_children(|layout| {
        spawn_controls(layout, session, assets);
        spawn_room(layout, session, assets);
    });
}

fn spawn_controls(parent: &mut ChildBuilder, session: &ClientSession, assets: &UiAssets) {
    parent
        .spawn((
            Node {
                width: Val::Px(420.0),
                max_width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(palette::panel()),
            BorderColor(Color::srgba(0.91, 0.88, 0.78, 0.18)),
        ))
        .with_children(|panel| {
            panel.spawn(text_node("Комната", 23.0, palette::text(), assets));
            panel.spawn(text_node(
                "Создайте новую комнату или войдите по коду.",
                14.0,
                palette::muted(),
                assets,
            ));
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_button(
                        row,
                        "Создать лобби",
                        UiAction::CreateRoom,
                        !session.is_busy(),
                        true,
                        assets,
                    );
                });
            spawn_input(
                panel,
                "Код комнаты",
                &session.room_code_input,
                FocusField::RoomCode,
                session.focused == FocusField::RoomCode,
                false,
                assets,
            );
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_button(
                        row,
                        "Войти в лобби",
                        UiAction::JoinRoom,
                        !session.is_busy(),
                        false,
                        assets,
                    );
                    let has_lobby = session.lobby.is_some();
                    spawn_button(
                        row,
                        "Готов",
                        UiAction::Ready,
                        has_lobby && !session.is_busy() && !session.me_ready(),
                        false,
                        assets,
                    );
                    spawn_button(
                        row,
                        "Не готов",
                        UiAction::Unready,
                        has_lobby && !session.is_busy() && session.me_ready(),
                        false,
                        assets,
                    );
                    let can_start = session
                        .lobby
                        .as_ref()
                        .map(|lobby| lobby.can_start)
                        .unwrap_or(false);
                    spawn_button(
                        row,
                        "Старт (ГМ)",
                        UiAction::Start,
                        session.me_is_gm() && can_start && !session.is_busy(),
                        true,
                        assets,
                    );
                });
            if let Some(error) = &session.error {
                spawn_text(panel, error, 15.0, palette::danger(), assets);
            } else {
                spawn_text(panel, &session.status, 14.0, palette::muted(), assets);
            }
        });
}

fn spawn_room(parent: &mut ChildBuilder, session: &ClientSession, assets: &UiAssets) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                min_width: Val::Px(460.0),
                max_width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(10.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(palette::panel()),
            BorderColor(Color::srgba(0.91, 0.88, 0.78, 0.18)),
        ))
        .with_children(|panel| {
            let Some(lobby) = &session.lobby else {
                panel.spawn(text_node(
                    "Активного лобби нет. Создайте комнату или введите код.",
                    17.0,
                    palette::muted(),
                    assets,
                ));
                return;
            };
            panel.spawn(text_node(
                format!("Комната {}", lobby.code),
                25.0,
                palette::accent(),
                assets,
            ));
            panel.spawn(text_node(
                format!(
                    "Фаза: {} · игроков {}/{} · минимум {} · can_start: {}",
                    phase_label(&lobby.phase),
                    lobby.player_count,
                    lobby.max_players,
                    lobby.min_players,
                    if lobby.can_start { "да" } else { "нет" }
                ),
                15.0,
                palette::muted(),
                assets,
            ));
            panel.spawn(text_node(
                format!(
                    "Миссия {}: {}",
                    lobby.mission_id,
                    if lobby.mission_title.is_empty() {
                        "без названия"
                    } else {
                        &lobby.mission_title
                    }
                ),
                16.0,
                palette::text(),
                assets,
            ));
            panel.spawn(text_node(
                format!("Цель: {}", nonempty(&lobby.mission_objective)),
                14.0,
                palette::muted(),
                assets,
            ));
            panel.spawn(text_node("Места", 20.0, palette::text(), assets));
            for player in &lobby.players {
                panel
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(7.0)),
                            border: UiRect::bottom(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor(Color::srgba(0.91, 0.88, 0.78, 0.12)),
                    ))
                    .with_children(|row| {
                        let gm = if player.is_gm { " · ГМ" } else { "" };
                        row.spawn(text_node(
                            format!("{}{}", player.display_name, gm),
                            16.0,
                            palette::text(),
                            assets,
                        ));
                        row.spawn(text_node(
                            if player.ready {
                                "готов"
                            } else {
                                "ждем"
                            },
                            15.0,
                            if player.ready {
                                palette::ok()
                            } else {
                                palette::muted()
                            },
                            assets,
                        ));
                    });
            }
        });
}

fn phase_label(phase: &str) -> &str {
    match phase {
        "playing" => "игра",
        _ => "лобби",
    }
}

fn nonempty(value: &str) -> &str {
    if value.trim().is_empty() {
        "уточняется"
    } else {
        value
    }
}
