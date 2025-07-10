use crate::snake_snake::snake::{Direction, Snake, AISnake};
use crate::snake_window::draw::{draw_block, draw_rectangle};
use piston_window::rectangle::Shape;
use piston_window::types::Color;
use piston_window::{Context, G2d, Key};
use piston_window::Glyphs;
use rand::{thread_rng, Rng};

/// 食物颜色
const FOOD_COLOR: Color = [0.8, 0.2, 1.0, 1.0]; // 亮紫色
/// 上边框颜色
const T_BORDER_COLOR: Color = [0.0000, 0.5, 0.5, 0.6];
/// 下边框颜色
const B_BORDER_COLOR: Color = [0.0000, 0.5, 0.5, 0.6];
/// 左边框颜色
const L_BORDER_COLOR: Color = [0.0000, 0.5, 0.5, 0.6];
/// 右边框颜色
const R_BORDER_COLOR: Color = [0.0000, 0.5, 0.5, 0.6];

///游戏结束颜色
const GAMEOVER_COLOR: Color = [0.90, 0.00, 0.00, 0.5];

/// 移动周期，每过多长时间进行一次移动
const MOVING_PERIOD: f64 = 0.18;

/// AI蛇油滴粒子
#[derive(Debug, Clone)]
pub struct AIOilParticle {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub life: f64, // 剩余寿命，秒
    pub max_life: f64, // 初始寿命
}

/// 游戏主体
#[derive(Debug)]
pub struct Game {
    /// 蛇的主体
    snake: Snake,
    /// 食物是否存在
    food_exists: bool,
    /// 食物x坐标
    food_x: i32,
    /// 食物y坐标
    food_y: i32,
    /// 游戏的宽
    width: i32,
    /// 游戏的高
    height: i32,
    /// 游戏是否结束
    game_over: bool,
    /// 等待时间
    waiting_time: f64,
    /// 是否暂停
    game_pause: bool,
    /// 当前分数
    score: u32,
    pub level: u32,
    pub level_score: u32,
    pub waiting_next_level: bool,
    /// 障碍物位置
    obstacles: Vec<(i32, i32)>,
    /// AI蛇列表
    pub ai_snakes: Vec<AISnake>,
    /// AI蛇移动计时器
    ai_snake_timer: f64,
    ai_snake_speed: f64,
    /// AI蛇油滴粒子
    pub ai_oil_particles: Vec<AIOilParticle>,
}

impl Game {
    pub const LEVEL_GOAL: u32 = 5;
    /// 初始化游戏数据
    pub fn new(width: i32, height: i32) -> Game {
        let mut game = Game {
            snake: Snake::new(2, 2),
            food_exists: true,
            food_x: 6,
            food_y: 4,
            width,
            height,
            game_over: false,
            waiting_time: 0.0,
            game_pause: false,
            score: 0,
            level: 1,
            level_score: 0,
            waiting_next_level: false,
            obstacles: Vec::new(),
            ai_snakes: vec![AISnake::new(width-5, height-5)],
            ai_snake_timer: 0.0,
            ai_snake_speed: 0.18,
            ai_oil_particles: Vec::new(),
        };
        game.generate_obstacles();
        game
    }

    /// 对外暴露的控制方法
    pub fn key_pressed(&mut self, key: Key) {
        // 输入 R 快速重新游戏
        if key == Key::R {
            self.restart()
        }

        if self.game_over {
            return;
        }

        let dir = match key {
            Key::Up => Some(Direction::Up),
            Key::Down => Some(Direction::Down),
            Key::Left => Some(Direction::Left),
            Key::Right => Some(Direction::Right),
            Key::P => {
                // 输入 P 暂停/启动游戏
                self.game_pause = !self.game_pause;
                None
            }
            _ => None,
        };

        if let Some(d) = dir {
            // 如果输入方向为当前方向的相反方向，不做任何处理
            if d == self.snake.head_direction().opposite() {
                return;
            }
        }

        // 如果为有效输入，直接刷新蛇的方向
        self.update_snake(dir);
    }

    /// 是否吃到了果子
    fn check_eating(&mut self) {
        let (head_x, head_y) = self.snake.head_position();
        if self.food_exists && self.food_x == head_x && self.food_y == head_y {
            self.food_exists = false;
            self.snake.restore_tail();
            self.score += 1;
            self.level_score += 1;
            // 玩家吃到食物时AI蛇产卵
            // 这里不能直接用self，因为需要传递粒子数组，由主循环调用
            // 关卡过关检测
            if self.level_score >= Self::LEVEL_GOAL {
                self.waiting_next_level = true;
            }
        }
    }

    /// 对外暴露的游戏绘制
    pub fn draw(&self, con: &Context, g: &mut G2d, time: f64, glyphs: &mut Glyphs) {
        self.snake.draw(con, g, time); // 玩家蛇不需要glyphs
        for ai in &self.ai_snakes {
            // 残影
            let mut fade = 0.4;
            for block in ai.body.iter().skip(1).take(4) {
                draw_block([0.7, 0.0, 0.0, fade], Shape::Round(12.5, 16), block.x, block.y, con, g);
                fade *= 0.6;
            }
            // 恐怖高光
            let (hx, hy) = ai.head_position();
            draw_block([1.0, 0.0, 0.0, 0.7], Shape::Round(6.0, 16), hx, hy, con, g);
            ai.draw(con, g, time); // AI蛇不需要glyphs
        }
        if self.food_exists {
            // 怪核符号果
            draw_weirdcore_food(self.food_x, self.food_y, con, g, time, glyphs);
        }
        // 绘制AI蛇油滴粒子
        use piston_window::ellipse;
        for p in &self.ai_oil_particles {
            let alpha = ((p.life / p.max_life) as f32).min(1.0) * 0.7;
            let size = 6.0 * (p.life / p.max_life).max(0.4);
            ellipse([0.08, 0.08, 0.08, alpha], [p.x - size/2.0, p.y - size/2.0, size, size * 1.2], con.transform, g);
        }
        // 绘制障碍物（深灰色）
        let obstacle_color: Color = [0.2, 0.2, 0.2, 1.0];
        for &(ox, oy) in &self.obstacles {
            draw_block(obstacle_color, Shape::Square, ox, oy, con, g);
        }
        //上边框
        draw_rectangle(T_BORDER_COLOR, 0, 0, self.width, 1, con, g);
        // 下边框
        draw_rectangle(B_BORDER_COLOR, 0, self.height - 1, self.width, 1, con, g);
        // 左边框
        draw_rectangle(L_BORDER_COLOR, 0, 1, 1, self.height - 2, con, g);
        // 右边框
        draw_rectangle(
            R_BORDER_COLOR,
            self.width - 1,
            1,
            1,
            self.height - 2,
            con,
            g,
        );

        // 如果游戏失败 绘制游戏失败画面
        if self.game_over {
            draw_rectangle(GAMEOVER_COLOR, 0, 0, self.width, self.height, con, g);
        }
    }

    /// 对外暴露的游戏更新入口
    pub fn update(&mut self, delta_time: f64) {
        // 如果游戏暂停/结束时，不执行操作
        if self.game_pause || self.game_over {
            return;
        }

        // 增加游戏的等待时间
        self.waiting_time += delta_time;

        if !self.food_exists {
            self.add_food()
        }

        if self.waiting_time > MOVING_PERIOD {
            self.update_snake(None)
        }

        // AI蛇油滴粒子生成与更新
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for ai in &self.ai_snakes {
            // 头部坐标
            let (hx, hy) = ai.head_position();
            // 每帧有小概率生成油滴
            if rng.gen_bool(0.012) {
                let px = (hx as f64) * 20.0 + 10.0 + rng.gen_range(-3.0..3.0);
                let py = (hy as f64) * 20.0 + 18.0;
                let vx = rng.gen_range(-0.5..0.5);
                let vy = rng.gen_range(1.2..2.0);
                let life = rng.gen_range(0.7..1.2);
                self.ai_oil_particles.push(AIOilParticle {
                    x: px,
                    y: py,
                    vx,
                    vy,
                    life,
                    max_life: life,
                });
            }
            // 身体其他节也有更低概率掉落
            for (i, block) in ai.body.iter().enumerate().skip(1).take(2) {
                if rng.gen_bool(0.004) {
                    let px = (block.x as f64) * 20.0 + 10.0 + rng.gen_range(-3.0..3.0);
                    let py = (block.y as f64) * 20.0 + 18.0;
                    let vx = rng.gen_range(-0.4..0.4);
                    let vy = rng.gen_range(1.0..1.7);
                    let life = rng.gen_range(0.6..1.0);
                    self.ai_oil_particles.push(AIOilParticle {
                        x: px,
                        y: py,
                        vx,
                        vy,
                        life,
                        max_life: life,
                    });
                }
            }
        }
        // 更新油滴粒子
        for p in &mut self.ai_oil_particles {
            p.x += p.vx * delta_time * 60.0;
            p.y += p.vy * delta_time * 60.0;
            p.life -= delta_time;
        }
        self.ai_oil_particles.retain(|p| p.life > 0.0);
    }

    /// 添加果子
    fn add_food(&mut self) {
        let mut rng = thread_rng();

        let mut new_x = rng.gen_range(1..self.width - 1);
        let mut new_y = rng.gen_range(1..self.height - 1);

        while self.snake.over_tail(new_x, new_y) {
            new_x = rng.gen_range(1..self.width - 1);
            new_y = rng.gen_range(1..self.height - 1);
        }
        self.food_x = new_x;
        self.food_y = new_y;
        self.food_exists = true;
    }

    /// 生成障碍物，数量=10*level，不能与蛇、食物重叠
    fn generate_obstacles(&mut self) {
        use rand::seq::SliceRandom;
        let mut rng = thread_rng();
        let mut positions = Vec::new();
        for x in 1..self.width-1 {
            for y in 1..self.height-1 {
                // 不与蛇初始位置、食物重叠
                if (x, y) == (2, 2) || (x, y) == (self.food_x, self.food_y) || self.snake.over_tail(x, y) {
                    continue;
                }
                positions.push((x, y));
            }
        }
        positions.shuffle(&mut rng);
        let count = (self.level * 10) as usize;
        self.obstacles = positions.into_iter().take(count).collect();
    }

    /// 进入下一关，速度翻倍，关卡+1，分数清零，障碍物累积
    pub fn next_level(&mut self) {
        self.level += 1;
        self.level_score = 0;
        self.waiting_next_level = false;
        // 玩家和AI蛇长度恢复初始
        self.snake = Snake::new(2, 2);
        for ai in &mut self.ai_snakes {
            *ai = AISnake::new(self.width-5, self.height-5);
        }
        // 速度翻倍（有上限）
        self.ai_snake_speed = (self.ai_snake_speed / 2.0).max(0.04);
        // AI蛇速度范围增加（最大值翻倍，最小值减半）
        // 这里需要主循环配合修改全局MOVING_PERIOD
    }
    /// 获取当前关卡
    pub fn get_level(&self) -> u32 {
        self.level
    }
    /// 获取障碍物位置
    pub fn get_obstacles(&self) -> &Vec<(i32, i32)> {
        &self.obstacles
    }

    /// 判断游戏是否结束
    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    /// 获取蛇头坐标
    pub fn get_snake_head(&self) -> (i32, i32) {
        self.snake.head_position()
    }

    /// 检查当前游戏蛇的生存状态，蛇自身碰撞检测、游戏边界碰撞检测
    fn check_if_snake_alive(&self, dir: Option<Direction>) -> bool {
        let (next_x, next_y) = self.snake.next_head(dir);

        if self.snake.over_tail(next_x, next_y) {
            return false;
        }

        // 蛇头碰到障碍物判定死亡
        if self.obstacles.iter().any(|&(ox, oy)| ox == next_x && oy == next_y) {
            return false;
        }

        next_x > 0 && next_y > 0 && next_x < self.width - 1 && next_y < self.height - 1
    }

    /// 更新蛇的数据
    fn update_snake(&mut self, dir: Option<Direction>) {
        if self.game_pause {
            return;
        }
        if self.check_if_snake_alive(dir) {
            self.snake.move_forward(dir);
            self.check_eating();
        } else {
            self.game_over = true;
        }
        self.waiting_time = 0.0;
    }

    /// 重置游戏
    pub fn restart(&mut self) {
        self.snake = Snake::new(2, 2);
        self.food_exists = true;
        self.food_x = 6;
        self.food_y = 4;
        self.game_over = false;
        self.waiting_time = 0.0;
        self.game_pause = false;
        self.score = 0;
        self.level = 1;
        self.level_score = 0;
        self.waiting_next_level = false;
        self.obstacles.clear();
        self.generate_obstacles();
        self.ai_snakes.clear();
        self.ai_snakes.push(AISnake::new(self.width-5, self.height-5));
        self.ai_snake_timer = 0.0;
        self.ai_snake_speed = 0.18;
        self.ai_oil_particles.clear();
    }

    /// 获取当前分数
    pub fn get_score(&self) -> u32 {
        self.score
    }

    /// 更新AI蛇
    pub fn update_ai_snakes(&mut self, ai_snake_speed_min: f64, ai_snake_speed_max: f64) {
        if self.game_pause { return; }
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        self.ai_snake_speed += rng.gen_range(-0.02..0.02);
        if self.ai_snake_speed < ai_snake_speed_min { self.ai_snake_speed = ai_snake_speed_min; }
        if self.ai_snake_speed > ai_snake_speed_max { self.ai_snake_speed = ai_snake_speed_max; }
        self.ai_snake_timer += 0.016;
        if self.ai_snake_timer < self.ai_snake_speed { return; }
        self.ai_snake_timer = 0.0;
        for ai in &mut self.ai_snakes {
            // 随机游走
            if rng.gen_bool(0.1) {
                let dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
                let dir = *dirs.choose(&mut rng).unwrap();
                ai.move_forward_wrap(Some(dir), self.width, self.height);
            } else {
                ai.move_forward_wrap(None, self.width, self.height);
            }
        }
    }

    /// 玩家与AI蛇碰撞检测
    pub fn check_player_ai_collision(&mut self) {
        let (px, py) = self.snake.head_position();
        for ai in &self.ai_snakes {
            for block in &ai.body {
                if px == block.x && py == block.y {
                    self.game_over = true;
                    return;
                }
            }
        }
    }

    /// 玩家吃到食物时让所有AI蛇产卵并变长
    pub fn ai_snake_lay_egg_now(&mut self, particles: &mut Vec<(f64, f64, f64, f64, f64)>) {
        if self.game_pause { return; }
        let mut to_add = vec![];
        let mut rng = rand::thread_rng();
        for ai in &mut self.ai_snakes {
            let (hx, hy) = ai.head_position();
            // 避免重复产卵
            if !self.obstacles.contains(&(hx, hy)) {
                to_add.push((hx, hy));
                // 爆炸粒子
                for _ in 0..18 {
                    let angle = rng.gen_range(0.0..std::f64::consts::PI * 2.0);
                    let speed = rng.gen_range(40.0..120.0);
                    let vx = speed * angle.cos();
                    let vy = speed * angle.sin();
                    particles.push((hx as f64 * 20.0 + 10.0, hy as f64 * 20.0 + 10.0, vx, vy, 0.7));
                }
            }
            // 不再变长
        }
        for (x, y) in to_add {
            self.obstacles.push((x, y));
        }
    }
}

// 怪核符号池
const WEIRDCORE_SYMBOLS: [(&str, [f32; 4]); 8] = [
    ("?", [0.9, 0.9, 0.2, 1.0]),
    ("!", [1.0, 0.2, 0.2, 1.0]),
    ("EXIT", [0.7, 0.7, 0.7, 1.0]),
    ("ERROR", [0.8, 0.2, 0.8, 1.0]),
    ("鬼", [0.9, 0.0, 0.0, 1.0]),
    ("眼", [0.7, 0.7, 1.0, 1.0]),
    ("门", [0.6, 0.6, 0.8, 1.0]),
    ("手", [0.8, 0.8, 0.8, 1.0]),
];

/// 怪核符号果绘制
pub fn draw_weirdcore_food(x: i32, y: i32, con: &Context, g: &mut G2d, time: f64, glyphs: &mut Glyphs) {
    use piston_window::{ellipse, line, Transformed};
    use rand::Rng;
    let mut rng = rand::thread_rng();
    // 1. 选取符号和主色（随时间变化）
    let idx = ((time * 0.7).sin().abs() * (WEIRDCORE_SYMBOLS.len() as f64)).floor() as usize % WEIRDCORE_SYMBOLS.len();
    let (ch, color) = WEIRDCORE_SYMBOLS[idx];
    // 2. 动态参数
    let base_x = (x as f64) * 20.0;
    let base_y = (y as f64) * 20.0;
    let t = (time * 2.0).sin();
    let scale = 1.0 + 0.13 * (time * 1.7).sin() + 0.07 * (time * 2.9).cos();
    let rot = (time * 1.2).sin() * 0.18;
    let alpha = 0.85 + 0.15 * (time * 3.1).cos();
    // 3. 光晕/阴影
    let glow_color = [color[0], color[1], color[2], 0.18 + 0.12 * (time * 2.7).sin().abs() as f32];
    ellipse(glow_color, [base_x - 8.0, base_y - 8.0, 36.0, 36.0], con.transform, g);
    // 4. 主体符号
    let font_size = if ch.len() > 2 { 18 } else { 28 };
    let symbol_color = [color[0], color[1], color[2], alpha as f32];
    let transform = con.transform.trans(base_x + 10.0, base_y + 10.0).rot_rad(rot).scale(scale, scale);
    piston_window::text(symbol_color, font_size, ch, glyphs, transform, g).ok();
    // 5. 噪点/裂缝
    for i in 0..rng.gen_range(3..7) {
        let angle = time * 2.0 + i as f64 * 1.3;
        let r = 10.0 + 6.0 * (angle * 1.2).sin();
        let px = base_x + 10.0 + r * (angle).cos();
        let py = base_y + 10.0 + r * (angle).sin();
        let dot_alpha = 0.18 + 0.18 * (angle * 1.7).sin().abs() as f32;
        ellipse([0.08, 0.08, 0.08, dot_alpha], [px, py, 2.0, 2.0], con.transform, g);
        if i % 2 == 0 {
            // 裂缝
            let x2 = px + 2.0 * (angle * 2.1).cos();
            let y2 = py + 2.0 * (angle * 2.1).sin();
            line([0.08, 0.08, 0.08, dot_alpha], 1.0, [px, py, x2, y2], con.transform, g);
        }
    }
    // 6. 符号碎片/漂浮点
    for i in 0..rng.gen_range(2..5) {
        let angle = time * 1.3 + i as f64 * 2.2;
        let r = 18.0 + 8.0 * (angle * 1.1).cos();
        let px = base_x + 10.0 + r * (angle).cos();
        let py = base_y + 10.0 + r * (angle).sin();
        let frag_alpha = 0.10 + 0.10 * (angle * 1.9).sin().abs() as f32;
        ellipse([color[0], color[1], color[2], frag_alpha], [px, py, 2.5, 2.5], con.transform, g);
    }
}
