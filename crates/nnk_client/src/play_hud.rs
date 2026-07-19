use crate::state::{same_user_id, ClientSession};
use crate::ui::{palette, spawn_button, spawn_text, text_node, UiAction, UiAssets};
use bevy::prelude::*;
use nnk_protocol::{LobbyPlayerView, LobbyView};

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
        header.spawn(text_node("Партия идет", 32.0, palette::accent(), assets));
        header
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row| {
                spawn_button(
                    row,
                    "Обновить",
                    UiAction::RefreshLobby,
                    !session.is_busy(),
                    false,
                    assets,
                );
                spawn_button(row, "Выйти", UiAction::Logout, true, false, assets);
            });
    });

    let Some(lobby) = &session.lobby else {
        spawn_text(
            root,
            "Нет состояния партии. Обновите страницу.",
            18.0,
            palette::danger(),
            assets,
        );
        return;
    };

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
        spawn_tablet(layout, session, lobby, assets);
        spawn_roster(layout, lobby, assets);
    });
}

fn spawn_tablet(
    parent: &mut ChildBuilder,
    session: &ClientSession,
    lobby: &LobbyView,
    assets: &UiAssets,
) {
    let me = me_player(session, lobby);
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                min_width: Val::Px(560.0),
                max_width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(10.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(palette::panel()),
            BorderColor(Color::srgba(0.77, 0.64, 0.35, 0.42)),
        ))
        .with_children(|panel| {
            panel.spawn(text_node(
                format!("Комната {} · фаза игра", lobby.code),
                18.0,
                palette::muted(),
                assets,
            ));
            panel.spawn(text_node(
                mission_title(lobby),
                28.0,
                palette::accent(),
                assets,
            ));
            panel.spawn(text_node(
                format!("Цель: {}", nonempty(&lobby.mission_objective)),
                17.0,
                palette::text(),
                assets,
            ));
            panel.spawn(text_node(
                format!(
                    "Ходит: {} · День {}, {:02}:00 · игроков {}/{}",
                    active_name(lobby),
                    lobby.game_day,
                    lobby.game_hour,
                    lobby.player_count,
                    lobby.max_players
                ),
                16.0,
                palette::muted(),
                assets,
            ));
            spawn_field_grid(panel, me, lobby, assets);
            panel.spawn(text_node(
                format!(
                    "Брифинг: {}",
                    compact(nonempty(&lobby.mission_briefing), 360)
                ),
                15.0,
                palette::muted(),
                assets,
            ));
            if let Some(error) = &session.error {
                spawn_text(panel, error, 15.0, palette::danger(), assets);
            } else {
                spawn_text(panel, &session.status, 14.0, palette::muted(), assets);
            }
        });
}

fn spawn_field_grid(
    parent: &mut ChildBuilder,
    me: Option<&LobbyPlayerView>,
    lobby: &LobbyView,
    assets: &UiAssets,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(8.0),
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|grid| {
            field(
                grid,
                "Позывной",
                me.map(|p| p.display_name.as_str()).unwrap_or("-"),
                assets,
            );
            field(
                grid,
                "Локация",
                me.and_then(|p| p.location.as_deref()).unwrap_or("-"),
                assets,
            );
            field(
                grid,
                "Сектор",
                &me.and_then(|p| p.sector)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "-".into()),
                assets,
            );
            field(grid, "ККК", &ccc_label(me), assets);
            field(
                grid,
                "Текущий гекс",
                &me.map(|p| coord_label(&p.hex))
                    .unwrap_or_else(|| "-".into()),
                assets,
            );
            field(
                grid,
                "Цель гекс",
                &me.and_then(|p| p.target_hex.map(|hex| coord_label(&hex)))
                    .unwrap_or_else(|| "-".into()),
                assets,
            );
            field(
                grid,
                "ОД",
                &me.map(|p| p.move_points.to_string())
                    .unwrap_or_else(|| "-".into()),
                assets,
            );
            field(
                grid,
                "Стадия",
                me.map(|p| stage_label(&p.travel_stage)).unwrap_or("-"),
                assets,
            );
            field(grid, "Активный", &active_name(lobby), assets);
            field(grid, "Миссия", &format!("{}", lobby.mission_id), assets);
        });
}

fn field(parent: &mut ChildBuilder, label: &str, value: &str, assets: &UiAssets) {
    parent
        .spawn((
            Node {
                width: Val::Px(172.0),
                min_height: Val::Px(62.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(palette::input()),
            BorderColor(Color::srgba(0.91, 0.88, 0.78, 0.14)),
        ))
        .with_children(|cell| {
            cell.spawn(text_node(label, 12.0, palette::muted(), assets));
            cell.spawn(text_node(value, 15.0, palette::text(), assets));
        });
}

fn spawn_roster(parent: &mut ChildBuilder, lobby: &LobbyView, assets: &UiAssets) {
    parent
        .spawn((
            Node {
                width: Val::Px(460.0),
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
            panel.spawn(text_node("Отряд и журнал", 22.0, palette::text(), assets));
            for player in &lobby.players {
                let active = lobby
                    .active_user_id
                    .map(|id| id == player.user_id)
                    .unwrap_or(false);
                panel.spawn(text_node(
                    format!(
                        "{}{} · {} · {} · сектор {} · гекс {} · цель {} · ОД {}",
                        player.display_name,
                        if player.is_gm { " [ГМ]" } else { "" },
                        if active { "ходит" } else { "ждет" },
                        player.location.as_deref().unwrap_or("-"),
                        player
                            .sector
                            .map(|sector| sector.to_string())
                            .unwrap_or_else(|| "-".into()),
                        coord_label(&player.hex),
                        player
                            .target_hex
                            .map(|hex| coord_label(&hex))
                            .unwrap_or_else(|| "-".into()),
                        player.move_points
                    ),
                    14.0,
                    if active {
                        palette::accent()
                    } else {
                        palette::text()
                    },
                    assets,
                ));
            }
            if !lobby.npcs.is_empty() {
                panel.spawn(text_node("НПС", 18.0, palette::text(), assets));
                for npc in &lobby.npcs {
                    panel.spawn(text_node(
                        format!(
                            "{} · {} · гекс {}",
                            npc.name,
                            npc.kind,
                            coord_label(&npc.hex)
                        ),
                        14.0,
                        palette::muted(),
                        assets,
                    ));
                }
            }
            panel.spawn(text_node("История", 18.0, palette::text(), assets));
            for line in lobby.history.iter().rev().take(8) {
                panel.spawn(text_node(
                    format_history(line),
                    13.0,
                    palette::muted(),
                    assets,
                ));
            }
        });
}

fn me_player<'a>(session: &ClientSession, lobby: &'a LobbyView) -> Option<&'a LobbyPlayerView> {
    let user_id = session.user_id.as_deref()?;
    lobby
        .players
        .iter()
        .find(|player| same_user_id(&player.user_id.to_string(), user_id))
}

fn mission_title(lobby: &LobbyView) -> String {
    if lobby.mission_title.trim().is_empty() {
        format!("Миссия {}", lobby.mission_id)
    } else {
        format!("Миссия {} · {}", lobby.mission_id, lobby.mission_title)
    }
}

fn active_name(lobby: &LobbyView) -> String {
    lobby
        .active_display_name
        .clone()
        .or_else(|| {
            lobby.active_user_id.and_then(|id| {
                lobby
                    .players
                    .iter()
                    .find(|player| player.user_id == id)
                    .map(|player| player.display_name.clone())
            })
        })
        .unwrap_or_else(|| "-".into())
}

fn coord_label(hex: &nnk_domain::HexCoord) -> String {
    format!("({}, {})", hex.q, hex.r)
}

fn ccc_label(player: Option<&LobbyPlayerView>) -> String {
    let Some(player) = player else {
        return "-".into();
    };
    if player.ccc_tens.is_none() && player.ccc_units.is_none() && player.sector.is_none() {
        "-".into()
    } else {
        format!(
            "{} / {} -> {}",
            player
                .ccc_tens
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".into()),
            player
                .ccc_units
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".into()),
            player
                .sector
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".into())
        )
    }
}

fn stage_label(stage: &str) -> &str {
    match stage {
        "need_location" => "нужна локация",
        "need_sector" => "нужен сектор",
        "need_hex" => "нужен гекс",
        "need_d6" => "нужен D6",
        "need_move" => "движение",
        _ => stage,
    }
}

fn nonempty(value: &str) -> &str {
    if value.trim().is_empty() {
        "-"
    } else {
        value
    }
}

fn compact(value: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for ch in value.chars().take(max_chars) {
        out.push(ch);
    }
    if value.chars().count() > max_chars {
        out.push_str("...");
    }
    out
}

fn format_history(line: &str) -> String {
    if line == "room_created" {
        return "Комната создана".into();
    }
    if let Some(name) = line.strip_prefix("player_joined:") {
        return format!("Игрок вошел: {name}");
    }
    if line.starts_with("game_started:") {
        return "Игра началась".into();
    }
    line.replace("ready:", "готовность: ")
        .replace("true", "да")
        .replace("false", "нет")
}
