use piston_window::types::Color;
use piston_window::{clear, Button, PistonWindow, PressEvent, UpdateEvent, WindowSettings, Glyphs, TextureSettings, Transformed};
use std::path::Path;
use rand::Rng;
use rand::seq::SliceRandom;

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

    // 粒子特效相关
    let mut particles: Vec<Particle> = Vec::new();
    let mut flash_timer: f64 = 0.0;
    let mut death_pos: Option<(f64, f64)> = None;

    // 鬼字变形状态
    let mut ghost_deforms: Vec<GhostDeform> = vec![];
    let mut ghost_deform_timer: f64 = 0.0;

    // 监听窗口输入内容
    while let Some(event) = window.next() {
        match state {
            GameState::Start => {
                // 任意按键进入游戏
                if let Some(Button::Keyboard(_)) = event.press_args() {
                    state = GameState::Playing;
                }
                window.draw_2d(&event, |c, g, device| {
                    // 动态渐变背景
                    let t = (bg_time * 0.1).sin() * 0.5 + 0.5;
                    let color1 = [0.2 + 0.3 * t as f32, 0.3 + 0.4 * t as f32, 0.5 + 0.3 * t as f32, 1.0];
                    let color2 = [0.1 + 0.2 * (1.0 - t as f32), 0.2 + 0.3 * (1.0 - t as f32), 0.4 + 0.4 * (1.0 - t as f32), 1.0];
                    use piston_window::rectangle;
                    for i in 0..30 {
                        let k = i as f32 / 29.0;
                        let color = [
                            color1[0] * (1.0 - k) + color2[0] * k,
                            color1[1] * (1.0 - k) + color2[1] * k,
                            color1[2] * (1.0 - k) + color2[2] * k,
                            1.0,
                        ];
                        rectangle(color, [0.0, i as f64 * 20.0, 600.0, 20.0], c.transform, g);
                    }
                    // 星空特效
                    for star in &stars {
                        let star_color = [0.7, 0.85, 1.0, 0.8];
                        piston_window::ellipse(star_color, [star.x, star.y, star.size, star.size], c.transform, g);
                    }
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
                    // 动态渐变背景
                    let t = (bg_time * 0.1).sin() * 0.5 + 0.5;
                    let color1 = [0.2 + 0.3 * t as f32, 0.3 + 0.4 * t as f32, 0.5 + 0.3 * t as f32, 1.0];
                    let color2 = [0.1 + 0.2 * (1.0 - t as f32), 0.2 + 0.3 * (1.0 - t as f32), 0.4 + 0.4 * (1.0 - t as f32), 1.0];
                    use piston_window::{rectangle, ellipse};
                    for i in 0..30 {
                        let k = i as f32 / 29.0;
                        let color = [
                            color1[0] * (1.0 - k) + color2[0] * k,
                            color1[1] * (1.0 - k) + color2[1] * k,
                            color1[2] * (1.0 - k) + color2[2] * k,
                            1.0,
                        ];
                        rectangle(color, [0.0, i as f64 * 20.0, 600.0, 20.0], c.transform, g);
                    }
                    // 星空特效
                    for star in &stars {
                        let star_color = [0.7, 0.85, 1.0, 0.8];
                        ellipse(star_color, [star.x, star.y, star.size, star.size], c.transform, g);
                    }
                    // 游戏区立体边框
                    // 上、左为浅色，下、右为深色
                    let border_light = [0.9, 0.9, 0.9, 1.0];
                    let border_dark = [0.2, 0.2, 0.2, 1.0];
                    // 上边框
                    rectangle(border_light, [0.0, 0.0, 600.0, 8.0], c.transform, g);
                    // 左边框
                    rectangle(border_light, [0.0, 0.0, 8.0, 600.0], c.transform, g);
                    // 下边框
                    rectangle(border_dark, [0.0, 592.0, 600.0, 8.0], c.transform, g);
                    // 右边框
                    rectangle(border_dark, [592.0, 0.0, 8.0, 600.0], c.transform, g);
                    // 发光边框
                    let glow = [0.5, 0.7, 1.0, 0.18];
                    rectangle(glow, [-8.0, -8.0, 616.0, 16.0], c.transform, g); // 上
                    rectangle(glow, [-8.0, -8.0, 16.0, 616.0], c.transform, g); // 左
                    rectangle(glow, [-8.0, 592.0, 616.0, 16.0], c.transform, g); // 下
                    rectangle(glow, [592.0, -8.0, 16.0, 616.0], c.transform, g); // 右
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
                        let x = (ox as f64) * 20.0;
                        let y = (oy as f64) * 20.0;
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
                        rectangle([0.0, 0.0, 0.0, 0.6], [0.0, 0.0, 600.0, 600.0], c.transform, g);
                        // 粒子特效
                        for p in &particles {
                            let color = [0.8, 0.0, 0.0, (p.life / 1.2).min(1.0) as f32];
                            piston_window::ellipse(color, [p.x-3.0, p.y-3.0, 6.0, 6.0], c.transform, g);
                        }
                        // 屏幕闪烁
                        if flash_timer > 0.0 && ((flash_timer * 20.0) as i32) % 2 == 0 {
                            rectangle([1.0, 0.0, 0.0, 0.18], [0.0, 0.0, 600.0, 600.0], c.transform, g);
                        }
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
                    game.update_ai_snakes();
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
                });
            }
        }
    }
}
