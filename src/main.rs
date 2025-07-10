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
                    // 右上角显示分数和关卡（美化）
                    let score_text = format!("分数: {}", game.get_score());
                    let level_text = format!("关卡: {}", game.get_level());
                    let transform_score_shadow = c.transform.trans(484.0, 44.0);
                    let transform_score = c.transform.trans(480.0, 40.0);
                    let transform_level_shadow = c.transform.trans(484.0, 74.0);
                    let transform_level = c.transform.trans(480.0, 70.0);
                    piston_window::text([0.0, 0.0, 0.0, 0.5], 28, &score_text, &mut glyphs, transform_score_shadow, g).ok();
                    piston_window::text([1.0, 1.0, 0.2, 1.0], 28, &score_text, &mut glyphs, transform_score, g).unwrap();
                    piston_window::text([0.0, 0.0, 0.0, 0.5], 24, &level_text, &mut glyphs, transform_level_shadow, g).ok();
                    piston_window::text([0.2, 0.8, 1.0, 1.0], 24, &level_text, &mut glyphs, transform_level, g).unwrap();
                    // 在每个障碍物上绘制红色“鬼”字
                    for &(ox, oy) in game.get_obstacles() {
                        let x = (ox as f64) * 20.0 + 2.0;
                        let y = (oy as f64) * 20.0 + 18.0;
                        let transform_ghost = c.transform.trans(x, y);
                        piston_window::text([1.0, 0.0, 0.0, 1.0], 16, "鬼", &mut glyphs, transform_ghost, g).ok();
                    }
                    // 游戏结束界面美化
                    if game.is_game_over() {
                        use piston_window::rectangle;
                        // 半透明黑色遮罩
                        rectangle([0.0, 0.0, 0.0, 0.6], [0.0, 0.0, 600.0, 600.0], c.transform, g);
                        // 大字“游戏结束”
                        let over_text = "游戏结束";
                        let transform_over_shadow = c.transform.trans(152.0, 260.0);
                        let transform_over = c.transform.trans(148.0, 256.0);
                        piston_window::text([0.0, 0.0, 0.0, 0.7], 56, over_text, &mut glyphs, transform_over_shadow, g).ok();
                        piston_window::text([1.0, 0.2, 0.2, 1.0], 56, over_text, &mut glyphs, transform_over, g).unwrap();
                        // 分数和关卡
                        let result_text = format!("分数: {}   关卡: {}", game.get_score(), game.get_level());
                        let transform_result = c.transform.trans(170.0, 320.0);
                        piston_window::text([1.0, 1.0, 1.0, 1.0], 32, &result_text, &mut glyphs, transform_result, g).unwrap();
                        // 重开提示
                        let tip_text = "按R键重新开始";
                        let transform_tip = c.transform.trans(180.0, 370.0);
                        piston_window::text([1.0, 1.0, 0.2, 1.0], 24, tip_text, &mut glyphs, transform_tip, g).unwrap();
                    }
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
