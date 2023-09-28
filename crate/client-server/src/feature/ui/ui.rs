use std::sync::Arc;
use crate::{
  feature::{
    multiplayer::client::InitConnectionEvent,
    ui::HudPlugins,
  },
  ui::rich_text, lib::*
};
use bevy_egui::{
  egui,
  EguiContexts,
};
use bevy::prelude::*;
use epaint::Color32;
use ureq::Error;

#[derive(Resource)]
struct ApiSettings{
  url: Arc<String>,
  token: Option<Arc<String>>,
  me: Option<entity::res::Me>,
}

impl ApiSettings {
  fn new(url: Arc<String>) -> Self {
    Self {
      url,
      token: None,
      me: None,
    }
  }
}

#[derive(Resource)]
struct UiState {
  is_auth_open: bool,
  is_connection_open: bool,
  is_login_open: bool,
  is_register_open: bool,
  is_user_info_open: bool,
}

impl Default for UiState {
  fn default() -> Self {
    Self {
      is_auth_open: true,
      is_connection_open: false,
      is_login_open: false,
      is_register_open: false,
      is_user_info_open: false,
    }
  }
}

#[derive(Default, Resource)]
struct RegisterState {
  account_name: String,
  password: String,
  repeated_password: String,
}

#[derive(Default, Resource)]
struct LoginState {
  account_name: String,
  password: String,
}

#[derive(Resource)]
struct ConnectionState {
  username: String,
  addr: String
}

impl Default for ConnectionState {
  fn default() -> Self {
    Self {
      username: "noname".to_string(),
      addr: "127.0.0.1:5000".to_string(),
    }
  }
} 

// Plugin

pub struct UiPlugins {
  api_url: Arc<String>,
}

impl UiPlugins {
  pub fn by_string(api_url: Arc<String>) -> Self {
    Self {
      api_url 
    }
  }
}

/// EguiPlugin nessesarl
impl Plugin for UiPlugins {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(ApiSettings::new(format!("http://{}/", *self.api_url).into()))
      .add_plugins(HudPlugins)
      .init_resource::<ConnectionState>()
      .init_resource::<LoginState>()
      .init_resource::<RegisterState>()
      .init_resource::<UiState>()
      .add_systems(Update, (hello, login));
  }
}

fn hello (
  mut contexts: EguiContexts,
  mut ui_state: ResMut<UiState>,
  mut login_state: ResMut<LoginState>,
  mut register_state: ResMut<RegisterState>,
  mut res_api: ResMut<ApiSettings>,
  mut ev: EventWriter<InitConnectionEvent>,
  mut connection_state: ResMut<ConnectionState>,
) {
  let ctx = contexts.ctx_mut();

  let font = egui::FontId {
    family: egui::FontFamily::Monospace,
    ..default()
  };

  // let screen_center = egui::Pos2 { x: ctx.raw_input().screen_size.x * 0.5, y: ctx.raw_input().screen_size.y * 0.5 };

  if ui_state.is_auth_open {
    egui::Window::new(rich_text("Hello", &font))
      .frame(*crate::lib::egui_frame_preset::NO_SHADOW_FRAME)
      // .default_pos(screen_center)
      .collapsible(false)
      .resizable(false)
      .vscroll(true)
      .show(ctx, |ui| {
        ui.label(rich_text("Am I seeing you for the first time?", &font));
        ui.horizontal(|ui| {
          if ui.add(egui::Button::new("Login")).clicked() {
            ui_state.is_auth_open = false;
            ui_state.is_login_open = true;
          }
          if ui.add(egui::Button::new("Regester")).clicked() {
            ui_state.is_auth_open = false;
            ui_state.is_register_open = true;
          }
          // TODO remove
          if ui.add(egui::Button::new("не хочу ждать")).clicked() {
            ui_state.is_auth_open = false;
            ui_state.is_connection_open = true;
          }
        });
      });
  }

  if ui_state.is_register_open {
    egui::Window::new(rich_text("Register", &font))
      .frame(*crate::lib::egui_frame_preset::NO_SHADOW_FRAME)
      // .default_pos(screen_center)
      .collapsible(false)
      .resizable(false)
      .vscroll(true)
      .show(ctx, |ui| {
        ui.horizontal(|ui| {
          ui.label(rich_text("account name", &font));
          ui.add(egui::TextEdit::singleline(&mut register_state.account_name));
        });
        ui.horizontal(|ui| {
          ui.label(rich_text("password", &font));
          ui.add(egui::TextEdit::singleline(&mut register_state.password));
        });
        if register_state.repeated_password == register_state.password {
          ui.colored_label(Color32::RED, rich_text("repeated password", &font));
        }
        ui.horizontal(|ui| {
          ui.label(rich_text("repeated password", &font));
          ui.add(egui::TextEdit::singleline(&mut register_state.repeated_password));
        });

        ui.horizontal(|ui| {
          if ui.add(egui::Button::new("Back")).clicked() {
            ui_state.is_register_open = false;
            ui_state.is_auth_open = true;
          }
          if ui.add(egui::Button::new("Continue")).clicked() {
            ui_state.is_register_open = false;
          }
        });
      });
  }

  if ui_state.is_connection_open {
    egui::Window::new(rich_text("Connection", &font))
      .frame(*crate::lib::egui_frame_preset::NO_SHADOW_FRAME)
      // .default_pos(screen_center)
      .collapsible(false)
      .resizable(false)
      .vscroll(false)
      .show(ctx, |ui| {
        let user = res_api.me.clone().expect("user not exist?");
        ui.horizontal(|ui| {
          ui.label(rich_text(format!("username: {}", user.name.clone()), &font));
        });
        ui.horizontal(|ui| {
          ui.label(rich_text(format!("account name: {}", user.account_name), &font));
        });

        ui.horizontal(|ui| {
          ui.label(rich_text("server", &font));
          ui.add(egui::TextEdit::singleline(&mut connection_state.addr));
        });
        if ui.add(egui::Button::new("Connect")).clicked() {
          ev.send(InitConnectionEvent { addr: connection_state.addr.clone(), username: user.name });
          ui_state.is_connection_open = false;
        }
    });
  }
}

fn login(
  mut contexts: EguiContexts,
  mut ui_state: ResMut<UiState>,
  mut login_state: ResMut<LoginState>,
  mut res_api: ResMut<ApiSettings>,
) { 
  let ctx = contexts.ctx_mut();

  let font = egui::FontId {
    family: egui::FontFamily::Monospace,
    ..default()
  };

  if ui_state.is_login_open {
    egui::Window::new(rich_text("Login", &font))
      .frame(*crate::lib::egui_frame_preset::NO_SHADOW_FRAME)
      // .default_pos(screen_center)
      .collapsible(false)
      .resizable(false)
      .vscroll(false)
      .show(ctx, |ui| {
        ui.horizontal(|ui| {
          ui.label(rich_text("account name", &font));
          ui.add(egui::TextEdit::singleline(&mut login_state.account_name));
        });
        ui.horizontal(|ui| {
          ui.label(rich_text("password", &font));
          ui.add(egui::TextEdit::singleline(&mut login_state.password));
        });
        ui.horizontal(|ui| {
          if ui.add(egui::Button::new("Back")).clicked() {
            ui_state.is_login_open = false;
            ui_state.is_auth_open = true;
          }
          if ui.add(egui::Button::new("Continue")).clicked() {
            let token = match api::login(&res_api.url, &login_state.account_name, &login_state.password) {
              Ok(user) => user, 
              Err(err) => panic!("{:#?}", err),
            };
            res_api.token = Some(token.clone());

            let user = match api::me(&res_api.url, token.as_ref()) {
              Ok(user) => user,
              Err(err) => panic!("{:#?}", err), 
            };
            res_api.me = Some(user);
            ui_state.is_connection_open = true;
            ui_state.is_login_open = false;
          }
        });
      });
  }
}

// fn misc(
//   mut contexts: EguiContexts,
//   mut ui_state: ResMut<UiState>,
// ) {
//   let ctx = contexts.ctx_mut();
//
//   let font = egui::FontId {
//     family: egui::FontFamily::Monospace,
//     ..default()
//   };
//
// }
