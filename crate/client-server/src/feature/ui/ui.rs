use std::sync::Arc;
use crate::{
  feature::{
    multiplayer::client::InitConnectionEvent,
    ui::{
      debug::UiDebugState,
      HudPlugins,
    }
  },
  ui::rich_text
};
use bevy_egui::{
  egui,
  EguiContexts,
};
use bevy::{
  prelude::*,
  tasks::AsyncComputeTaskPool,
  diagnostic::DiagnosticsStore,
};
use serde_json::json;
use ureq::Error;
use egui::Align;

#[derive(Resource)]
struct ApiSettings{
  url: Arc<String>,
  token: Option<Arc<String>>,
}

impl ApiSettings {
  fn new(url: Arc<String>) -> Self {
    Self {
      url,
      token: None
    }
  }
}

#[derive(Resource)]
struct UiState {
  is_hello: bool,
  is_connection_open: bool,
  is_login: bool,
  is_register: bool,
}

impl Default for UiState {
  fn default() -> Self {
    Self {
      is_hello: true,
      is_connection_open: false,
      is_login: false,
      is_register: false,
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

// Events

#[derive(Event)]
struct LoginSuccessEvent {
  token: Arc<String>,
} 

#[derive(Event)]
struct LoginErrorEvent {
  message: String,
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

/// EguiPlugin nessesarly
impl Plugin for UiPlugins {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(ApiSettings::new(self.api_url.clone()))
      .add_plugins(HudPlugins)
      .add_event::<LoginSuccessEvent>()
      .add_event::<LoginErrorEvent>()
      .init_resource::<ConnectionState>()
      .init_resource::<LoginState>()
      .init_resource::<RegisterState>()
      .init_resource::<UiState>()
      .add_systems(Update, (hello, misc));
  }
}

fn login_events_handler(
  mut login_success_event: EventReader<LoginSuccessEvent>,
  mut login_error_event: EventReader<LoginErrorEvent>,
  ) {
  for success in login_success_event.iter() {

  }

  for error in login_error_event.iter() {

  }
}

fn hello (
  mut contexts: EguiContexts,
  mut ui_state: ResMut<UiState>,
  mut login_state: ResMut<LoginState>,
  mut register_state: ResMut<RegisterState>,
  mut login_success_event: EventWriter<LoginSuccessEvent>,
  mut login_error_event: EventWriter<LoginErrorEvent>,
  mut res_api: ResMut<ApiSettings>,
) {
  let ctx = contexts.ctx_mut();

  let font = egui::FontId {
    family: egui::FontFamily::Monospace,
    ..default()
  };

  // let screen_center = egui::Pos2 { x: ctx.raw_input().screen_size.x * 0.5, y: ctx.raw_input().screen_size.y * 0.5 };

  if ui_state.is_hello {
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
            ui_state.is_hello = false;
            ui_state.is_login = true;
          }
          if ui.add(egui::Button::new("Regester")).clicked() {
            ui_state.is_hello = false;
            ui_state.is_register = true;
          }
          // TODO remove
          if ui.add(egui::Button::new("не хочу ждать")).clicked() {
            ui_state.is_hello = false;
            ui_state.is_connection_open = true;
          }
        });
      });
  }

  if ui_state.is_register {
    egui::Window::new(rich_text("Login", &font))
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
        ui.horizontal(|ui| {
          ui.label(rich_text("password", &font));
          ui.add(egui::TextEdit::singleline(&mut register_state.repeated_password));
        });

        ui.horizontal(|ui| {
          if ui.add(egui::Button::new("Back")).clicked() {
            ui_state.is_register = false;
            ui_state.is_hello = true;
          }
          if ui.add(egui::Button::new("Continue")).clicked() {
            ui_state.is_register = false;
          }
        });
      });
  }

  if ui_state.is_login {
    egui::Window::new(rich_text("Login", &font))
      .frame(*crate::lib::egui_frame_preset::NO_SHADOW_FRAME)
      // .default_pos(screen_center)
      .collapsible(false)
      .resizable(false)
      .vscroll(true)
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
            ui_state.is_login = false;
            ui_state.is_hello = true;
          }
          if ui.add(egui::Button::new("Continue")).clicked() {
            let url = format!("http:/{}//user/login", res_api.url);
            let json_body = json!({
              "account_name": login_state.account_name,
              "password": login_state.password, 
            });
            
            let resp = ureq::post(url.as_str())
              .set("Content-Type", "application/json")
              .send_json(json_body);

            match resp {
              Ok(body) => {
                let body = body.into_string().expect("твой код говно");
                res_api.token = Some(body.into());
                ui_state.is_login = false;
              },
              Err(Error::Status(code, response)) => {
                println!("Error: {}, {:#?}", code, response);
              },
              Err(err) => {
                println!("Error: {}", err);
              }
            };
          }
        });
      });
  }
}

fn misc(
  mut contexts: EguiContexts,
  mut connection_state: ResMut<ConnectionState>,
  mut ui_state: ResMut<UiState>,
  mut ev: EventWriter<InitConnectionEvent>,
) {
  let ctx = contexts.ctx_mut();

  let font = egui::FontId {
    family: egui::FontFamily::Monospace,
    ..default()
  };

  if ui_state.is_connection_open {
    egui::Window::new(rich_text("Connection", &font))
      .frame(*crate::lib::egui_frame_preset::NO_SHADOW_FRAME)
      // .default_pos(screen_center)
      .collapsible(false)
      .resizable(false)
      .vscroll(true)
      .show(ctx, |ui| {
        ui.horizontal(|ui| {
          ui.label(rich_text("username", &font));
          ui.add(egui::TextEdit::singleline(&mut connection_state.username));
        });
        ui.horizontal(|ui| {
          ui.label(rich_text("server", &font));
          ui.add(egui::TextEdit::singleline(&mut connection_state.addr));
        });
        if ui.add(egui::Button::new("Connect")).clicked() {
          ev.send(InitConnectionEvent { addr: connection_state.addr.clone(), username: connection_state.username.clone() });
          ui_state.is_connection_open = false;
        }
      });
  }
}
