#![allow(dead_code)]

use glam::Vec2;
use std::collections::HashMap;
pub use winit::event::{ElementState, VirtualKeyCode};
use winit::{
    dpi::PhysicalPosition,
    event::{Event, WindowEvent, KeyboardInput},
};
use gilrs::{Gilrs, Button, Axis, EventType};

// Gamepad button mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    A,
    B,
    X,
    Y,
    LeftBumper,
    RightBumper,
    Back,
    Start,
    Guide,
    LeftStick,
    RightStick,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    LeftTrigger,
    RightTrigger,
}

// Gamepad axis mapping  
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}

#[derive(Clone)]
pub struct GamepadButtonState {
    pub ticks: u32,
    pub value: f32,
}

#[derive(Default, Clone)]
pub struct GamepadState {
    buttons_down: HashMap<GamepadButton, GamepadButtonState>,
    axes: HashMap<GamepadAxis, f32>,
    pub connected: bool,
}

impl GamepadButton {
    fn from_gilrs(button: Button) -> Option<Self> {
        match button {
            Button::South => Some(GamepadButton::A),
            Button::East => Some(GamepadButton::B),
            Button::West => Some(GamepadButton::X),
            Button::North => Some(GamepadButton::Y),
            Button::LeftTrigger => Some(GamepadButton::LeftBumper),
            Button::RightTrigger => Some(GamepadButton::RightBumper),
            Button::Select => Some(GamepadButton::Back),
            Button::Start => Some(GamepadButton::Start),
            Button::Mode => Some(GamepadButton::Guide),
            Button::LeftThumb => Some(GamepadButton::LeftStick),
            Button::RightThumb => Some(GamepadButton::RightStick),
            Button::DPadUp => Some(GamepadButton::DPadUp),
            Button::DPadDown => Some(GamepadButton::DPadDown),
            Button::DPadLeft => Some(GamepadButton::DPadLeft),
            Button::DPadRight => Some(GamepadButton::DPadRight),
            _ => None,
        }
    }
}

impl GamepadAxis {
    fn from_gilrs(axis: Axis) -> Option<Self> {
        match axis {
            Axis::LeftStickX => Some(GamepadAxis::LeftStickX),
            Axis::LeftStickY => Some(GamepadAxis::LeftStickY),
            Axis::RightStickX => Some(GamepadAxis::RightStickX),
            Axis::RightStickY => Some(GamepadAxis::RightStickY),
            Axis::LeftZ => Some(GamepadAxis::LeftTrigger),
            Axis::RightZ => Some(GamepadAxis::RightTrigger),
            _ => None,
        }
    }
}

impl GamepadState {
    pub fn is_button_down(&self, button: GamepadButton) -> bool {
        self.get_button_down(button).is_some()
    }

    pub fn was_button_just_pressed(&self, button: GamepadButton) -> bool {
        self.get_button_down(button).map(|s| s.ticks == 1).unwrap_or_default()
    }

    pub fn get_button_down(&self, button: GamepadButton) -> Option<&GamepadButtonState> {
        self.buttons_down.get(&button)
    }

    pub fn get_button_value(&self, button: GamepadButton) -> f32 {
        self.get_button_down(button).map(|s| s.value).unwrap_or(0.0)
    }

    pub fn get_axis(&self, axis: GamepadAxis) -> f32 {
        self.axes.get(&axis).copied().unwrap_or(0.0)
    }

    pub fn set_button(&mut self, button: GamepadButton, pressed: bool, value: f32) {
        if pressed {
            self.buttons_down.entry(button).or_insert(GamepadButtonState { ticks: 0, value: 0.0 }).value = value;
        } else {
            self.buttons_down.remove(&button);
        }
    }

    pub fn set_axis(&mut self, axis: GamepadAxis, value: f32) {
        // Apply deadzone
        let deadzone = 0.1;
        let final_value = if value.abs() < deadzone { 0.0 } else { value };
        self.axes.insert(axis, final_value);
    }

    pub fn update_ticks(&mut self) {
        for state in self.buttons_down.values_mut() {
            state.ticks += 1;
        }
    }

    pub fn update_from_gilrs(&mut self, gilrs: &mut Gilrs) {
        self.connected = false;
        
        // Check for any connected gamepad
        for (_id, gamepad) in gilrs.gamepads() {
            if gamepad.is_connected() {
                self.connected = true;
                break;
            }
        }

        if !self.connected {
            self.buttons_down.clear();
            self.axes.clear();
            return;
        }

        // Process events
        while let Some(gilrs::Event { id: _, event, time: _ }) = gilrs.next_event() {
            match event {
                EventType::ButtonPressed(button, _) => {
                    if let Some(gamepad_button) = GamepadButton::from_gilrs(button) {
                        self.set_button(gamepad_button, true, 1.0);
                    }
                }
                EventType::ButtonReleased(button, _) => {
                    if let Some(gamepad_button) = GamepadButton::from_gilrs(button) {
                        self.set_button(gamepad_button, false, 0.0);
                    }
                }
                EventType::AxisChanged(axis, value, _) => {
                    if let Some(gamepad_axis) = GamepadAxis::from_gilrs(axis) {
                        self.set_axis(gamepad_axis, value);
                    }
                }
                _ => {}
            }
        }

        // Update trigger buttons based on axis values
        let left_trigger = self.get_axis(GamepadAxis::LeftTrigger);
        let right_trigger = self.get_axis(GamepadAxis::RightTrigger);
        
        if left_trigger > 0.1 {
            self.set_button(GamepadButton::LeftTrigger, true, left_trigger);
        } else {
            self.set_button(GamepadButton::LeftTrigger, false, 0.0);
        }
        
        if right_trigger > 0.1 {
            self.set_button(GamepadButton::RightTrigger, true, right_trigger);
        } else {
            self.set_button(GamepadButton::RightTrigger, false, 0.0);
        }
    }
}

#[derive(Clone)]
pub struct KeyState {
    pub ticks: u32,
}

#[derive(Default, Clone)]
pub struct KeyboardState {
    keys_down: HashMap<VirtualKeyCode, KeyState>,
}

impl KeyboardState {
    pub fn is_down(&self, key: VirtualKeyCode) -> bool {
        self.get_down(key).is_some()
    }

    pub fn was_just_pressed(&self, key: VirtualKeyCode) -> bool {
        self.get_down(key).map(|s| s.ticks == 1).unwrap_or_default()
    }

    pub fn get_down(&self, key: VirtualKeyCode) -> Option<&KeyState> {
        self.keys_down.get(&key)
    }

    pub fn update(&mut self, events: &[Event<'_, ()>]) {
        for event in events {
            if let Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } = event
            {
                if let Some(vk) = input.virtual_keycode {
                    if input.state == ElementState::Pressed {
                        self.keys_down.entry(vk).or_insert(KeyState { ticks: 0 });
                    } else {
                        self.keys_down.remove(&vk);
                    }
                }
            }
        }

        for ks in self.keys_down.values_mut() {
            ks.ticks += 1;
        }
    }
}

#[derive(Clone, Copy)]
pub struct MouseState {
    pub physical_position: PhysicalPosition<f64>,
    pub delta: Vec2,
    pub buttons_held: u32,
    pub buttons_pressed: u32,
    pub buttons_released: u32,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            physical_position: PhysicalPosition { x: 0.0, y: 0.0 },
            delta: Vec2::ZERO,
            buttons_held: 0,
            buttons_pressed: 0,
            buttons_released: 0,
        }
    }
}

impl MouseState {
    pub fn update(&mut self, events: &[Event<'_, ()>]) {
        self.buttons_pressed = 0;
        self.buttons_released = 0;
        self.delta = Vec2::ZERO;

        for event in events {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        self.physical_position = *position;
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        let button_id = match button {
                            winit::event::MouseButton::Left => 0,
                            winit::event::MouseButton::Middle => 1,
                            winit::event::MouseButton::Right => 2,
                            _ => 0,
                        };

                        if let ElementState::Pressed = state {
                            self.buttons_held |= 1 << button_id;
                            self.buttons_pressed |= 1 << button_id;
                        } else {
                            self.buttons_held &= !(1 << button_id);
                            self.buttons_released |= 1 << button_id;
                        }
                    }
                    _ => (),
                },
                Event::DeviceEvent {
                    device_id: _,
                    event: winit::event::DeviceEvent::MouseMotion { delta },
                } => {
                    self.delta.x += delta.0 as f32;
                    self.delta.y += delta.1 as f32;
                }
                _ => (),
            }
        }
    }
}

pub type InputAxis = &'static str;

pub struct KeyMap {
    axis: InputAxis,
    multiplier: f32,
    activation_time: f32,
}

impl KeyMap {
    pub fn new(axis: InputAxis, multiplier: f32) -> Self {
        Self {
            axis,
            multiplier,
            activation_time: 0.15,
        }
    }

    pub fn activation_time(mut self, value: f32) -> Self {
        self.activation_time = value;
        self
    }
}

// GamepadMap para botones
pub struct GamepadButtonMap {
    axis: InputAxis,
    multiplier: f32,
    activation_time: f32,
}

impl GamepadButtonMap {
    pub fn new(axis: InputAxis, multiplier: f32) -> Self {
        Self {
            axis,
            multiplier,
            activation_time: 0.15,
        }
    }

    pub fn activation_time(mut self, value: f32) -> Self {
        self.activation_time = value;
        self
    }
}

// GamepadMap para ejes
pub struct GamepadAxisMap {
    axis: InputAxis,
    multiplier: f32,
}

impl GamepadAxisMap {
    pub fn new(axis: InputAxis, multiplier: f32) -> Self {
        Self {
            axis,
            multiplier,
        }
    }
}

struct KeyMapState {
    map: KeyMap,
    activation: f32,
}

struct GamepadButtonMapState {
    map: GamepadButtonMap,
    activation: f32,
}

pub struct KeyboardMap {
    bindings: Vec<(VirtualKeyCode, KeyMapState)>,
}

pub struct GamepadMap {
    button_bindings: Vec<(GamepadButton, GamepadButtonMapState)>,
    axis_bindings: Vec<(GamepadAxis, GamepadAxisMap)>,
}

impl Default for KeyboardMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GamepadMap {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardMap {
    pub fn new() -> Self {
        Self {
            bindings: Default::default(),
        }
    }

    pub fn bind(mut self, key: VirtualKeyCode, map: KeyMap) -> Self {
        self.bindings.push((
            key,
            KeyMapState {
                map,
                activation: 0.0,
            },
        ));
        self
    }

    pub fn map(&mut self, keyboard: &KeyboardState, dt: f32) -> HashMap<InputAxis, f32> {
        let mut result: HashMap<InputAxis, f32> = HashMap::new();

        for (vk, s) in &mut self.bindings {
            #[allow(clippy::collapsible_else_if)]
            if s.map.activation_time > 1e-10 {
                let change = if keyboard.is_down(*vk) { dt } else { -dt };
                s.activation = (s.activation + change / s.map.activation_time).clamp(0.0, 1.0);
            } else {
                if keyboard.is_down(*vk) {
                    s.activation = 1.0;
                } else {
                    s.activation = 0.0;
                }
            }

            *result.entry(s.map.axis).or_default() += s.activation.powi(2) * s.map.multiplier;
        }

        for value in result.values_mut() {
            *value = value.clamp(-1.0, 1.0);
        }

        result
    }
}

impl GamepadMap {
    pub fn new() -> Self {
        Self {
            button_bindings: Default::default(),
            axis_bindings: Default::default(),
        }
    }

    pub fn bind_button(mut self, button: GamepadButton, map: GamepadButtonMap) -> Self {
        self.button_bindings.push((
            button,
            GamepadButtonMapState {
                map,
                activation: 0.0,
            },
        ));
        self
    }

    pub fn bind_axis(mut self, axis: GamepadAxis, map: GamepadAxisMap) -> Self {
        self.axis_bindings.push((axis, map));
        self
    }

    pub fn map(&mut self, gamepad: &GamepadState, dt: f32) -> HashMap<InputAxis, f32> {
        let mut result: HashMap<InputAxis, f32> = HashMap::new();

        if !gamepad.connected {
            return result;
        }

        // Handle button bindings
        for (button, s) in &mut self.button_bindings {
            #[allow(clippy::collapsible_else_if)]
            if s.map.activation_time > 1e-10 {
                let change = if gamepad.is_button_down(*button) { dt } else { -dt };
                s.activation = (s.activation + change / s.map.activation_time).clamp(0.0, 1.0);
            } else {
                if gamepad.is_button_down(*button) {
                    s.activation = 1.0;
                } else {
                    s.activation = 0.0;
                }
            }

            *result.entry(s.map.axis).or_default() += s.activation.powi(2) * s.map.multiplier;
        }

        // Handle axis bindings
        for (axis, map) in &self.axis_bindings {
            let axis_value = gamepad.get_axis(*axis);
            *result.entry(map.axis).or_default() += axis_value * map.multiplier;
        }

        for value in result.values_mut() {
            *value = value.clamp(-1.0, 1.0);
        }

        result
    }
}
