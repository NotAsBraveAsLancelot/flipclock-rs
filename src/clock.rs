use crate::config::ClockSettings;
use crate::graphics_engine::{
    AnimationState, GraphicsEngine, TimeDigitPosition,
};
use chrono::{Local, Timelike};
use std::time::{Duration, Instant};

pub struct FlipClock<E: GraphicsEngine> {
    engine: E,
    settings: ClockSettings,
    past_hour: Option<u32>,
    past_minute: Option<u32>,
    past_second: Option<u32>,
}

impl<E: GraphicsEngine> FlipClock<E> {
    pub fn new(engine: E, settings: &ClockSettings) -> Self {
        FlipClock {
            engine,
            settings: settings.clone(),
            past_hour: None,
            past_minute: None,
            past_second: None,
        }
    }

    fn get_current_time(&self) -> (u32, u32, u32, bool) {
        let time = Local::now();
        let settings = self.engine.get_settings();

        let (hour, am_pm) = if settings.use_24hour {
            (time.hour(), false)
        } else {
            let (am_pm, hour) = time.hour12(); // bool is false in the afternoon.
            (hour, am_pm)
        };

        let minute = time.minute();
        let second = time.second();

        (hour, minute, second, am_pm)
    }
    fn render(&mut self) -> Result<(), String> {
        let (hour, minute, second, is_pm) = self.get_current_time();
        let settings = self.engine.get_settings();
        let animate_flip = settings.animate_flip;
        let animation_duration_ms = settings.animation_duration_ms;
        let show_seconds = settings.show_seconds;

        let layout = self.engine.calculate_layout();
        let hour_rect = layout.hour_rect;
        let minute_rect = layout.minute_rect;
        let second_rect_option = layout.second_rect;

        let is_hour_changed = self.past_hour.map_or(true, |past| past != hour);
        let is_minute_changed = self.past_minute.map_or(true, |past| past != minute);
        let is_second_changed = self.past_second.map_or(true, |past| past != second);

        self.engine.clear()?; // Clear the entire canvas at the beginning of each full render cycle

        if animate_flip {
            let duration = Duration::from_millis(animation_duration_ms as u64);
            let animation_start_time = Instant::now(); // Start time for the entire animation sequence

            while animation_start_time.elapsed() < duration {
                let current_animation_progress =
                    animation_start_time.elapsed().as_secs_f32() / duration.as_secs_f32();

                self.engine.clear()?; // Clear for each frame of the animation

                // Render hour
                self.engine.render_digit(
                    hour,
                    TimeDigitPosition::Hour,
                    &hour_rect,
                    if is_hour_changed {
                        Some(AnimationState {
                            current_value: hour,
                            previous_value: self.past_hour,
                            is_animating: true,
                            progress: current_animation_progress.clamp(0.0, 1.0),
                        })
                    } else {
                        None // No animation needed
                    },
                )?;

                // Render minute
                self.engine.render_digit(
                    minute,
                    TimeDigitPosition::Minute,
                    &minute_rect,
                    if is_minute_changed {
                        Some(AnimationState {
                            current_value: minute,
                            previous_value: self.past_minute,
                            is_animating: true,
                            progress: current_animation_progress.clamp(0.0, 1.0),
                        })
                    } else {
                        None // No animation needed
                    },
                )?;

                // Render second
                if show_seconds {
                    if let Some(rect) = &second_rect_option {
                        self.engine.render_digit(
                            second,
                            TimeDigitPosition::Second,
                            rect,
                            if is_second_changed {
                                Some(AnimationState {
                                    current_value: second,
                                    previous_value: self.past_second,
                                    is_animating: true,
                                    progress: current_animation_progress.clamp(0.0, 1.0),
                                })
                            } else {
                                None // No animation needed
                            },
                        )?;
                    }
                }

                self.engine.render_am_pm_indicator(&hour_rect, is_pm)?;

                self.engine.present()?;
                std::thread::sleep(Duration::from_millis(16));

                // If all changed flags are false, the animation is complete for this update
                if !is_hour_changed && !is_minute_changed && !is_second_changed {
                    break;
                }
            }
        } else {
            // If no animation, just render the current state
            self.engine.clear()?;
            self.engine
                .render_digit(hour, TimeDigitPosition::Hour, &hour_rect, None)?;
            self.engine
                .render_digit(minute, TimeDigitPosition::Minute, &minute_rect, None)?;
            if show_seconds {
                if let Some(rect) = &second_rect_option {
                    self.engine
                        .render_digit(second, TimeDigitPosition::Second, rect, None)?;
                }
            }
            self.engine.render_am_pm_indicator(&hour_rect, is_pm)?;
            self.engine.present()?;
        }

        self.past_hour = Some(hour);
        self.past_minute = Some(minute);
        self.past_second = Some(second);

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), String> {
        let mut last_update = Instant::now();
        loop {
            if self.engine.handle_events()? {
                break;
            }

            if last_update.elapsed() >= Duration::from_millis(250) {
                self.render()?;
                last_update = Instant::now();
            }

            std::thread::sleep(Duration::from_millis(16));
        }

        Ok(())
    }
}
