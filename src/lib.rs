use std::time::Instant;

use egui::{Key, Modifiers, PointerButton, Pos2, RawInput, Rect};
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Mod;
use sdl2::mouse::{Cursor, MouseButton, SystemCursor};
use sdl2::video::Window;
use sdl2::VideoSubsystem;

pub struct FusedCursor {
    pub cursor: sdl2::mouse::Cursor,
    pub icon: sdl2::mouse::SystemCursor,
}

impl FusedCursor {
    pub fn new() -> Self {
        Self {
            cursor: sdl2::mouse::Cursor::from_system(sdl2::mouse::SystemCursor::Arrow).unwrap(),
            icon: sdl2::mouse::SystemCursor::Arrow,
        }
    }
}

impl Default for FusedCursor {
    fn default() -> Self {
        Self::new()
    }
}

pub fn translate_virtual_key_code(key: sdl2::keyboard::Keycode) -> Option<egui::Key> {
    use Keycode::*;

    Some(match key {
        Left => Key::ArrowLeft,
        Up => Key::ArrowUp,
        Right => Key::ArrowRight,
        Down => Key::ArrowDown,

        Escape => Key::Escape,
        Tab => Key::Tab,
        Backspace => Key::Backspace,
        Space => Key::Space,
        Return => Key::Enter,

        Insert => Key::Insert,
        Home => Key::Home,
        Delete => Key::Delete,
        End => Key::End,
        PageDown => Key::PageDown,
        PageUp => Key::PageUp,

        Kp0 | Num0 => Key::Num0,
        Kp1 | Num1 => Key::Num1,
        Kp2 | Num2 => Key::Num2,
        Kp3 | Num3 => Key::Num3,
        Kp4 | Num4 => Key::Num4,
        Kp5 | Num5 => Key::Num5,
        Kp6 | Num6 => Key::Num6,
        Kp7 | Num7 => Key::Num7,
        Kp8 | Num8 => Key::Num8,
        Kp9 | Num9 => Key::Num9,

        A => Key::A,
        B => Key::B,
        C => Key::C,
        D => Key::D,
        E => Key::E,
        F => Key::F,
        G => Key::G,
        H => Key::H,
        I => Key::I,
        J => Key::J,
        K => Key::K,
        L => Key::L,
        M => Key::M,
        N => Key::N,
        O => Key::O,
        P => Key::P,
        Q => Key::Q,
        R => Key::R,
        S => Key::S,
        T => Key::T,
        U => Key::U,
        V => Key::V,
        W => Key::W,
        X => Key::X,
        Y => Key::Y,
        Z => Key::Z,

        _ => {
            return None;
        }
    })
}

pub struct EguiSDL2State {
    start_time: std::time::Instant,
    raw_input: RawInput,
    modifiers: Modifiers,
    dpi_scaling: f32,
    mouse_pointer_position: egui::Pos2,
    fused_cursor: FusedCursor,
}

pub fn get_dpi(window: &Window, video_subsystem: &VideoSubsystem) -> f32 {
    if cfg!(not(target_os = "linux")) {
        window.drawable_size().0 as f32 / window.size().0 as f32
    } else {
        video_subsystem.display_dpi(window.display_index().unwrap_or(0))
            .map(|(_, dpi, _)| dpi / 96.0)
            .unwrap_or(1.0)
    }
}

impl EguiSDL2State {
    pub fn sdl2_input_to_egui(&mut self, window: &sdl2::video::Window, event: &sdl2::event::Event) {
        fn sdl_button_to_egui(btn: &MouseButton) -> Option<PointerButton> {
            match btn {
                MouseButton::Left => Some(egui::PointerButton::Primary),
                MouseButton::Middle => Some(egui::PointerButton::Middle),
                MouseButton::Right => Some(egui::PointerButton::Secondary),
                _ => None,
            }
        }

        use sdl2::event::Event::*;
        if event.get_window_id() != Some(window.id()) {
            return;
        }
        match event {
            // handle when window Resized and SizeChanged.
            Window { win_event, .. } => match win_event {
                WindowEvent::Resized(x, y) | sdl2::event::WindowEvent::SizeChanged(x, y) => {
                    self.update_screen_rect(window);
                }
                _ => (),
            },
            MouseButtonDown { mouse_btn, .. } => {
                if let Some(pressed) = sdl_button_to_egui(mouse_btn) {
                    self.raw_input.events.push(egui::Event::PointerButton {
                        pos: self.mouse_pointer_position,
                        button: pressed,
                        pressed: true,
                        modifiers: self.modifiers,
                    });
                }
            }
            MouseButtonUp { mouse_btn, .. } => {
                if let Some(released) = sdl_button_to_egui(mouse_btn) {
                    self.raw_input.events.push(egui::Event::PointerButton {
                        pos: self.mouse_pointer_position,
                        button: released,
                        pressed: false,
                        modifiers: self.modifiers,
                    });
                }
            }

            MouseMotion { x, y, .. } => {
                self.mouse_pointer_position =
                    egui::pos2(*x as f32 / self.dpi_scaling, *y as f32 / self.dpi_scaling);
                self.raw_input
                    .events
                    .push(egui::Event::PointerMoved(self.mouse_pointer_position));
            }

            KeyUp {
                keycode, keymod, ..
            } => {
                let key_code = match keycode {
                    Some(key_code) => key_code,
                    _ => return,
                };
                let key = match translate_virtual_key_code(*key_code) {
                    Some(key) => key,
                    _ => return,
                };
                self.modifiers = Modifiers {
                    alt: (*keymod & Mod::LALTMOD == Mod::LALTMOD)
                        || (*keymod & Mod::RALTMOD == Mod::RALTMOD),
                    ctrl: (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                        || (*keymod & Mod::RCTRLMOD == Mod::RCTRLMOD),
                    shift: (*keymod & Mod::LSHIFTMOD == Mod::LSHIFTMOD)
                        || (*keymod & Mod::RSHIFTMOD == Mod::RSHIFTMOD),
                    mac_cmd: *keymod & Mod::LGUIMOD == Mod::LGUIMOD,

                    //TOD: Test on both windows and mac
                    command: (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                        || (*keymod & Mod::LGUIMOD == Mod::LGUIMOD),
                };

                self.raw_input.events.push(egui::Event::Key {
                    key,
                    pressed: false,
                    modifiers: self.modifiers,
                });
            }

            KeyDown {
                keycode, keymod, ..
            } => {
                let key_code = match keycode {
                    Some(key_code) => key_code,
                    _ => return,
                };

                let key = match translate_virtual_key_code(*key_code) {
                    Some(key) => key,
                    _ => return,
                };
                self.modifiers = Modifiers {
                    alt: (*keymod & Mod::LALTMOD == Mod::LALTMOD)
                        || (*keymod & Mod::RALTMOD == Mod::RALTMOD),
                    ctrl: (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                        || (*keymod & Mod::RCTRLMOD == Mod::RCTRLMOD),
                    shift: (*keymod & Mod::LSHIFTMOD == Mod::LSHIFTMOD)
                        || (*keymod & Mod::RSHIFTMOD == Mod::RSHIFTMOD),
                    mac_cmd: *keymod & Mod::LGUIMOD == Mod::LGUIMOD,

                    //TOD: Test on both windows and mac
                    command: (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                        || (*keymod & Mod::LGUIMOD == Mod::LGUIMOD),
                };

                self.raw_input.events.push(egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers: self.modifiers,
                });

                if self.modifiers.command && key == Key::C {
                    // println!("copy event");
                    self.raw_input.events.push(egui::Event::Copy);
                } else if self.modifiers.command && key == Key::X {
                    // println!("cut event");
                    self.raw_input.events.push(egui::Event::Cut);
                } else if self.modifiers.command && key == Key::V {
                    // println!("paste");
                    if let Ok(contents) = window.subsystem().clipboard().clipboard_text() {
                        self.raw_input.events.push(egui::Event::Text(contents));
                    }
                }
            }

            TextInput { text, .. } => {
                self.raw_input.events.push(egui::Event::Text(text.clone()));
            }
            MouseWheel { x, y, .. } => {
                let delta = egui::vec2(*x as f32 * 8.0, *y as f32 * 8.0);
                let sdl = window.subsystem().sdl();
                // zoom:
                if sdl.keyboard().mod_state() & Mod::LCTRLMOD == Mod::LCTRLMOD
                    || sdl.keyboard().mod_state() & Mod::RCTRLMOD == Mod::RCTRLMOD
                {
                    let zoom_delta = (delta.y / 125.0).exp();
                    self.raw_input.events.push(egui::Event::Zoom(zoom_delta));
                }
                // horizontal scroll:
                else if sdl.keyboard().mod_state() & Mod::LSHIFTMOD == Mod::LSHIFTMOD
                    || sdl.keyboard().mod_state() & Mod::RSHIFTMOD == Mod::RSHIFTMOD
                {
                    self.raw_input
                        .events
                        .push(egui::Event::Scroll(egui::vec2(delta.x + delta.y, 0.0)));
                    // regular scroll:
                } else {
                    self.raw_input
                        .events
                        .push(egui::Event::Scroll(egui::vec2(delta.x, delta.y)));
                }
            }
            _ => {}
        }
    }

    pub fn update_screen_rect(&mut self, window: &Window) {
        let size = window.size();
        let rect = egui::vec2(size.0 as f32, size.1 as f32);
        self.raw_input.screen_rect = Some(Rect::from_min_size(Pos2::new(0f32, 0f32), rect));
    }

    pub fn take_egui_input(&mut self, window: &Window) -> RawInput {
        self.raw_input.time = Some(self.start_time.elapsed().as_secs_f64());

        let pixels_per_point = self.dpi_scaling;

        let drawable_size = window.drawable_size();
        let screen_size_in_points = egui::vec2(drawable_size.0 as f32, drawable_size.1 as f32) / pixels_per_point;

        self.raw_input.pixels_per_point = Some(pixels_per_point);
        self.raw_input.screen_rect =
            if screen_size_in_points.x > 0.0 && screen_size_in_points.y > 0.0 {
                Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    screen_size_in_points,
                ))
            } else {
                None
            };

        self.raw_input.take()
    }


    pub fn new(pixels_per_point: f32) -> Self {
        let raw_input = RawInput {
            pixels_per_point: Some(pixels_per_point),
            ..RawInput::default()
        };
        let modifiers = Modifiers::default();
        EguiSDL2State {
            start_time: Instant::now(),
            raw_input,
            modifiers,
            dpi_scaling: pixels_per_point,
            mouse_pointer_position: egui::Pos2::new(0.0, 0.0),
            fused_cursor: FusedCursor::new(),
        }
    }

    pub fn process_output(&mut self, window: &Window, egui_output: &egui::PlatformOutput) {
        if !egui_output.copied_text.is_empty() {
            let copied_text = egui_output.copied_text.clone();
            {
                let result = window
                    .subsystem()
                    .clipboard()
                    .set_clipboard_text(&copied_text);
                if result.is_err() {
                    dbg!("Unable to set clipboard content to SDL clipboard.");
                }
            }
        }
        EguiSDL2State::translate_cursor(&mut self.fused_cursor, egui_output.cursor_icon);
    }

    fn translate_cursor(fused: &mut FusedCursor, cursor_icon: egui::CursorIcon) {
        let tmp_icon = match cursor_icon {
            egui::CursorIcon::Crosshair => SystemCursor::Crosshair,
            egui::CursorIcon::Default => SystemCursor::Arrow,
            egui::CursorIcon::Grab => SystemCursor::Hand,
            egui::CursorIcon::Grabbing => SystemCursor::SizeAll,
            egui::CursorIcon::Move => SystemCursor::SizeAll,
            egui::CursorIcon::PointingHand => SystemCursor::Hand,
            egui::CursorIcon::ResizeHorizontal => SystemCursor::SizeWE,
            egui::CursorIcon::ResizeNeSw => SystemCursor::SizeNESW,
            egui::CursorIcon::ResizeNwSe => SystemCursor::SizeNWSE,
            egui::CursorIcon::ResizeVertical => SystemCursor::SizeNS,
            egui::CursorIcon::Text => SystemCursor::IBeam,
            egui::CursorIcon::NotAllowed | egui::CursorIcon::NoDrop => SystemCursor::No,
            egui::CursorIcon::Wait => SystemCursor::Wait,
            //There doesn't seem to be a suitable SDL equivalent...
            _ => SystemCursor::Arrow,
        };

        if tmp_icon != fused.icon {
            fused.cursor = Cursor::from_system(tmp_icon).unwrap();
            fused.icon = tmp_icon;
            fused.cursor.set();
        }
    }
}
