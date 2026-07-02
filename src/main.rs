pub mod file_formats;
pub mod core;
pub mod audio;
pub mod macros;

use crate::audio::mixer::MixerT;
use crate::audio::sample::Sample;
use crate::audio::with_rodio::get_mixer;
use macroquad::color::Color;
use macroquad::input::{is_mouse_button_pressed, MouseButton};
use macroquad::text::draw_text;
use macroquad::time::get_frame_time;
use macroquad::window::{clear_background, next_frame};
use macroquad::{conf, miniquad, Window};
use rodio::DeviceSinkBuilder;

fn main() {
    Window::from_config(
        conf::Conf {
            miniquad_conf: miniquad::conf::Conf {
                window_title: String::from("Noteblock Studio Plus"),
                icon: Some(miniquad::conf::Icon {
                    small: file_formats::bmp_fmt::to_rwmj_rgba::<{ 16 * 16 * 4 }>(include_bytes!(asset!("favicon/small.bmp"))).unwrap(),
                    medium: file_formats::bmp_fmt::to_rwmj_rgba::<{ 32 * 32 * 4 }>(include_bytes!(asset!("favicon/medium.bmp"))).unwrap(),
                    big: file_formats::bmp_fmt::to_rwmj_rgba::<{ 64 * 64 * 4 }>(include_bytes!(asset!("favicon/big.bmp"))).unwrap(),
                }),
                platform: miniquad::conf::Platform {
                    swap_interval: Some(0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        main_loop()
    );
}

async fn main_loop() {
    let maybe_handle = DeviceSinkBuilder::open_default_sink();
    let mixer = get_mixer(&maybe_handle);

    let test_sample = Sample::from_file(include_bytes!(asset!("sounds/pling.ogg"))).unwrap();
    let mut pitch = 0.;

    let play_sound = |pitch| {
        if let Some(mix) = &mixer {
            mix.add(&test_sample.shift_pitch(pitch))
        }
    };

    loop {
        clear_background(Color::from_hex(0x000000));

        let (left, right) = (is_mouse_button_pressed(MouseButton::Left), is_mouse_button_pressed(MouseButton::Right));
        if left || right {
            if left { pitch += 100. }
            if right { pitch -= 100. }
            play_sound(pitch);
        }

        draw_text(format!("FPS: {:.2}", 1. / get_frame_time()), 10., 25., 30., Color::from_hex(0xFFFFFF));
        next_frame().await;
    }
}