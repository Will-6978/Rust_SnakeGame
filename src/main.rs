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

                    // 居中大标题（血色渐变、滴血阴影）
                    let title = "梦魇贪吃蛇";
                    let title_size = 88;
                    let title_w = title.chars().count() as f64 * title_size as f64 * 0.9;
                    let title_x = (window_width as f64 - title_w) / 2.0 - 70.0;
                    let title_y = 220.0;
                    // 渐变阴影
                    for i in 1..6 {
                        let alpha = 0.18 - 0.03 * (i as f32);
                        piston_window::text([0.7, 0.0, 0.0, alpha], title_size, title, &mut glyphs, c.transform.trans(title_x + (i as f64), title_y + (i as f64)), g).ok();
                    }
                    // 主标题
                    piston_window::text([0.95, 0.0, 0.0, 1.0], title_size, title, &mut glyphs, c.transform.trans(title_x, title_y), g).unwrap();

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
                    let c_game = &c.trans(game_x, game_y);
                    game.draw(c_game, g);
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
