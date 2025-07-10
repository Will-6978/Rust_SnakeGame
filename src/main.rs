use piston_window::types::Color;
use piston_window::{clear, Button, PistonWindow, PressEvent, UpdateEvent, WindowSettings, Glyphs, TextureSettings, Transformed};
use std::path::Path;

mod snake_game;
mod snake_snake;
mod snake_window;

use crate::snake_game::game::Game;
use snake_window::draw::to_coord_u32;

/// 定义背景颜色
const BACK_COLOR: Color = [0.5, 0.5, 0.5, 1.0];

// 游戏状态枚举
enum GameState {
    Start,
    Playing,
}

fn main() {
    // 定义窗口大小的参数
    let (width, height) = (30, 30);

    // 定义游戏窗口
    let mut window: PistonWindow =
        WindowSettings::new("Snake", [to_coord_u32(width), to_coord_u32(height)])
            .exit_on_esc(true)
            .build()
            .unwrap();

    // 加载字体
    let assets = Path::new("assets");
    let ref font = assets.join("FZSTK.TTF");
    let mut glyphs = window.load_font(font).expect("无法加载字体文件");

    // 创建游戏
    let mut game = Game::new(width, height);
    // 初始为开始界面
    let mut state = GameState::Start;

    // 监听窗口输入内容
    while let Some(event) = window.next() {
        match state {
            GameState::Start => {
                // 任意按键进入游戏
                if let Some(Button::Keyboard(_)) = event.press_args() {
                    state = GameState::Playing;
                }
                window.draw_2d(&event, |c, g, device| {
                    clear(BACK_COLOR, g);
                    // 居中显示标题和提示（加大字号、加阴影、颜色更鲜明）
                    let title = "贪吃蛇";
                    let tip = "按任意键开始";
                    let transform_title = c.transform.trans(110.0, 200.0);
                    let transform_title_shadow = c.transform.trans(114.0, 204.0);
                    let transform_tip = c.transform.trans(100.0, 270.0);
                    let transform_tip_shadow = c.transform.trans(104.0, 274.0);
                    // 阴影
                    piston_window::text([0.0, 0.0, 0.0, 0.5], 64, title, &mut glyphs, transform_title_shadow, g).ok();
                    piston_window::text([0.0, 0.0, 0.0, 0.5], 32, tip, &mut glyphs, transform_tip_shadow, g).ok();
                    // 主体
                    piston_window::text([0.2, 0.8, 0.2, 1.0], 64, title, &mut glyphs, transform_title, g).unwrap();
                    piston_window::text([1.0, 0.8, 0.2, 1.0], 32, tip, &mut glyphs, transform_tip, g).unwrap();
                    let _ = glyphs.factory.encoder.flush(device);
                });
            }
            GameState::Playing => {
                // 监听用户输入
                if let Some(Button::Keyboard(key)) = event.press_args() {
                    game.key_pressed(key);
                }
                // 清理当前窗口内容，并重新绘制游戏内容
                window.draw_2d(&event, |c, g, device| {
                    clear(BACK_COLOR, g);
                    game.draw(&c, g);
                    // 右上角显示分数
                    let score_text = format!("分数: {}", game.get_score());
                    let transform_score = c.transform.trans(480.0, 40.0);
                    let transform_score_shadow = c.transform.trans(484.0, 44.0);
                    piston_window::text([0.0, 0.0, 0.0, 0.5], 28, &score_text, &mut glyphs, transform_score_shadow, g).ok();
                    piston_window::text([1.0, 1.0, 1.0, 1.0], 28, &score_text, &mut glyphs, transform_score, g).unwrap();
                    glyphs.factory.encoder.flush(device);
                });
                // 更新游戏数据
                event.update(|arg| {
                    game.update(arg.dt);
                });
            }
        }
    }
}
