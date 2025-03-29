use crate::config::{ClockSettings, RgbColor};

pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_rgb_color(color: &RgbColor, opacity: f32) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
            a: (opacity * 255.0) as u8,
        }
    }
}

#[derive(PartialEq)]
pub enum TimeDigitPosition {
    Hour,
    Minute,
    Second,
}

pub struct AnimationState {
    pub current_value: u32,
    pub previous_value: Option<u32>,
    pub is_animating: bool,
    pub progress: f32,
}

pub trait GraphicsEngine {
    fn clear(&mut self) -> Result<(), String>;
    fn present(&mut self) -> Result<(), String>;

    fn draw_rect(
        &mut self,
        rect: &Rect,
        color: Color,
        border_color: Option<Color>,
        border_width: u32,
        radius: i32,
        filled: bool,
    ) -> Result<(), String>;

    fn render_digit(
        &mut self,
        value: u32,
        position: TimeDigitPosition,
        rect: &Rect,
        animation: Option<AnimationState>,
    ) -> Result<(), String>;

    fn render_am_pm_indicator(&mut self, rect: &Rect, is_pm: bool) -> Result<(), String>;

    fn handle_events(&mut self) -> Result<bool, String>;

    fn calculate_layout(&self) -> ClockLayout;

    fn get_settings(&self) -> &ClockSettings;
}

pub struct ClockLayout {
    pub hour_rect: Rect,
    pub minute_rect: Rect,
    pub second_rect: Option<Rect>,
    pub is_horizontal: bool,
    pub rect_size: u32,
    pub spacing: i32,
}
