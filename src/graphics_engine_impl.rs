use crate::config::{ClockSettings, RgbColor};
use crate::graphics_engine::{
    AnimationState, ClockLayout, Color, GraphicsEngine, Rect, TimeDigitPosition,
};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color as SdlColor;
use sdl2::rect::Rect as SdlRect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{FullscreenType, Window, WindowContext};

const FONT_SIZE_SCALE: f32 = 0.55;
const RECT_SIZE_SCALE: f32 = 0.65;

#[derive(Clone, Copy, Debug)]
enum Quadrant {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub struct Sdl2GraphicsEngine<'a> {
    sdl_context: sdl2::Sdl,
    video_subsystem: sdl2::VideoSubsystem,
    settings: &'a ClockSettings,
    canvas: Canvas<Window>,
    texture_creator: Option<TextureCreator<WindowContext>>,
    time_font: Font<'a, 'a>,
    mode_font: Font<'a, 'a>,
    ttf_context: &'a sdl2::ttf::Sdl2TtfContext,
}

impl<'a> Sdl2GraphicsEngine<'a> {
    pub fn new(
        ttf_context: &'a sdl2::ttf::Sdl2TtfContext,
        settings: &'a ClockSettings,
    ) -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let mut window = video_subsystem
            .window("Flip Clock", settings.width, settings.height)
            .position_centered()
            .resizable()
            .borderless()
            .allow_highdpi()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;
        if settings.fullscreen {
            window.set_fullscreen(FullscreenType::Desktop)?;
        }

        // Todo figure out opacity and use the background_opacity from config here.
        //window.set_opacity(0.0).unwrap();

        let mut canvas = window
            .into_canvas()
            .present_vsync()
            .accelerated()
            .build()
            .map_err(|e| e.to_string())?;
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

        canvas.set_draw_color(SdlColor::RGBA(
            settings.background_color.r,
            settings.background_color.g,
            settings.background_color.b,
            255,
        ));

        canvas.clear();
        canvas.present();

        let texture_creator = canvas.texture_creator();
        let time_font_size = (settings.height as f32 * FONT_SIZE_SCALE) as u16;
        let mode_font_size = (settings.height as f32 / 16.5) as u16;
        let time_font = ttf_context
            .load_font(&settings.font_path, time_font_size)
            .unwrap();
        let mode_font = ttf_context
            .load_font(&settings.font_path, mode_font_size)
            .unwrap();

        Ok(Sdl2GraphicsEngine {
            sdl_context,
            video_subsystem,
            settings,
            canvas,
            texture_creator: Some(texture_creator),
            time_font,
            mode_font,
            ttf_context,
        })
    }

    fn to_sdl_color(&self, color: Color) -> SdlColor {
        SdlColor::RGBA(color.r, color.g, color.b, color.a)
    }

    fn settings_color_to_sdl_color(&self, color: RgbColor) -> SdlColor {
        SdlColor::RGBA(color.r, color.g, color.b, 255)
    }

    fn to_sdl_rect(&self, rect: &Rect) -> SdlRect {
        SdlRect::new(rect.x, rect.y, rect.width, rect.height)
    }

    fn format_time(&self, time: u32) -> String {
        if self.settings.show_leading_zero {
            format!("{:02}", time)
        } else {
            format!("{}", time)
        }
    }

    fn easing_function(t: f32) -> f32 {
        let t = t * 2.0;
        if t < 1.0 {
            0.5 * t * t * t
        } else {
            let t = t - 2.0;
            0.5 * (t * t * t + 2.0)
        }
    }

    fn draw_filled_quarter_circle(
        &mut self,
        center_x: i32,
        center_y: i32,
        radius: i32,
        quadrant: Quadrant,
        color: SdlColor,
    ) -> Result<(), String> {
        self.canvas.set_draw_color(color);

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= radius * radius {
                    let pixel_x = center_x + dx;
                    let pixel_y = center_y + dy;

                    match quadrant {
                        Quadrant::TopLeft => {
                            if dx <= 0 && dy <= 0 {
                                self.canvas.draw_point((pixel_x, pixel_y))?;
                            }
                        }
                        Quadrant::TopRight => {
                            if dx >= 0 && dy <= 0 {
                                self.canvas.draw_point((pixel_x, pixel_y))?;
                            }
                        }
                        Quadrant::BottomLeft => {
                            if dx <= 0 && dy >= 0 {
                                self.canvas.draw_point((pixel_x, pixel_y))?;
                            }
                        }
                        Quadrant::BottomRight => {
                            if dx >= 0 && dy >= 0 {
                                self.canvas.draw_point((pixel_x, pixel_y))?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn draw_arc(
        &mut self,
        center_x: i32,
        center_y: i32,
        radius: i32,
        start_angle: f64,
        end_angle: f64,
        color: SdlColor,
    ) -> Result<(), String> {
        self.canvas.set_draw_color(color);

        let start_rad = start_angle.to_radians();
        let end_rad = end_angle.to_radians();

        const ANGLE_STEP: f64 = 0.01;
        let mut current_rad = start_rad;

        while current_rad <= end_rad {
            let x = center_x as f64 + radius as f64 * current_rad.cos();
            let y = center_y as f64 + radius as f64 * current_rad.sin();

            self.canvas
                .draw_point((x.round() as i32, y.round() as i32))?;
            current_rad += ANGLE_STEP;
        }

        Ok(())
    }

    fn fill_rounded_rect(
        &mut self,
        rect: &Rect,
        color: SdlColor,
        radius: i32,
    ) -> Result<(), String> {
        let sdl_rect = self.to_sdl_rect(rect);
        if radius <= 0 {
            self.canvas.set_draw_color(color);
            return self.canvas.fill_rect(sdl_rect).map_err(|e| e.to_string());
        }

        let x = rect.x;
        let y = rect.y;
        let w = rect.width as i32;
        let h = rect.height as i32;
        let r = radius.min(w / 2).min(h / 2);

        self.canvas.set_draw_color(color);
        self.canvas
            .fill_rect(SdlRect::new(x + r, y, w as u32 - 2 * r as u32, h as u32))?;
        self.canvas
            .fill_rect(SdlRect::new(x, y + r, w as u32, h as u32 - 2 * r as u32))?;
        self.draw_filled_quarter_circle(x + r, y + r, r, Quadrant::TopLeft, color)?;
        self.draw_filled_quarter_circle(x + w - r - 1, y + r, r, Quadrant::TopRight, color)?;
        self.draw_filled_quarter_circle(
            x + w - r - 1,
            y + h - r - 1,
            r,
            Quadrant::BottomRight,
            color,
        )?;
        self.draw_filled_quarter_circle(x + r, y + h - r - 1, r, Quadrant::BottomLeft, color)?;
        Ok(())
    }

    fn draw_rounded_rect_border(
        &mut self,
        rect: &Rect,
        color: SdlColor,
        radius: i32,
        thickness: u32,
    ) -> Result<(), String> {
        if radius <= 0 {
            return self.draw_rect_border(rect, color, thickness);
        }

        let x = rect.x;
        let y = rect.y;
        let w = rect.width as i32;
        let h = rect.height as i32;
        let r = radius.min(w / 2).min(h / 2);

        self.canvas.set_draw_color(color);

        for i in 0..thickness {
            let current_x = x + i as i32;
            let current_y = y + i as i32;
            let current_w = w - 2 * i as i32;
            let current_h = h - 2 * i as i32;
            let current_r = (r - i as i32).max(0);

            if current_r > 0 {
                self.draw_arc(
                    current_x + current_r,
                    current_y + current_r,
                    current_r,
                    180.0,
                    270.0,
                    color,
                )?;
                self.draw_arc(
                    current_x + current_w - current_r - 1,
                    current_y + current_r,
                    current_r,
                    270.0,
                    360.0,
                    color,
                )?;
                self.draw_arc(
                    current_x + current_w - current_r - 1,
                    current_y + current_h - current_r - 1,
                    current_r,
                    0.0,
                    90.0,
                    color,
                )?;
                self.draw_arc(
                    current_x + current_r,
                    current_y + current_h - current_r - 1,
                    current_r,
                    90.0,
                    180.0,
                    color,
                )?;

                self.draw_line(
                    current_x + current_r,
                    current_y,
                    current_x + current_w - current_r - 1,
                    current_y,
                    color,
                )?;
                self.draw_line(
                    current_x + current_w,
                    current_y + current_r,
                    current_x + current_w,
                    current_y + current_h - current_r - 1,
                    color,
                )?;
                self.draw_line(
                    current_x + current_r,
                    current_y + current_h,
                    current_x + current_w - current_r - 1,
                    current_y + current_h,
                    color,
                )?;
                self.draw_line(
                    current_x,
                    current_y + current_r,
                    current_x,
                    current_y + current_h - current_r - 1,
                    color,
                )?;
            } else {
                self.canvas.draw_rect(SdlRect::new(
                    current_x,
                    current_y,
                    current_w as u32,
                    current_h as u32,
                ))?;
            }
        }

        Ok(())
    }

    fn draw_rect_border(
        &mut self,
        rect: &Rect,
        color: SdlColor,
        thickness: u32,
    ) -> Result<(), String> {
        self.canvas.set_draw_color(color);
        for i in 0..thickness {
            self.canvas.draw_rect(SdlRect::new(
                rect.x + i as i32,
                rect.y + i as i32,
                rect.width - 2 * i,
                rect.height - 2 * i,
            ))?;
        }
        Ok(())
    }

    fn draw_line(
        &mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        color: SdlColor,
    ) -> Result<(), String> {
        self.canvas.set_draw_color(color);
        self.canvas.draw_line((x1, y1), (x2, y2))?;
        Ok(())
    }

    fn render_digits(
        &mut self,
        current_time: u32,
        past_time: Option<u32>,
        rect: &Rect,
        is_animating: bool,
        animation_progress: f32,
    ) -> Result<(), String> {
        let gap_height = self.settings.card_gap;
        let top_half_height = (rect.height as i32 - gap_height) / 2;
        let bottom_half_height = rect.height as i32 - top_half_height - gap_height;

        let time_str = self.format_time(current_time);
        let past_time_str = past_time.map(|t| self.format_time(t));

        let card_color = self.settings_color_to_sdl_color(self.settings.card_color);

        let border_color = self.settings_color_to_sdl_color(self.settings.card_border_color);

        let corner_radius = if self.settings.card_rounded_corners {
            (rect.height / 10) as i32
        } else {
            0
        };

        self.fill_rounded_rect(&rect, card_color, 66)?;
        if self.settings.card_border_size > 0 {
            self.draw_rounded_rect_border(&rect, border_color, 0, 0)?;
        }

        if !is_animating {
            {
                let current_digit_surface = self
                    .time_font
                    .render(&time_str)
                    .blended(self.settings_color_to_sdl_color(self.settings.font_color))
                    .map_err(|e| e.to_string())?;

                let current_digit_texture = self
                    .texture_creator
                    .as_ref()
                    .unwrap()
                    .create_texture_from_surface(&current_digit_surface)
                    .map_err(|e| e.to_string())?;

                let texture_query = current_digit_texture.query();
                let digit_width = texture_query.width;
                let digit_height = texture_query.height;
                let digit_x = rect.x + (rect.width as i32 - digit_width as i32) / 2;

                let top_src_full_rect = Rect::new(0, 0, digit_width, digit_height / 2);
                let bottom_src_full_rect =
                    Rect::new(0, digit_height as i32 / 2, digit_width, digit_height / 2);

                let top_dest_rect = Rect::new(
                    digit_x,
                    rect.y + (top_half_height - digit_height as i32 / 2) / 2,
                    digit_width,
                    top_half_height as u32,
                );
                let bottom_dest_rect = Rect::new(
                    digit_x,
                    rect.y
                        + top_half_height
                        + gap_height
                        + (bottom_half_height - digit_height as i32 / 2) / 2,
                    digit_width,
                    bottom_half_height as u32,
                );
                self.canvas.copy(
                    &current_digit_texture,
                    self.to_sdl_rect(&top_src_full_rect),
                    self.to_sdl_rect(&top_dest_rect),
                )?;
                self.canvas.copy(
                    &current_digit_texture,
                    self.to_sdl_rect(&bottom_src_full_rect),
                    self.to_sdl_rect(&bottom_dest_rect),
                )?;
            }

            self.canvas.set_draw_color(card_color);
            let gap_rect = Rect::new(
                rect.x,
                rect.y + top_half_height,
                rect.width,
                gap_height as u32,
            );
            self.canvas.fill_rect(self.to_sdl_rect(&gap_rect))?;
        } else if let Some(past) = past_time {
            let eased_progress = Self::easing_function(animation_progress);

            if eased_progress < 0.5 {
                let reveal_progress = eased_progress * 2.0;
                let revealed_height =
                    (top_half_height as f32 * reveal_progress).min(top_half_height as f32) as u32;

                {
                    let current_digit_surface = self
                        .time_font
                        .render(&time_str)
                        .blended(self.settings_color_to_sdl_color(self.settings.font_color))
                        .map_err(|e| e.to_string())?;
                    let current_digit_texture = self
                        .texture_creator
                        .as_ref()
                        .unwrap()
                        .create_texture_from_surface(&current_digit_surface)
                        .map_err(|e| e.to_string())?;
                    let texture_query = current_digit_texture.query();
                    let digit_width = texture_query.width;
                    let digit_x = rect.x + (rect.width as i32 - digit_width as i32) / 2;
                    let current_top_src_rect = Rect::new(0, 0, digit_width, revealed_height);
                    let current_top_dest_rect =
                        Rect::new(digit_x, rect.y, digit_width, revealed_height);
                    self.canvas.copy(
                        &current_digit_texture,
                        self.to_sdl_rect(&current_top_src_rect),
                        self.to_sdl_rect(&current_top_dest_rect),
                    )?;
                }

                {
                    let past_time_str = past_time_str.unwrap();
                    let past_digit_surface = self
                        .time_font
                        .render(&past_time_str)
                        .blended(self.settings_color_to_sdl_color(self.settings.font_color))
                        .map_err(|e| e.to_string())?;
                    let past_digit_texture = self
                        .texture_creator
                        .as_ref()
                        .unwrap()
                        .create_texture_from_surface(&past_digit_surface)
                        .map_err(|e| e.to_string())?;
                    let texture_query = past_digit_texture.query();
                    let digit_width = texture_query.width;
                    let digit_x = rect.x + (rect.width as i32 - digit_width as i32) / 2;
                    let top_src_full_rect = Rect::new(0, 0, digit_width, texture_query.height / 2);
                    let shrink_progress = 1.0 - eased_progress * 2.0;
                    let current_top_past_height =
                        (top_half_height as f32 * shrink_progress).max(0.0) as u32;
                    let top_past_dest_rect = Rect::new(
                        digit_x,
                        rect.y + top_half_height as i32 - current_top_past_height as i32,
                        digit_width,
                        current_top_past_height,
                    );
                    self.canvas.copy(
                        &past_digit_texture,
                        self.to_sdl_rect(&top_src_full_rect),
                        self.to_sdl_rect(&top_past_dest_rect),
                    )?;
                }

                {
                    let past_time_str = self.format_time(past_time.clone().unwrap());
                    let past_digit_surface = self
                        .time_font
                        .render(&past_time_str)
                        .blended(self.settings_color_to_sdl_color(self.settings.font_color))
                        .map_err(|e| e.to_string())?;
                    let past_digit_texture = self
                        .texture_creator
                        .as_ref()
                        .unwrap()
                        .create_texture_from_surface(&past_digit_surface)
                        .map_err(|e| e.to_string())?;
                    let texture_query = past_digit_texture.query();
                    let digit_width = texture_query.width;
                    let digit_x = rect.x + (rect.width as i32 - digit_width as i32) / 2;
                    let bottom_src_full_rect = Rect::new(
                        0,
                        texture_query.height as i32 / 2,
                        digit_width,
                        texture_query.height / 2,
                    );
                    let bottom_past_dest_rect = Rect::new(
                        digit_x,
                        rect.y + top_half_height + gap_height,
                        digit_width,
                        bottom_half_height as u32,
                    );
                    self.canvas.copy(
                        &past_digit_texture,
                        self.to_sdl_rect(&bottom_src_full_rect),
                        self.to_sdl_rect(&bottom_past_dest_rect),
                    )?;
                }
            } else {
                {
                    let current_digit_surface = self
                        .time_font
                        .render(&time_str)
                        .blended(self.settings_color_to_sdl_color(self.settings.font_color))
                        .map_err(|e| e.to_string())?;
                    let current_digit_texture = self
                        .texture_creator
                        .as_ref()
                        .unwrap()
                        .create_texture_from_surface(&current_digit_surface)
                        .map_err(|e| e.to_string())?;
                    let texture_query = current_digit_texture.query();
                    let digit_width = texture_query.width;
                    let digit_x = rect.x + (rect.width as i32 - digit_width as i32) / 2;
                    let top_src_full_rect = Rect::new(0, 0, digit_width, texture_query.height / 2);
                    let top_dest_rect =
                        Rect::new(digit_x, rect.y, digit_width, top_half_height as u32);
                    self.canvas.copy(
                        &current_digit_texture,
                        self.to_sdl_rect(&top_src_full_rect),
                        self.to_sdl_rect(&top_dest_rect),
                    )?;
                }

                let bottom_flip_progress = (eased_progress - 0.5) * 2.0;

                {
                    let past_time_str = past_time_str.unwrap();
                    let past_digit_surface = self
                        .time_font
                        .render(&past_time_str)
                        .blended(self.settings_color_to_sdl_color(self.settings.font_color))
                        .map_err(|e| e.to_string())?;
                    let past_digit_texture = self
                        .texture_creator
                        .as_ref()
                        .unwrap()
                        .create_texture_from_surface(&past_digit_surface)
                        .map_err(|e| e.to_string())?;
                    let texture_query = past_digit_texture.query();
                    let digit_width = texture_query.width;
                    let digit_x = rect.x + (rect.width as i32 - digit_width as i32) / 2;
                    let bottom_src_full_rect = Rect::new(
                        0,
                        texture_query.height as i32 / 2,
                        digit_width,
                        texture_query.height / 2,
                    );
                    let old_bottom_visible_height =
                        (bottom_half_height as f32 * (1.0 - bottom_flip_progress)).max(0.0) as u32;
                    let old_bottom_src_rect = Rect::new(
                        0,
                        texture_query.height as i32 / 2
                            + (bottom_half_height as u32 - old_bottom_visible_height) as i32,
                        digit_width,
                        old_bottom_visible_height,
                    );
                    let old_bottom_dest_rect = Rect::new(
                        digit_x,
                        rect.y
                            + top_half_height
                            + gap_height
                            + (bottom_half_height as i32 - old_bottom_visible_height as i32),
                        digit_width,
                        old_bottom_visible_height,
                    );
                    self.canvas.copy(
                        &past_digit_texture,
                        self.to_sdl_rect(&old_bottom_src_rect),
                        self.to_sdl_rect(&old_bottom_dest_rect),
                    )?;
                }

                {
                    let current_digit_surface = self
                        .time_font
                        .render(&time_str)
                        .blended(self.settings_color_to_sdl_color(self.settings.font_color))
                        .map_err(|e| e.to_string())?;
                    let current_digit_texture = self
                        .texture_creator
                        .as_ref()
                        .unwrap()
                        .create_texture_from_surface(&current_digit_surface)
                        .map_err(|e| e.to_string())?;
                    let texture_query = current_digit_texture.query();
                    let digit_width = texture_query.width;
                    let digit_x = rect.x + (rect.width as i32 - digit_width as i32) / 2;
                    let bottom_src_full_rect = Rect::new(
                        0,
                        texture_query.height as i32 / 2,
                        digit_width,
                        texture_query.height / 2,
                    );
                    let new_bottom_revealed_height =
                        (bottom_half_height as f32 * bottom_flip_progress).max(0.0) as u32;
                    let new_bottom_src_rect = Rect::new(
                        0,
                        texture_query.height as i32 / 2,
                        digit_width,
                        new_bottom_revealed_height,
                    );
                    let new_bottom_dest_rect = Rect::new(
                        digit_x,
                        rect.y + top_half_height + gap_height,
                        digit_width,
                        new_bottom_revealed_height,
                    );
                    self.canvas.copy(
                        &current_digit_texture,
                        self.to_sdl_rect(&new_bottom_src_rect),
                        self.to_sdl_rect(&new_bottom_dest_rect),
                    )?;
                }
            }

            self.canvas.set_draw_color(card_color);
            let gap_rect = Rect::new(
                rect.x,
                rect.y + top_half_height,
                rect.width,
                gap_height as u32,
            );
            self.canvas.fill_rect(self.to_sdl_rect(&gap_rect))?;
        }

        Ok(())
    }

    fn render_am_pm(&mut self, rect: SdlRect, is_pm: bool) -> Result<(), String> {
        if !self.settings.use_24hour {
            let am_pm_text = if is_pm { "PM" } else { "AM" };
            let am_pm_surface = self
                .mode_font
                .render(am_pm_text)
                .blended(SdlColor::RGB(
                    self.settings.font_color.r,
                    self.settings.font_color.g,
                    self.settings.font_color.b,
                ))
                .map_err(|e| e.to_string())?;

            let am_pm_texture = self
                .texture_creator
                .as_ref()
                .unwrap()
                .create_texture_from_surface(&am_pm_surface)
                .map_err(|e| e.to_string())?;

            let texture_query = am_pm_texture.query();
            let x = rect.x() + rect.width() as i32 / 2 - texture_query.width as i32 / 2;
            let y = if is_pm {
                rect.y() + rect.height() as i32 - texture_query.height as i32 - 10
            } else {
                rect.y() + 10
            };

            self.canvas.copy(
                &am_pm_texture,
                None,
                self.to_sdl_rect(&Rect::new(x, y, texture_query.width, texture_query.height)),
            )?;
        }
        Ok(())
    }
}

impl<'a> GraphicsEngine for Sdl2GraphicsEngine<'a> {
    fn clear(&mut self) -> Result<(), String> {
        self.canvas.set_draw_color(SdlColor::RGBA(
            self.settings.background_color.r,
            self.settings.background_color.g,
            self.settings.background_color.b,
            255,
        ));
        self.canvas.clear();
        Ok(())
    }

    fn present(&mut self) -> Result<(), String> {
        self.canvas.present();
        Ok(())
    }

    fn draw_rect(
        &mut self,
        rect: &Rect,
        color: Color,
        border_color: Option<Color>,
        border_width: u32,
        radius: i32,
        filled: bool,
    ) -> Result<(), String> {
        let sdl_color = self.to_sdl_color(color);

        if filled {
            self.fill_rounded_rect(rect, sdl_color, radius)?;
        }

        if let Some(border_color) = border_color {
            if border_width > 0 {
                let sdl_border_color = self.to_sdl_color(border_color);
                self.draw_rounded_rect_border(rect, sdl_border_color, radius, border_width)?;
            }
        }

        Ok(())
    }

    fn render_digit(
        &mut self,
        value: u32,
        position: TimeDigitPosition,
        rect: &Rect,
        animation: Option<AnimationState>,
    ) -> Result<(), String> {
        // Convert rect to SDL2 rect
        let sdl_rect = sdl2::rect::Rect::new(rect.x, rect.y, rect.width, rect.height);

        // Use the existing render_digits implementation but adapted to the new interface
        let past_time = animation.as_ref().and_then(|a| a.previous_value);
        let is_animating = animation.as_ref().map(|a| a.is_animating).unwrap_or(false);
        let progress = animation.as_ref().map(|a| a.progress).unwrap_or(0.0);

        self.render_digits(value, past_time, rect, is_animating, progress)
    }

    fn render_am_pm_indicator(&mut self, rect: &Rect, is_pm: bool) -> Result<(), String> {
        // Convert rect to SDL2 rect
        let sdl_rect = sdl2::rect::Rect::new(rect.x, rect.y, rect.width, rect.height);

        self.render_am_pm(sdl_rect, is_pm)
    }

    fn handle_events(&mut self) -> Result<bool, String> {
        let mut event_pump = self.sdl_context.event_pump()?;
        let mut event_count = 0; // mouse movement seems a bit aggressive so after 5 events.

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape) | Some(Keycode::Return) | Some(Keycode::Space),
                    ..
                } => return Ok(true),
                Event::KeyDown { .. }
                | Event::MouseButtonDown { .. }
                | Event::MouseButtonUp { .. }
                | Event::MouseWheel { .. } => {
                    if self.settings.close_on_any_input {
                        return Ok(true);
                    }
                    return Ok(false);
                }
                Event::MouseMotion { .. } => {
                    event_count += 1;
                    if event_count > 4 {
                        return Ok(true);
                    }
                    return Ok(false);
                }
                _ => return Ok(false),
            }
        }

        Ok(false)
    }

    fn calculate_layout(&self) -> ClockLayout {
        let is_horizontal = self.settings.width > self.settings.height;
        let rect_size = if is_horizontal {
            (self.settings.height as f32 * RECT_SIZE_SCALE) as u32 // Uses height for horizontal
        } else {
            (self.settings.width as f32 * RECT_SIZE_SCALE) as u32 // Uses width for vertical
        };

        let spacing = if is_horizontal {
            (self.settings.width as f32 * 0.031) as i32 // Uses width for horizontal spacing
        } else {
            (self.settings.height as f32 * 0.031) as i32 // Uses height for vertical spacing
        };

        let hour_rect = if is_horizontal {
            Rect::new(
                ((self.settings.width as i32 - spacing - (rect_size as i32 * 2)) / 2) as i32, // Centers horizontally
                ((self.settings.height as i32 - rect_size as i32) / 2) as i32, // Centers vertically
                rect_size,
                rect_size,
            )
        } else {
            Rect::new(
                ((self.settings.width as i32 - rect_size as i32) / 2) as i32, // Centers horizontally
                ((self.settings.height as i32 - spacing - (rect_size as i32 * 2)) / 2) as i32, // Centers vertically
                rect_size,
                rect_size,
            )
        };

        let minute_rect = if is_horizontal {
            Rect::new(
                hour_rect.x + rect_size as i32 + spacing, // Positioned to the right of hour
                hour_rect.y,
                rect_size,
                rect_size,
            )
        } else {
            Rect::new(
                hour_rect.x,
                hour_rect.y + rect_size as i32 + spacing, // Positioned below hour
                rect_size,
                rect_size,
            )
        };

        let seconds_rect = if self.settings.show_seconds {
            Some(if is_horizontal {
                Rect::new(
                    minute_rect.x + rect_size as i32 + spacing, // Positioned to the right of minute
                    minute_rect.y,
                    rect_size,
                    rect_size,
                )
            } else {
                Rect::new(
                    minute_rect.x,
                    minute_rect.y + rect_size as i32 + spacing, // Positioned below minute
                    rect_size,
                    rect_size,
                )
            })
        } else {
            None
        };

        ClockLayout {
            hour_rect,
            minute_rect,
            second_rect: seconds_rect,
            is_horizontal,
            rect_size,
            spacing,
        }
    }

    fn get_settings(&self) -> &ClockSettings {
        &self.settings
    }
}
