use piston_window::types::Color;
use piston_window::{clear, Button, PistonWindow, PressEvent, UpdateEvent, WindowSettings, Glyphs, TextureSettings, Transformed, Key};
use std::path::Path;
use rand::Rng;
use rand::seq::SliceRandom;
use piston_window::{rectangle, ellipse};

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

// 星星结构体
struct Star {
    x: f64,
    y: f64,
    speed: f64,
    size: f64,
}

// 粒子结构体
struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    life: f64,
}

// 鬼字变形状态
struct GhostDeform {
    scale: f64,
    angle: f64,
    color: [f32; 4],
    tear: bool,
}

// 在main函数顶部添加符号和雾气结构体
#[derive(Clone)]
struct FloatingSymbol {
    ch: &'static str,
    x: f64,
    y: f64,
    speed: f64,
    angle: f64,
    scale: f64,
    alpha: f32,
    angle_speed: f64,
    scale_speed: f64,
}

#[derive(Clone)]
struct BloodMist {
    x: f64,
    y: f64,
    rx: f64,
    ry: f64,
    dx: f64,
    dy: f64,
    drx: f64,
    dry: f64,
    alpha: f32,
}

// 在main函数顶部添加怪核符号结构体
#[derive(Clone)]
struct WeirdcoreSymbol {
    ch: &'static str,
    x: f64,
    y: f64,
    angle: f64,
    scale: f64,
    alpha: f32,
    life: f64,
    max_life: f64,
    color: [f32; 4],
}

fn main() {
    // 游戏区大小
    let (game_width, game_height) = (30, 30); // 600x600
    // 窗口大小
    let window_width = 700;
    let window_height = 800;
    // 游戏区左上角坐标（居中偏下）
    let game_x = 50.0;
    let game_y = 100.0;

    let mut window: PistonWindow =
        WindowSettings::new("Snake", [window_width, window_height])
            .exit_on_esc(true)
            .build()
            .unwrap();

    // 加载字体
    let assets = Path::new("assets");
    let ref font = assets.join("FZSTK.TTF");
    let mut glyphs = window.load_font(font).expect("无法加载字体文件");

    // 创建游戏
    let mut game = Game::new(game_width, game_height);
    // 初始为开始界面
    let mut state = GameState::Start;

    // 星空初始化
    let mut stars: Vec<Star> = (0..50)
        .map(|_| {
            let mut rng = rand::thread_rng();
            Star {
                x: rng.gen_range(0.0..600.0),
                y: rng.gen_range(0.0..600.0),
                speed: rng.gen_range(10.0..40.0),
                size: rng.gen_range(1.0..2.5),
            }
        })
        .collect();
    let mut bg_time: f64 = 0.0;

    // ====== 漂浮恐怖符号初始化 ======
    let mut floating_symbols: Vec<FloatingSymbol> = vec![
        FloatingSymbol { ch: "鬼", x: 120.0, y: 180.0, speed: 8.0, angle: 0.0, scale: 1.2, alpha: 0.32, angle_speed: 0.18, scale_speed: 0.07 },
        FloatingSymbol { ch: "卍", x: 540.0, y: 320.0, speed: 10.0, angle: 0.0, scale: 1.0, alpha: 0.22, angle_speed: -0.13, scale_speed: 0.09 },
        FloatingSymbol { ch: "手", x: 200.0, y: 500.0, speed: 7.0, angle: 0.0, scale: 1.3, alpha: 0.18, angle_speed: 0.22, scale_speed: -0.06 },
        FloatingSymbol { ch: "鬼", x: 400.0, y: 600.0, speed: 9.0, angle: 0.0, scale: 0.9, alpha: 0.28, angle_speed: 0.15, scale_speed: 0.05 },
    ];
    // ====== 血色雾气初始化 ======
    let mut blood_mists: Vec<BloodMist> = vec![
        BloodMist { x: 180.0, y: 320.0, rx: 90.0, ry: 38.0, dx: 0.12, dy: 0.08, drx: 0.04, dry: 0.03, alpha: 0.13 },
        BloodMist { x: 500.0, y: 180.0, rx: 60.0, ry: 28.0, dx: -0.09, dy: 0.11, drx: -0.03, dry: 0.02, alpha: 0.10 },
        BloodMist { x: 350.0, y: 520.0, rx: 70.0, ry: 32.0, dx: 0.07, dy: -0.10, drx: 0.02, dry: -0.04, alpha: 0.09 },
    ];

    // 粒子特效相关
    let mut particles: Vec<Particle> = Vec::new();
    let mut flash_timer: f64 = 0.0;
    let mut death_pos: Option<(f64, f64)> = None;

    // 鬼字变形状态
    let mut ghost_deforms: Vec<GhostDeform> = vec![];
    let mut ghost_deform_timer: f64 = 0.0;

    // AI蛇产卵爆炸粒子
    let mut ai_egg_particles: Vec<(f64, f64, f64, f64, f64)> = Vec::new();

    // 玩家吃到食物时AI蛇产卵的hook
    let mut last_score = 0;

    // 玩家速度控制变量
    let mut moving_period: f64 = 0.18;
    let mut ai_snake_speed_min: f64 = moving_period / 2.0;
    let mut ai_snake_speed_max: f64 = moving_period / 2.0;
    const INIT_MOVING_PERIOD: f64 = 0.18;

    // 怪核符号池
    let weirdcore_symbol_pool: [(&str, [f32; 4]); 8] = [
        ("?", [0.9, 0.9, 0.2, 1.0]),
        ("!", [1.0, 0.2, 0.2, 1.0]),
        ("EXIT", [0.7, 0.7, 0.7, 1.0]),
        ("ERROR", [0.8, 0.2, 0.8, 1.0]),
        ("鬼", [0.9, 0.0, 0.0, 1.0]),
        ("眼", [0.7, 0.7, 1.0, 1.0]),
        ("门", [0.6, 0.6, 0.8, 1.0]),
        ("手", [0.8, 0.8, 0.8, 1.0]),
    ];
    let mut weirdcore_symbols: Vec<WeirdcoreSymbol> = Vec::new();
    let mut last_weirdcore_time: f64 = 0.0;

    // 监听窗口输入内容
    while let Some(event) = window.next() {
        match state {
            GameState::Start => {
                window.draw_2d(&event, |c, g, device| {
                    // 动态血色渐变背景
                    let t = (bg_time * 0.5).sin() * 0.5 + 0.5;
                    let bg_color = [
                        (0.1 + 0.2 * t) as f32,
                        0.0,
                        (0.08 + 0.12 * t) as f32,
                        1.0,
                    ];
                    rectangle(bg_color, [0.0, 0.0, window_width as f64, window_height as f64], c.transform, g);

                    // ====== 全屏周期性闪光/闪烁 ======
                    let flash_period = 2.0;
                    let flash_phase = (bg_time % flash_period) / flash_period;
                    let flash_alpha = if flash_phase < 0.12 {
                        // 前12%时间闪光，alpha随sin变化
                        ((0.12 - flash_phase) / 0.12 * std::f64::consts::PI).sin().abs() as f32 * 0.55
                    } else { 0.0 };
                    if flash_alpha > 0.01 {
                        // 血色或白色闪光
                        let color = if flash_phase < 0.06 {
                            [1.0, 1.0, 1.0, flash_alpha]
                        } else {
                            [0.9, 0.1, 0.1, flash_alpha * 0.8]
                        };
                        rectangle(color, [0.0, 0.0, window_width as f64, window_height as f64], c.transform, g);
                    }

                    // ====== 主标题动态抖动/颜色突变 ======
                    let title = "梦魇贪吃蛇";
                    let title_size = 88;
                    let title_w = title.chars().count() as f64 * title_size as f64 * 0.9;
                    // 抖动参数
                    let shake_x = (bg_time * 2.1).sin() * 8.0 + (bg_time * 1.3).cos() * 4.0;
                    let shake_y = (bg_time * 1.7).cos() * 6.0 + (bg_time * 2.7).sin() * 3.0;
                    let scale = 1.0 + (bg_time * 0.9).sin() * 0.03;
                    // 颜色突变
                    let color_flash = ((bg_time * 0.7).sin().abs() > 0.98) as u8;
                    let title_color = if color_flash == 1 {
                        [1.0, 1.0, 1.0, 1.0]
                    } else {
                        [0.95, 0.0, 0.0, 1.0]
                    };
                    let title_x = (window_width as f64 - title_w * scale) / 2.0 - 70.0 + shake_x;
                    let title_y = 220.0 + shake_y;
                    // 渐变阴影
                    for i in 1..6 {
                        let alpha = 0.18 - 0.03 * (i as f32);
                        piston_window::text([0.7, 0.0, 0.0, alpha], title_size, title, &mut glyphs, c.transform.trans(title_x + (i as f64), title_y + (i as f64)).scale(scale, scale), g).ok();
                    }
                    // 主标题
                    piston_window::text(title_color, title_size, title, &mut glyphs, c.transform.trans(title_x, title_y).scale(scale, scale), g).unwrap();

                    // 居中副标题
                    let subtitle = "DREAM HORROR SNAKE";
                    let subtitle_size = 32;
                    let subtitle_w = subtitle.chars().count() as f64 * subtitle_size as f64 * 0.6;
                    let subtitle_x = (window_width as f64 - subtitle_w) / 2.0 - 40.0;
                    let subtitle_y = title_y + 70.0;
                    piston_window::text([0.9, 0.2, 0.2, 0.7], subtitle_size, subtitle, &mut glyphs, c.transform.trans(subtitle_x, subtitle_y), g).ok();

                    // 居中恐怖提示
                    let tip = "按任意键进入噩梦";
                    let tip_size = 36;
                    let tip_w = tip.chars().count() as f64 * tip_size as f64 * 0.6;
                    let tip_x = (window_width as f64 - tip_w) / 2.0 - 40.0;
                    let tip_y = subtitle_y + 80.0;
                    // 多层阴影
                    for i in 1..4 {
                        piston_window::text([0.0, 0.0, 0.0, 0.18], tip_size, tip, &mut glyphs, c.transform.trans(tip_x + (i as f64), tip_y + (i as f64)), g).ok();
                    }
                    piston_window::text([1.0, 0.2, 0.2, 1.0], tip_size, tip, &mut glyphs, c.transform.trans(tip_x, tip_y), g).unwrap();

                    // 居中底部血池
                    let pool_w = 480.0;
                    let pool_h = 90.0;
                    let pool_x = (window_width as f64 - pool_w) / 2.0;
                    let pool_y = window_height as f64 - 90.0;
                    ellipse([0.5, 0.0, 0.0, 0.7], [pool_x, pool_y, pool_w, pool_h], c.transform, g);
                    // 居中血滴
                    let drop_x = window_width as f64 / 2.0 - 9.0;
                    ellipse([0.7, 0.0, 0.0, 0.7], [drop_x, pool_y - 30.0, 18.0, 24.0], c.transform, g);
                    ellipse([0.7, 0.0, 0.0, 0.5], [drop_x + 24.0, pool_y - 18.0, 8.0, 10.0], c.transform, g);
                    ellipse([0.7, 0.0, 0.0, 0.5], [drop_x - 24.0, pool_y - 18.0, 8.0, 10.0], c.transform, g);

                    // 左右两侧对称恐怖符号点缀
                    let side_y = window_height as f64 / 2.0 + 60.0;
                    piston_window::text([0.8, 0.0, 0.0, 0.4], 48, "卍", &mut glyphs, c.transform.trans(40.0, side_y), g).ok();
                    piston_window::text([0.8, 0.0, 0.0, 0.4], 48, "鬼", &mut glyphs, c.transform.trans(window_width as f64 - 80.0, side_y), g).ok();

                    // ====== 漂浮恐怖符号动态更新与绘制 ======
                    for sym in &mut floating_symbols {
                        sym.y += sym.speed * 0.016;
                        sym.angle += sym.angle_speed * 0.016;
                        sym.scale += sym.scale_speed * 0.016;
                        if sym.y > window_height as f64 + 60.0 {
                            sym.y = -60.0;
                        }
                        if sym.scale < 0.8 { sym.scale = 1.2; }
                        if sym.scale > 1.4 { sym.scale = 1.0; }
                        let color = [0.8, 0.0, 0.0, sym.alpha];
                        let transform = c.transform.trans(sym.x, sym.y).rot_rad(sym.angle).scale(sym.scale, sym.scale);
                        piston_window::text(color, 48, sym.ch, &mut glyphs, transform, g).ok();
                    }

                    // ====== 血色雾气动态更新与绘制 ======
                    use piston_window::ellipse;
                    for mist in &mut blood_mists {
                        mist.x += mist.dx;
                        mist.y += mist.dy;
                        mist.rx += mist.drx;
                        mist.ry += mist.dry;
                        if mist.x < 0.0 || mist.x > window_width as f64 { mist.dx = -mist.dx; }
                        if mist.y < 0.0 || mist.y > window_height as f64 { mist.dy = -mist.dy; }
                        if mist.rx < 40.0 || mist.rx > 120.0 { mist.drx = -mist.drx; }
                        if mist.ry < 18.0 || mist.ry > 60.0 { mist.dry = -mist.dry; }
                        let color = [0.8, 0.1, 0.1, mist.alpha];
                        ellipse(color, [mist.x - mist.rx/2.0, mist.y - mist.ry/2.0, mist.rx, mist.ry], c.transform, g);
                    }

                    // ====== 屏幕边缘黑雾/红雾 ======
                    // 多层半透明渐变矩形/椭圆覆盖屏幕边缘
                    let edge_layers = 5;
                    for i in 0..edge_layers {
                        let k = i as f32 / (edge_layers as f32);
                        let alpha = 0.18 * (1.0 - k).powf(1.5) + 0.09 * (bg_time * (1.2 + k as f64)).sin().abs() as f32;
                        let color = [0.08 + 0.3 * k, 0.0, 0.0, alpha];
                        // 上
                        rectangle(color, [0.0, 0.0, window_width as f64, 32.0 + 24.0 * (k as f64)], c.transform, g);
                        // 下
                        rectangle(color, [0.0, window_height as f64 - (32.0 + 24.0 * (k as f64)), window_width as f64, 32.0 + 24.0 * (k as f64)], c.transform, g);
                        // 左
                        rectangle(color, [0.0, 0.0, 32.0 + 24.0 * (k as f64), window_height as f64], c.transform, g);
                        // 右
                        rectangle(color, [window_width as f64 - (32.0 + 24.0 * (k as f64)), 0.0, 32.0 + 24.0 * (k as f64), window_height as f64], c.transform, g);
                    }

                    glyphs.factory.encoder.flush(device);
                });
                if let Some(Button::Keyboard(_)) = event.press_args() {
                    state = GameState::Playing;
                }
            }
            GameState::Playing => {
                // 关卡切换界面
                if game.waiting_next_level {
                    window.draw_2d(&event, |c, g, device| {
                        // 恐怖背景（递增）
                        let t = (bg_time * 0.1).sin() * 0.5 + 0.5;
                        let level = game.level;
                        let base = 0.2 + 0.1 * (level as f32).min(5.0);
                        let red = base + 0.2 * t as f32 + 0.08 * (level as f32);
                        let green = base * (1.0 - 0.08 * (level as f32));
                        let blue = base * (1.0 - 0.12 * (level as f32));
                        use piston_window::rectangle;
                        for i in 0..40 {
                            let k = i as f32 / 39.0;
                            let color = [
                                red * (1.0 - k) + blue * k,
                                green * (1.0 - k) + red * k,
                                blue * (1.0 - k) + red * k,
                                1.0,
                            ];
                            rectangle(color, [0.0, i as f64 * 20.0, window_width as f64, 20.0], c.transform, g);
                        }
                        // 恐怖关卡切换界面
                        let over_text = format!("第{}关完成", game.level);
                        let tip_text = "按任意键进入下一关";
                        let transform_over = c.transform.trans(210.0, 400.0);
                        let transform_tip = c.transform.trans(220.0, 480.0);
                        piston_window::text([1.0, 0.2, 0.2, 1.0], 56, &over_text, &mut glyphs, transform_over, g).unwrap();
                        piston_window::text([1.0, 1.0, 0.2, 1.0], 28, tip_text, &mut glyphs, transform_tip, g).unwrap();
                        glyphs.factory.encoder.flush(device);
                    });
                    // 按任意键进入下一关
                    if let Some(Button::Keyboard(_)) = event.press_args() {
                        game.next_level();
                        // 只有进入下一关时速度才乘2
                        moving_period = (moving_period / 2.0).max(0.04);
                        ai_snake_speed_min = moving_period / 2.0;
                        ai_snake_speed_max = moving_period / 2.0;
                    }
                    continue;
                }
        // 监听用户输入
        if let Some(Button::Keyboard(key)) = event.press_args() {
                    if key == piston_window::Key::R {
                        game.restart();
                        moving_period = INIT_MOVING_PERIOD;
                        ai_snake_speed_min = moving_period / 2.0;
                        ai_snake_speed_max = moving_period / 2.0;
                    }
            game.key_pressed(key);
        }
        // 清理当前窗口内容，并重新绘制游戏内容
                window.draw_2d(&event, |c, g, device| {
                    // 恐怖背景（递增）
                    let t = (bg_time * 0.1).sin() * 0.5 + 0.5;
                    let level = game.level;
                    let base = 0.2 + 0.1 * (level as f32).min(5.0);
                    let red = base + 0.2 * t as f32 + 0.08 * (level as f32);
                    let green = base * (1.0 - 0.08 * (level as f32));
                    let blue = base * (1.0 - 0.12 * (level as f32));
                    use piston_window::{rectangle, ellipse};
                    for i in 0..40 {
                        let k = i as f32 / 39.0;
                        let color = [
                            red * (1.0 - k) + blue * k,
                            green * (1.0 - k) + red * k,
                            blue * (1.0 - k) + red * k,
                            1.0,
                        ];
                        rectangle(color, [0.0, i as f64 * 20.0, window_width as f64, 20.0], c.transform, g);
                    }
                    // 游戏区外半透明黑色分隔带
                    rectangle([0.0, 0.0, 0.0, 0.45], [0.0, 0.0, window_width as f64, game_y], c.transform, g); // 顶部
                    rectangle([0.0, 0.0, 0.0, 0.45], [0.0, game_y + 600.0, window_width as f64, window_height as f64 - (game_y + 600.0)], c.transform, g); // 底部
                    // 游戏区血色发光边框
                    let border_glow = [0.8, 0.0, 0.0, 0.18];
                    rectangle(border_glow, [game_x-12.0, game_y-12.0, 624.0, 24.0], c.transform, g); // 上
                    rectangle(border_glow, [game_x-12.0, game_y-12.0, 24.0, 624.0], c.transform, g); // 左
                    rectangle(border_glow, [game_x-12.0, game_y+588.0, 624.0, 24.0], c.transform, g); // 下
                    rectangle(border_glow, [game_x+588.0, game_y-12.0, 24.0, 624.0], c.transform, g); // 右
                    // 游戏区立体边框
                    let border_light = [0.9, 0.9, 0.9, 1.0];
                    let border_dark = [0.2, 0.0, 0.0, 1.0];
                    rectangle(border_light, [game_x, game_y, 600.0, 8.0], c.transform, g); // 上
                    rectangle(border_light, [game_x, game_y, 8.0, 600.0], c.transform, g); // 左
                    rectangle(border_dark, [game_x, game_y+592.0, 600.0, 8.0], c.transform, g); // 下
                    rectangle(border_dark, [game_x+592.0, game_y, 8.0, 600.0], c.transform, g); // 右
                    // 游戏区内容平移
                    // ====== 梦核/怪核全局画面抽搐与色彩扰动 ======
                    let shake_period = 2.2;
                    let shake_phase = (bg_time % shake_period) / shake_period;
                    let shaking = shake_phase < 0.11;
                    let mut shake_x = 0.0;
                    let mut shake_y = 0.0;
                    let mut shake_scale = 1.0;
                    let mut shake_rot = 0.0;
                    if shaking {
                        // 画面抽搐参数
                        let t = (shake_phase * std::f64::consts::PI * 2.0) as f64;
                        shake_x = (bg_time * 23.0).sin() * 8.0 + (bg_time * 7.0).cos() * 4.0;
                        shake_y = (bg_time * 17.0).cos() * 6.0 + (bg_time * 11.0).sin() * 3.0;
                        shake_scale = 1.0 + (t * 2.0).sin() * 0.025;
                        shake_rot = (t * 1.3).sin() * 0.04;
                    }
                    let c_game = &c.trans(game_x + shake_x, game_y + shake_y)
                        .rot_rad(shake_rot)
                        .scale(shake_scale, shake_scale);
                    // 伪模糊/重影：抽搐时多绘制1~2层错位半透明内容
                    if shaking {
                        for i in 0..2 {
                            let offset = 6.0 + i as f64 * 3.0;
                            let scale = shake_scale * (1.0 + 0.012 * (i as f64 + 1.0));
                            let rot = shake_rot + (i as f64 + 1.0) * 0.02;
                            let c_blur = &c.trans(game_x + shake_x + offset, game_y + shake_y - offset)
                                .rot_rad(rot)
                                .scale(scale, scale);
                            game.draw(c_blur, g, bg_time);
                        }
                    }
                    game.draw(c_game, g, bg_time);
                    // 色彩扰动
                    if shaking {
                        let color_shift = [
                            0.3 + 0.2 * (bg_time * 2.0).sin() as f32,
                            0.1 + 0.3 * (bg_time * 1.3).cos() as f32,
                            0.4 + 0.2 * (bg_time * 1.7).sin() as f32,
                            0.18 + 0.18 * (shake_phase as f32),
                        ];
                        rectangle(color_shift, [game_x, game_y, 600.0, 600.0], c.transform, g);
                    }
                    // 在每个障碍物上绘制呼吸光效和红色“鬼”字（带变形）
                    let breath = ((bg_time * 2.0).sin() * 0.5 + 0.5) as f32; // 0~1
                    let obs = game.get_obstacles();
                    // 初始化变形状态
                    if ghost_deforms.len() != obs.len() {
                        ghost_deforms = obs.iter().map(|_| GhostDeform {
                            scale: 1.0,
                            angle: 0.0,
                            color: [1.0, 0.0, 0.0, 1.0],
                            tear: false,
                        }).collect();
                    }
                    for (i, &(ox, oy)) in obs.iter().enumerate() {
                        // 呼吸光圈
                        let x = (ox as f64) * 20.0 + game_x;
                        let y = (oy as f64) * 20.0 + game_y;
                        let glow_color = [1.0, 0.3, 0.3, 0.18 + 0.22 * breath];
                        let glow_size = 28.0 + 8.0 * breath as f64;
                        ellipse(glow_color, [x + 10.0 - glow_size/2.0, y + 10.0 - glow_size/2.0, glow_size, glow_size], c.transform, g);
                        // 变形参数
                        let deform = &ghost_deforms[i];
                        let tx = x + 2.0 + 8.0 * (1.0 - deform.scale); // 缩放时居中
                        let ty = y + 18.0;
                        let mut transform_ghost = c.transform.trans(tx, ty)
                            .rot_rad(deform.angle)
                            .scale(deform.scale, deform.scale);
                        piston_window::text(deform.color, 16, "鬼", &mut glyphs, transform_ghost, g).ok();
                        // 流泪
                        if deform.tear {
                            let tear_x = x + 10.0;
                            let tear_y = y + 26.0;
                            ellipse([0.8, 0.0, 0.0, 0.8], [tear_x-2.0, tear_y, 4.0, 6.0], c.transform, g);
                        }
                    }
                    // 游戏结束界面美化
                    if game.is_game_over() {
                        use piston_window::rectangle;
                        // 记录死亡点
                        if death_pos.is_none() {
                            let (hx, hy) = game.get_snake_head();
                            death_pos = Some(((hx as f64) * 20.0 + 10.0, (hy as f64) * 20.0 + 10.0));
                        }
                        // 半透明黑色遮罩
                        rectangle([0.0, 0.0, 0.0, 0.6], [0.0, 0.0, window_width as f64, window_height as f64], c.transform, g);
                        // 居中粒子特效
                        for p in &particles {
                            let px = p.x + game_x;
                            let py = p.y + game_y;
                            ellipse([0.8, 0.0, 0.0, (p.life / 1.2).min(1.0) as f32], [px, py, 6.0, 6.0], c.transform, g);
                        }
                        // 居中闪光
                        let flash_alpha = (flash_timer * 20.0).sin().abs().min(1.0) * 0.5;
                        if flash_alpha > 0.01 {
                            rectangle([1.0, 1.0, 1.0, flash_alpha as f32], [game_x, game_y, 600.0, 600.0], c.transform, g);
                        }
                        // 大字“游戏结束”
                        let over_text = "游戏结束";
                        let over_size = 56;
                        let over_w = over_text.chars().count() as f64 * over_size as f64 * 0.9;
                        let over_x = (window_width as f64 - over_w) / 2.0 - 70.0;
                        let transform_over_shadow = c.transform.trans(over_x + 4.0, 340.0);
                        let transform_over = c.transform.trans(over_x, 336.0);
                        piston_window::text([0.0, 0.0, 0.0, 0.7], over_size, over_text, &mut glyphs, transform_over_shadow, g).ok();
                        piston_window::text([1.0, 0.2, 0.2, 1.0], over_size, over_text, &mut glyphs, transform_over, g).unwrap();
                        // 分数和关卡
                        let result_text = format!("分数: {}   关卡: {}", game.get_score(), game.get_level());
                        let result_size = 32;
                        let result_w = result_text.chars().count() as f64 * result_size as f64 * 0.6;
                        let result_x = (window_width as f64 - result_w) / 2.0 - 40.0;
                        let transform_result = c.transform.trans(result_x, 400.0);
                        piston_window::text([1.0, 1.0, 1.0, 1.0], result_size, &result_text, &mut glyphs, transform_result, g).unwrap();
                        // 重开提示
                        let tip_text = "按R键重新开始";
                        let tip_size = 24;
                        let tip_w = tip_text.chars().count() as f64 * tip_size as f64 * 0.6;
                        let tip_x = (window_width as f64 - tip_w) / 2.0 - 40.0;
                        let transform_tip = c.transform.trans(tip_x, 460.0);
                        piston_window::text([1.0, 1.0, 0.2, 1.0], tip_size, tip_text, &mut glyphs, transform_tip, g).unwrap();
                    }
                    // AI蛇产卵爆炸粒子
                    ai_egg_particles.iter_mut().for_each(|p| {
                        p.0 += p.2 * 0.016;
                        p.1 += p.3 * 0.016;
                        p.4 -= 0.016;
                    });
                    ai_egg_particles.retain(|p| p.4 > 0.0);
                    for p in &ai_egg_particles {
                        let color = [0.9, 0.0, 0.0, (p.4 / 0.7).min(1.0) as f32];
                        piston_window::ellipse(color, [p.0-2.0, p.1-2.0, 4.0, 4.0], c.transform, g);
                    }
                    // 顶部UI：关卡/分数/目标
                    let goal_text = format!("第{}关 目标分数：{}/{}  总分：{}", game.level, game.level_score, snake_game::game::Game::LEVEL_GOAL, game.get_score());
                    let transform_goal = c.transform.trans(60.0, 60.0);
                    piston_window::text([0.9, 0.0, 0.0, 1.0], 32, &goal_text, &mut glyphs, transform_goal, g).unwrap();
                    // 底部UI：操作提示
                    let tip_text = "P暂停  R重开  方向键移动";
                    let transform_tip = c.transform.trans(180.0, 780.0);
                    piston_window::text([0.7, 0.0, 0.0, 0.8], 24, tip_text, &mut glyphs, transform_tip, g).unwrap();
                    // 侧边偶尔闪现恐怖符号
                    if (bg_time * 1.5).sin() > 0.92 {
                        let transform_side = c.transform.trans(20.0, 400.0).rot_rad(-0.4).scale(1.8, 1.8);
                        piston_window::text([0.8, 0.0, 0.0, 0.18], 32, "手", &mut glyphs, transform_side, g).ok();
                    }
                    if (bg_time * 1.2).cos() > 0.93 {
                        let transform_side = c.transform.trans(620.0, 700.0).rot_rad(0.3).scale(1.5, 1.5);
                        piston_window::text([0.9, 0.0, 0.0, 0.13], 32, "鬼", &mut glyphs, transform_side, g).ok();
                    }
                    // ====== 梦核/怪核符号随机浮现与闪现 ======
                    // 生成新符号
                    if bg_time - last_weirdcore_time > 1.5 + (bg_time * 0.7).sin().abs() * 1.2 {
                        use rand::Rng;
                        let mut rng = rand::thread_rng();
                        let n = rng.gen_range(1..=2);
                        for _ in 0..n {
                            let (ch, color) = weirdcore_symbol_pool[rng.gen_range(0..weirdcore_symbol_pool.len())];
                            let x = rng.gen_range(game_x + 40.0..game_x + 560.0);
                            let y = rng.gen_range(game_y + 40.0..game_y + 560.0);
                            let angle = rng.gen_range(-0.5..0.5);
                            let scale = rng.gen_range(0.9..1.4);
                            let max_life = rng.gen_range(0.18..0.38);
                            weirdcore_symbols.push(WeirdcoreSymbol {
                                ch,
                                x,
                                y,
                                angle,
                                scale,
                                alpha: 0.0,
                                life: max_life,
                                max_life,
                                color,
                            });
                        }
                        last_weirdcore_time = bg_time;
                    }
                    // 更新并绘制符号
                    let mut i = 0;
                    while i < weirdcore_symbols.len() {
                        let s = &mut weirdcore_symbols[i];
                        let t = 1.0 - (s.life / s.max_life) as f32;
                        // 透明度渐入渐出
                        if t < 0.2 {
                            s.alpha = t / 0.2;
                        } else if t > 0.8 {
                            s.alpha = (1.0 - t) / 0.2;
                        } else {
                            s.alpha = 1.0;
                        }
                        // 抖动/缩放/旋转
                        let scale = s.scale * (1.0 + 0.08 * (bg_time * 7.0 + i as f64).sin());
                        let angle = s.angle + (bg_time * 2.0 + i as f64).cos() * 0.08;
                        let color = [s.color[0], s.color[1], s.color[2], s.color[3] * s.alpha];
                        let transform = c.transform.trans(s.x, s.y).rot_rad(angle).scale(scale, scale);
                        let font_size = if s.ch.len() > 2 { 28 } else { 38 };
                        piston_window::text(color, font_size, s.ch, &mut glyphs, transform, g).ok();
                        s.life -= 0.016;
                        if s.life <= 0.0 {
                            weirdcore_symbols.remove(i);
                        } else {
                            i += 1;
                        }
                    }
                    glyphs.factory.encoder.flush(device);
                });
        // 更新游戏数据
        event.update(|arg| {
                    let prev_score = game.get_score();
            game.update(arg.dt);
                    game.update_ai_snakes(ai_snake_speed_min, ai_snake_speed_max);
                    // 玩家吃到食物时AI蛇产卵
                    let new_score = game.get_score();
                    if new_score > prev_score {
                        game.ai_snake_lay_egg_now(&mut ai_egg_particles);
                    }
                    // 检查玩家与AI蛇碰撞
                    game.check_player_ai_collision();
                    // 星空移动和背景时间推进
                    bg_time += arg.dt;
                    for star in &mut stars {
                        star.y += star.speed * arg.dt;
                        if star.y > 600.0 {
                            let mut rng = rand::thread_rng();
                            star.y = 0.0;
                            star.x = rng.gen_range(0.0..600.0);
                            star.size = rng.gen_range(1.0..2.5);
                            star.speed = rng.gen_range(10.0..40.0);
                        }
                    }
                    // 死亡粒子与闪烁
                    if game.is_game_over() {
                        if let Some((cx, cy)) = death_pos {
                            if particles.is_empty() {
                                // 生成血红色粒子
                                let mut rng = rand::thread_rng();
                                for _ in 0..60 {
                                    let angle = rng.gen_range(0.0..std::f64::consts::PI * 2.0);
                                    let speed = rng.gen_range(80.0..180.0);
                                    let vx = speed * angle.cos();
                                    let vy = speed * angle.sin();
                                    particles.push(Particle {
                                        x: cx,
                                        y: cy,
                                        vx,
                                        vy,
                                        life: 1.2,
                                    });
                                }
                                flash_timer = 0.5;
                            }
                        }
                        // 粒子运动
                        for p in &mut particles {
                            p.x += p.vx * arg.dt;
                            p.y += p.vy * arg.dt;
                            p.life -= arg.dt;
                        }
                        particles.retain(|p| p.life > 0.0);
                        // 闪烁计时
                        if flash_timer > 0.0 {
                            flash_timer -= arg.dt;
                        }
                    } else {
                        particles.clear();
                        flash_timer = 0.0;
                        death_pos = None;
                    }
                    // 鬼字变形定时器
                    ghost_deform_timer += arg.dt;
                    if ghost_deform_timer > 1.2 {
                        ghost_deform_timer = 0.0;
                        let obs = game.get_obstacles();
                        let mut rng = rand::thread_rng();
                        for deform in &mut ghost_deforms {
                            if rng.gen_bool(0.25) {
                                deform.scale = rng.gen_range(0.8..1.3);
                                deform.angle = rng.gen_range(-0.4..0.4);
                                let c = rng.gen_range(0.7..1.0) as f32;
                                deform.color = [c, 0.0, 0.0, 1.0];
                                deform.tear = rng.gen_bool(0.18);
                            }
                        }
                    }
                    // 死亡后重置速度
                    if game.is_game_over() {
                        moving_period = INIT_MOVING_PERIOD;
                        ai_snake_speed_min = moving_period / 2.0;
                        ai_snake_speed_max = moving_period / 2.0;
                    }
                });
            }
        }
    }
}
