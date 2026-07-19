use crate::net::{self, AuthMode, NetEvent, NetRx, NetTx};
use crate::state::{AppState, ClientSession, FocusField, HealthState};
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;

const FONT_PATH: &str = "fonts/NotoSans-Regular.ttf";

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClientSession::default())
            .insert_resource(HealthPoll(Timer::from_seconds(20.0, TimerMode::Repeating)))
            .add_systems(Startup, setup_ui)
            .add_systems(
                Update,
                (
                    drain_net_events,
                    poll_health,
                    handle_keyboard_input,
                    handle_button_actions,
                    update_button_visuals,
                    redraw_ui,
                ),
            );
    }
}

#[derive(Resource, Clone)]
pub struct UiAssets {
    pub font: Handle<Font>,
}

#[derive(Resource)]
struct HealthPoll(Timer);

#[derive(Component)]
struct UiRoot {
    version: u64,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiAction {
    Focus(FocusField),
    Login,
    Register,
    Logout,
    CreateRoom,
    JoinRoom,
    Ready,
    Unready,
    Start,
    RefreshLobby,
    /// Play: D20 location → CCC → D20 hex → D6 → move.
    RollD20Location,
    DrawCcc,
    RollD20Hex,
    RollD6Move,
    MoveToken { q: i32, r: i32 },
}

#[derive(Component, Debug, Clone, Copy)]
pub struct UiEnabled(pub bool);

/// Primary CTA (gold fill). Must stay high-contrast — never dark text on dark fill.
#[derive(Component, Debug, Clone, Copy)]
pub struct UiPrimary(pub bool);

pub mod palette {
    use bevy::prelude::*;

    pub fn bg() -> Color {
        Color::srgba(0.07, 0.09, 0.06, 1.0)
    }

    pub fn panel() -> Color {
        Color::srgba(0.12, 0.14, 0.10, 0.96)
    }

    pub fn input() -> Color {
        Color::srgba(0.06, 0.08, 0.05, 1.0)
    }

    pub fn input_focused() -> Color {
        Color::srgba(0.16, 0.14, 0.08, 1.0)
    }

    pub fn button() -> Color {
        // Secondary: dark olive with clear edge — text must stay light.
        Color::srgba(0.18, 0.20, 0.14, 1.0)
    }

    pub fn button_hover() -> Color {
        Color::srgba(0.28, 0.30, 0.20, 1.0)
    }

    pub fn button_pressed() -> Color {
        Color::srgba(0.36, 0.32, 0.18, 1.0)
    }

    pub fn button_disabled() -> Color {
        Color::srgba(0.12, 0.13, 0.10, 1.0)
    }

    pub fn primary() -> Color {
        Color::srgb(0.82, 0.68, 0.34)
    }

    pub fn primary_hover() -> Color {
        Color::srgb(0.90, 0.76, 0.42)
    }

    pub fn primary_pressed() -> Color {
        Color::srgb(0.70, 0.56, 0.24)
    }

    pub fn primary_disabled() -> Color {
        Color::srgba(0.42, 0.36, 0.20, 1.0)
    }

    pub fn accent() -> Color {
        Color::srgb(0.86, 0.72, 0.38)
    }

    pub fn text() -> Color {
        Color::srgb(0.96, 0.93, 0.84)
    }

    pub fn text_on_primary() -> Color {
        Color::srgb(0.12, 0.10, 0.05)
    }

    pub fn muted() -> Color {
        // Readable gray-gold — not near-black on dark panels.
        Color::srgb(0.78, 0.74, 0.62)
    }

    pub fn disabled_text() -> Color {
        Color::srgb(0.70, 0.66, 0.54)
    }

    pub fn ok() -> Color {
        Color::srgb(0.55, 0.78, 0.40)
    }

    pub fn danger() -> Color {
        Color::srgb(0.90, 0.48, 0.38)
    }
}

pub fn text_node(
    value: impl Into<String>,
    size: f32,
    color: Color,
    assets: &UiAssets,
) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font: assets.font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(color),
    )
}

pub fn spawn_text(
    parent: &mut ChildBuilder,
    value: impl Into<String>,
    size: f32,
    color: Color,
    assets: &UiAssets,
) {
    parent.spawn(text_node(value, size, color, assets));
}

pub fn spawn_button(
    parent: &mut ChildBuilder,
    label: impl Into<String>,
    action: UiAction,
    enabled: bool,
    primary: bool,
    assets: &UiAssets,
) {
    let (bg, text_color, border) = button_colors(primary, enabled, Interaction::None);
    parent
        .spawn((
            Button,
            Node {
                min_width: Val::Px(148.0),
                height: Val::Px(44.0),
                padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                margin: UiRect::right(Val::Px(8.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(bg),
            BorderColor(border),
            action,
            UiEnabled(enabled),
            UiPrimary(primary),
        ))
        .with_children(|button| {
            button.spawn(text_node(label, 16.0, text_color, assets));
        });
}

fn button_colors(primary: bool, enabled: bool, interaction: Interaction) -> (Color, Color, Color) {
    if !enabled {
        if primary {
            (
                palette::primary_disabled(),
                palette::text_on_primary(),
                palette::accent(),
            )
        } else {
            (
                palette::button_disabled(),
                palette::disabled_text(),
                Color::srgba(0.91, 0.88, 0.78, 0.28),
            )
        }
    } else if primary {
        let bg = match interaction {
            Interaction::Pressed => palette::primary_pressed(),
            Interaction::Hovered => palette::primary_hover(),
            Interaction::None => palette::primary(),
        };
        (bg, palette::text_on_primary(), palette::accent())
    } else {
        let bg = match interaction {
            Interaction::Pressed => palette::button_pressed(),
            Interaction::Hovered => palette::button_hover(),
            Interaction::None => palette::button(),
        };
        (
            bg,
            palette::text(),
            Color::srgba(0.91, 0.88, 0.78, 0.35),
        )
    }
}

pub fn spawn_input(
    parent: &mut ChildBuilder,
    label: &str,
    value: &str,
    field: FocusField,
    focused: bool,
    password: bool,
    assets: &UiAssets,
) {
    let display = if password {
        "*".repeat(value.chars().count())
    } else {
        value.to_string()
    };
    let cursor = if focused { "|" } else { "" };
    parent.spawn(text_node(label, 13.0, palette::muted(), assets));
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(42.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                margin: UiRect::bottom(Val::Px(8.0)),
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(if focused {
                palette::input_focused()
            } else {
                palette::input()
            }),
            BorderColor(if focused {
                palette::accent()
            } else {
                Color::srgba(0.91, 0.88, 0.78, 0.16)
            }),
            UiAction::Focus(field),
            UiEnabled(true),
        ))
        .with_children(|input| {
            input.spawn(text_node(
                format!("{display}{cursor}"),
                17.0,
                palette::text(),
                assets,
            ));
        });
}

fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    net_tx: Res<NetTx>,
    mut session: ResMut<ClientSession>,
) {
    commands.insert_resource(UiAssets {
        font: asset_server.load(FONT_PATH),
    });
    net::spawn_health(net_tx.0.clone());
    if let Some(token) = session.token.clone() {
        session.begin_request("восстановление сессии...");
        net::spawn_validate_token(token, net_tx.0.clone());
    } else {
        session.mark_dirty();
    }
}

fn redraw_ui(
    mut commands: Commands,
    roots: Query<(Entity, &UiRoot)>,
    session: Res<ClientSession>,
    assets: Option<Res<UiAssets>>,
) {
    let Some(assets) = assets else {
        return;
    };
    if roots
        .iter()
        .any(|(_, root)| root.version == session.ui_version)
    {
        return;
    }
    for (entity, _) in roots.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(18.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(palette::bg()),
            UiRoot {
                version: session.ui_version,
            },
        ))
        .with_children(|root| match session.app_state {
            AppState::Auth => crate::auth_ui::spawn(root, &session, &assets),
            AppState::Lobby => crate::lobby_ui::spawn(root, &session, &assets),
            AppState::Playing => crate::play_hud::spawn(root, &session, &assets),
        });
}

fn poll_health(
    time: Res<Time>,
    mut poll: ResMut<HealthPoll>,
    net_tx: Res<NetTx>,
    mut session: ResMut<ClientSession>,
) {
    if poll.0.tick(time.delta()).just_finished() {
        session.health = HealthState::Checking;
        session.mark_dirty();
        net::spawn_health(net_tx.0.clone());
    }
}

fn drain_net_events(net_rx: Res<NetRx>, net_tx: Res<NetTx>, mut session: ResMut<ClientSession>) {
    let Ok(rx) = net_rx.0.lock() else {
        return;
    };
    for event in rx.try_iter() {
        match event {
            NetEvent::Health { state, message } => {
                session.health = state;
                session.status = message;
                session.mark_dirty();
            }
            NetEvent::Authenticated {
                token,
                user_id,
                username,
            } => {
                session.token = Some(token);
                session.user_id = Some(user_id);
                session.display_name = Some(username);
                session.app_state = AppState::Lobby;
                session.finish_request("авторизация ок");
            }
            NetEvent::Lobby { lobby, connect_ws } => {
                let code = lobby.code.clone();
                session.inflight = session.inflight.saturating_sub(1);
                session.status = "лобби обновлено".into();
                session.set_lobby(lobby);
                if connect_ws {
                    if let Some(token) = session.token.clone() {
                        session.ws_generation = session.ws_generation.wrapping_add(1);
                        net::spawn_ws(code, token, session.ws_generation, net_tx.0.clone());
                    }
                }
            }
            NetEvent::WsStatus {
                generation,
                message,
            } if generation == session.ws_generation => {
                session.status = message;
                session.mark_dirty();
            }
            NetEvent::WsError {
                generation,
                message,
            } if generation == session.ws_generation => {
                session.error = Some(message);
                session.mark_dirty();
            }
            NetEvent::WsStatus { .. } | NetEvent::WsError { .. } => {}
            NetEvent::Error { message } => {
                session.set_error(message);
            }
        }
    }
}

fn handle_button_actions(
    mut interactions: Query<
        (&Interaction, &UiAction, &UiEnabled),
        (Changed<Interaction>, With<Button>),
    >,
    mut session: ResMut<ClientSession>,
    net_tx: Res<NetTx>,
) {
    for (interaction, action, enabled) in interactions.iter_mut() {
        if *interaction != Interaction::Pressed || !enabled.0 {
            continue;
        }
        match *action {
            UiAction::Focus(field) => {
                session.focused = field;
                session.mark_dirty();
            }
            UiAction::Login => start_auth(AuthMode::Login, &mut session, &net_tx),
            UiAction::Register => start_auth(AuthMode::Register, &mut session, &net_tx),
            UiAction::Logout => session.logout(),
            UiAction::CreateRoom => {
                if let Some(token) = authed_token(&mut session) {
                    session.begin_request("создание лобби...");
                    net::spawn_create_room(token, net_tx.0.clone());
                }
            }
            UiAction::JoinRoom => start_join(&mut session, &net_tx),
            UiAction::Ready => start_ready(true, &mut session, &net_tx),
            UiAction::Unready => start_ready(false, &mut session, &net_tx),
            UiAction::Start => start_game(&mut session, &net_tx),
            UiAction::RefreshLobby => start_refresh(&mut session, &net_tx),
            UiAction::RollD20Location => start_play_action(
                nnk_protocol::ClientMsg::RollD20Location,
                "бросок D20: локация...",
                &mut session,
                &net_tx,
            ),
            UiAction::DrawCcc => start_play_action(
                nnk_protocol::ClientMsg::DrawCcc,
                "вытягиваем ККК...",
                &mut session,
                &net_tx,
            ),
            UiAction::RollD20Hex => start_play_action(
                nnk_protocol::ClientMsg::RollD20Hex,
                "бросок D20: гекс...",
                &mut session,
                &net_tx,
            ),
            UiAction::RollD6Move => start_play_action(
                nnk_protocol::ClientMsg::RollD6Move,
                "бросок D6: ОД...",
                &mut session,
                &net_tx,
            ),
            UiAction::MoveToken { q, r } => start_play_action(
                nnk_protocol::ClientMsg::MoveToken {
                    to: nnk_domain::HexCoord { q, r },
                },
                "ход токена...",
                &mut session,
                &net_tx,
            ),
        }
    }
}

fn handle_keyboard_input(
    mut events: EventReader<KeyboardInput>,
    mut session: ResMut<ClientSession>,
    net_tx: Res<NetTx>,
) {
    for event in events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            Key::Character(chars) => {
                for ch in chars.chars().filter(|ch| !ch.is_control()) {
                    push_char(&mut session, ch);
                }
            }
            Key::Backspace => {
                active_input(&mut session).pop();
                session.mark_dirty();
            }
            Key::Enter => match session.app_state {
                AppState::Auth => start_auth(AuthMode::Login, &mut session, &net_tx),
                AppState::Lobby => start_join(&mut session, &net_tx),
                AppState::Playing => start_refresh(&mut session, &net_tx),
            },
            _ => {}
        }
    }
}

fn update_button_visuals(
    mut query: Query<
        (
            &Interaction,
            &UiEnabled,
            &UiPrimary,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut texts: Query<&mut TextColor>,
) {
    for (interaction, enabled, primary, mut bg, mut border, children) in query.iter_mut() {
        let (next_bg, next_text, next_border) =
            button_colors(primary.0, enabled.0, *interaction);
        bg.0 = next_bg;
        border.0 = next_border;
        for child in children.iter() {
            if let Ok(mut text_color) = texts.get_mut(*child) {
                text_color.0 = next_text;
            }
        }
    }
}

fn start_auth(mode: AuthMode, session: &mut ClientSession, net_tx: &NetTx) {
    if session.is_busy() {
        return;
    }
    let username = session.username_input.trim().to_string();
    let password = session.password_input.clone();
    if username.len() < 3 || password.len() < 8 {
        session.set_error("логин от 3 символов, пароль от 8");
        return;
    }
    let status = match mode {
        AuthMode::Login => "вход...",
        AuthMode::Register => "регистрация...",
    };
    session.begin_request(status);
    net::spawn_auth(mode, username, password, net_tx.0.clone());
}

fn start_join(session: &mut ClientSession, net_tx: &NetTx) {
    if session.is_busy() {
        return;
    }
    let code = session.room_code_input.trim().to_uppercase();
    if code.len() < 4 {
        session.set_error("введите код комнаты");
        return;
    }
    if let Some(token) = authed_token(session) {
        session.begin_request("вход в лобби...");
        net::spawn_join_room(code, token, net_tx.0.clone());
    }
}

fn start_ready(ready: bool, session: &mut ClientSession, net_tx: &NetTx) {
    if session.is_busy() {
        return;
    }
    let Some(token) = authed_token(session) else {
        return;
    };
    let Some(code) = current_code(session) else {
        session.set_error("нет активной комнаты");
        return;
    };
    session.begin_request(if ready {
        "готовность..."
    } else {
        "снятие готовности..."
    });
    net::spawn_ready(code, ready, token, net_tx.0.clone());
}

fn start_game(session: &mut ClientSession, net_tx: &NetTx) {
    if session.is_busy() {
        return;
    }
    if !session.me_is_gm() {
        session.set_error("стартовать может только ГМ");
        return;
    }
    let can_start = session
        .lobby
        .as_ref()
        .map(|lobby| lobby.can_start)
        .unwrap_or(false);
    if !can_start {
        session.set_error("нужно 4+ готовых игрока");
        return;
    }
    let Some(token) = authed_token(session) else {
        return;
    };
    let Some(code) = current_code(session) else {
        session.set_error("нет активной комнаты");
        return;
    };
    session.begin_request("старт партии...");
    net::spawn_start(code, token, net_tx.0.clone());
}

fn start_refresh(session: &mut ClientSession, net_tx: &NetTx) {
    if session.is_busy() {
        return;
    }
    let Some(token) = authed_token(session) else {
        return;
    };
    let Some(code) = current_code(session) else {
        session.set_error("нет активной комнаты");
        return;
    };
    session.begin_request("обновление...");
    net::spawn_refresh_lobby(code, token, net_tx.0.clone());
}

fn start_play_action(
    msg: nnk_protocol::ClientMsg,
    status: &str,
    session: &mut ClientSession,
    net_tx: &NetTx,
) {
    if session.is_busy() {
        return;
    }
    let Some(token) = authed_token(session) else {
        return;
    };
    let Some(code) = current_code(session) else {
        session.set_error("нет активной комнаты");
        return;
    };
    session.begin_request(status);
    net::spawn_action(code, token, msg, net_tx.0.clone());
}

fn authed_token(session: &mut ClientSession) -> Option<String> {
    match session.token.clone() {
        Some(token) => Some(token),
        None => {
            session.set_error("сначала войдите");
            None
        }
    }
}

fn current_code(session: &ClientSession) -> Option<String> {
    session
        .lobby
        .as_ref()
        .map(|lobby| lobby.code.clone())
        .or_else(|| {
            let code = session.room_code_input.trim();
            (!code.is_empty()).then(|| code.to_uppercase())
        })
}

fn active_input(session: &mut ClientSession) -> &mut String {
    match session.focused {
        FocusField::Username => &mut session.username_input,
        FocusField::Password => &mut session.password_input,
        FocusField::RoomCode => &mut session.room_code_input,
    }
}

fn push_char(session: &mut ClientSession, ch: char) {
    match session.focused {
        FocusField::Username => {
            if session.username_input.chars().count() < 32 {
                session.username_input.push(ch);
                session.mark_dirty();
            }
        }
        FocusField::Password => {
            if session.password_input.chars().count() < 128 {
                session.password_input.push(ch);
                session.mark_dirty();
            }
        }
        FocusField::RoomCode => {
            if session.room_code_input.chars().count() < 6 && ch.is_ascii_alphanumeric() {
                session.room_code_input.push(ch.to_ascii_uppercase());
                session.mark_dirty();
            }
        }
    }
}
