use crate::state::{ClientSession, FocusField, HealthState};
use crate::ui::{palette, spawn_button, spawn_input, spawn_text, text_node, UiAction, UiAssets};
use bevy::prelude::*;

pub fn spawn(root: &mut ChildBuilder, session: &ClientSession, assets: &UiAssets) {
    root.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(8.0),
        ..default()
    })
    .with_children(|header| {
        header.spawn(text_node("NNK Stalker", 38.0, palette::accent(), assets));
        header.spawn(text_node(
            "Кордон · Лобби 4-5 · D20 -> ККК -> гекс -> D6 -> движение",
            17.0,
            palette::muted(),
            assets,
        ));
        let (health_text, health_color) = match session.health {
            HealthState::Checking => ("Сервер: проверка...", palette::accent()),
            HealthState::Online => ("Сервер: онлайн", palette::ok()),
            HealthState::Offline => ("Сервер: недоступен", palette::danger()),
        };
        header.spawn(text_node(health_text, 15.0, health_color, assets));
    });

    root.spawn((
        Node {
            width: Val::Px(520.0),
            max_width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(18.0)),
            margin: UiRect::top(Val::Px(20.0)),
            row_gap: Val::Px(6.0),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(palette::panel()),
        BorderColor(Color::srgba(0.91, 0.88, 0.78, 0.18)),
    ))
    .with_children(|panel| {
        panel.spawn(text_node("Авторизация", 24.0, palette::text(), assets));
        panel.spawn(text_node(
            "Регистрация создает сессию сразу. Логин 3-32, пароль от 8.",
            14.0,
            palette::muted(),
            assets,
        ));
        spawn_input(
            panel,
            "Логин",
            &session.username_input,
            FocusField::Username,
            session.focused == FocusField::Username,
            false,
            assets,
        );
        spawn_input(
            panel,
            "Пароль",
            &session.password_input,
            FocusField::Password,
            session.focused == FocusField::Password,
            true,
            assets,
        );
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                row_gap: Val::Px(8.0),
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            })
            .with_children(|row| {
                let enabled = !session.is_busy();
                spawn_button(row, "Войти", UiAction::Login, enabled, true, assets);
                spawn_button(
                    row,
                    "Регистрация",
                    UiAction::Register,
                    enabled,
                    false,
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
