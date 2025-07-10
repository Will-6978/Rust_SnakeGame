use crate::snake_window::draw::draw_block;
use piston_window::rectangle::Shape;
use piston_window::types::Color;
use piston_window::{Context, G2d};
use std::collections::LinkedList;

/// 蛇身体的颜色
const SNAKE_BODY_COLOR: Color = [1.0, 0.7, 0.2, 1.0]; // 橙黄色
/// 蛇头的颜色
const SNAKE_HEAD_COLOR: Color = [1.0, 0.2, 0.2, 1.0]; // 亮红色

/// 输入方向限定为 上下左右
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// 方向输入合法性验证，不能直接转向相反方向
    pub fn opposite(&self) -> Direction {
        match *self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

/// 块，蛇的身体的最小单元
#[derive(Debug, Clone)]
pub struct Block {
    pub x: i32,
    pub y: i32,
}

/// 定义蛇的数据
#[derive(Debug)]
pub struct Snake {
    /// 当前朝向
    direction: Direction,
    /// 蛇的身体
    body: LinkedList<Block>,
    /// 蛇的尾巴
    tail: Option<Block>,
}

impl Snake {
    /// 蛇的初始化
    pub fn new(x: i32, y: i32) -> Snake {
        let mut body: LinkedList<Block> = LinkedList::new();
        body.push_back(Block { x: x + 2, y: y });
        body.push_back(Block { x: x + 1, y: y });
        body.push_back(Block { x: x, y: y });
        Snake {
            direction: Direction::Right,
            body,
            tail: None,
        }
    }

    /// 蛇的绘制
    pub fn draw(&self, con: &Context, g: &mut G2d, time: f64) {
        let mut is_head = true;
        let rainbow = [
            [1.0, 0.2, 0.2, 1.0], // 红
            [1.0, 0.7, 0.2, 1.0], // 橙
            [1.0, 1.0, 0.2, 1.0], // 黄
            [0.2, 1.0, 0.2, 1.0], // 绿
            [0.2, 0.7, 1.0, 1.0], // 青
            [0.4, 0.2, 1.0, 1.0], // 蓝
            [1.0, 0.2, 1.0, 1.0], // 紫
        ];
        let mut idx = 0;
        for block in &self.body {
            if is_head {
                is_head = false;
                // 噩梦感蛇头主色：苍白带青紫
                let nightmare_head_color = [0.7, 0.8, 1.0, 1.0];
                // 抖动偏移
                let shake_x = (time * 8.0).sin() * 1.5 + (time * 3.3).cos() * 1.0;
                let shake_y = (time * 7.0).cos() * 1.2 + (time * 2.1).sin() * 0.8;
                let base_x = (block.x as f64) * 20.0 + shake_x;
                let base_y = (block.y as f64) * 20.0 + shake_y;
                use piston_window::{ellipse, line};
                // 蛇头
                draw_block(
                    nightmare_head_color,
                    Shape::Round(10.0, 16),
                    block.x,
                    block.y,
                    con,
                    g,
                );
                // 蛇头高光
                draw_block([0.9, 0.95, 1.0, 0.5], Shape::Round(5.0, 16), block.x, block.y, con, g);
                // 眼睛参数
                let eye_w = 6.0;
                let eye_h = 8.0;
                let eye_y = base_y + 6.0;
                // 左眼
                ellipse([0.2, 0.2, 0.3, 1.0], [base_x + 3.0, eye_y, eye_w, eye_h], con.transform, g); // 黑眼圈
                ellipse([0.85, 0.9, 1.0, 1.0], [base_x + 4.0, eye_y + 1.0, 4.0, 6.0], con.transform, g); // 眼白
                ellipse([0.5, 0.6, 0.8, 1.0], [base_x + 5.5, eye_y + 3.0, 1.5, 2.0], con.transform, g); // 泪痕
                // 右眼
                ellipse([0.2, 0.2, 0.3, 1.0], [base_x + 11.0, eye_y, eye_w, eye_h], con.transform, g); // 黑眼圈
                ellipse([0.85, 0.9, 1.0, 1.0], [base_x + 12.0, eye_y + 1.0, 4.0, 6.0], con.transform, g); // 眼白
                ellipse([0.5, 0.6, 0.8, 1.0], [base_x + 13.5, eye_y + 3.0, 1.5, 2.0], con.transform, g); // 泪痕
                // 嘴巴（下垂弧线）
                line(
                    [0.4, 0.3, 0.5, 1.0],
                    2.0,
                    [base_x + 7.0, base_y + 16.0, base_x + 13.0, base_y + 18.0],
                    con.transform,
                    g,
                );
            } else {
                let color = rainbow[idx % rainbow.len()];
                draw_block(
                    color,
                    Shape::Round(12.5, 16),
                    block.x,
                    block.y,
                    con,
                    g,
                );
                // 蛇身高光
                draw_block([1.0, 1.0, 1.0, 0.3], Shape::Round(6.0, 16), block.x, block.y, con, g);
                idx += 1;
            }
        }
    }

    /// 蛇头的当前坐标
    pub fn head_position(&self) -> (i32, i32) {
        let head = self.body.front().unwrap();
        (head.x, head.y)
    }

    /// 蛇头的当前方向
    pub fn head_direction(&self) -> Direction {
        self.direction
    }

    /// 蛇头的下一个位置的坐标
    pub fn next_head(&self, dir: Option<Direction>) -> (i32, i32) {
        let (head_x, head_y): (i32, i32) = self.head_position();

        let mut moving_dir = self.direction;
        match dir {
            Some(d) => moving_dir = d,
            None => {}
        }

        match moving_dir {
            Direction::Up => (head_x, head_y - 1),
            Direction::Down => (head_x, head_y + 1),
            Direction::Left => (head_x - 1, head_y),
            Direction::Right => (head_x + 1, head_y),
        }
    }

    /// 向前移动
    pub fn move_forward(&mut self, dir: Option<Direction>) {
        match dir {
            Some(d) => self.direction = d,
            None => (),
        }

        let (x, y) = self.next_head(dir);
        self.body.push_front(Block { x, y });
        let remove_block = self.body.pop_back().unwrap();
        self.tail = Some(remove_block);
    }

    /// 增加蛇的长度
    pub fn restore_tail(&mut self) {
        let blk = self.tail.clone().unwrap();
        self.body.push_back(blk);
    }

    /// 自身碰撞检测
    pub fn over_tail(&self, x: i32, y: i32) -> bool {
        let mut ch = 0;
        for block in &self.body {
            if x == block.x && y == block.y {
                return true;
            }
            ch += 1;
            if ch == self.body.len() - 1 {
                break;
            }
        }
        false
    }
}

#[derive(Debug)]
pub struct AISnake {
    pub direction: Direction,
    pub body: LinkedList<Block>,
    pub tail: Option<Block>,
    pub color_head: Color,
    pub color_body: Color,
}

impl AISnake {
    pub fn new(x: i32, y: i32) -> AISnake {
        let mut body: LinkedList<Block> = LinkedList::new();
        body.push_back(Block { x: x + 2, y: y });
        body.push_back(Block { x: x + 1, y: y });
        body.push_back(Block { x: x, y: y });
        AISnake {
            direction: Direction::Left,
            body,
            tail: None,
            color_head: [0.7, 0.0, 0.0, 1.0], // 血红色
            color_body: [0.2, 0.0, 0.0, 1.0], // 暗红色
        }
    }
    pub fn head_position(&self) -> (i32, i32) {
        let head = self.body.front().unwrap();
        (head.x, head.y)
    }
    pub fn move_forward_wrap(&mut self, dir: Option<Direction>, width: i32, height: i32) {
        match dir {
            Some(d) => self.direction = d,
            None => (),
        }
        let (mut x, mut y) = self.next_head(dir);
        // 穿墙逻辑
        if x < 0 { x = width - 2; }
        if x > width - 2 { x = 0; }
        if y < 0 { y = height - 2; }
        if y > height - 2 { y = 0; }
        self.body.push_front(Block { x, y });
        let remove_block = self.body.pop_back().unwrap();
        self.tail = Some(remove_block);
    }
    pub fn next_head(&self, dir: Option<Direction>) -> (i32, i32) {
        let (head_x, head_y): (i32, i32) = self.head_position();
        let mut moving_dir = self.direction;
        match dir {
            Some(d) => moving_dir = d,
            None => {}
        }
        match moving_dir {
            Direction::Up => (head_x, head_y - 1),
            Direction::Down => (head_x, head_y + 1),
            Direction::Left => (head_x - 1, head_y),
            Direction::Right => (head_x + 1, head_y),
        }
    }
    pub fn draw(&self, con: &Context, g: &mut G2d, time: f64) {
        let mut is_head = true;
        let mut idx = 0;
        // 动态错位参数
        let twitch_period = 2.0; // 每2秒一次
        let twitch_phase = (time + (self.head_position().0 as f64) * 0.37 + (self.head_position().1 as f64) * 0.21) % twitch_period;
        let twitching = twitch_phase < 0.08; // 持续约0.08秒
        let mut twitch_idx = 0; // 哪一节抽搐
        if twitching {
            // 随机选一节（头或身）
            let n = self.body.len().min(4);
            if n > 0 {
                twitch_idx = ((time * 13.7).sin().abs() * (n as f64)).floor() as usize;
            }
        }
        for block in &self.body {
            let mut offset_x = 0.0;
            let mut offset_y = 0.0;
            if twitching && idx == twitch_idx {
                offset_x = ((time * 20.0).sin() * 3.0 + (time * 7.0).cos() * 2.0);
                offset_y = ((time * 17.0).cos() * 2.0 + (time * 5.0).sin() * 1.5);
            }
            if is_head {
                is_head = false;
                // 恐怖谷主色：苍白蜡黄
                let uncanny_head_color = [0.95, 0.93, 0.78, 1.0];
                let base_x = (block.x as f64) * 20.0 + offset_x;
                let base_y = (block.y as f64) * 20.0 + offset_y;
                use piston_window::{ellipse, line};
                // 蛇头
                draw_block(
                    uncanny_head_color,
                    Shape::Round(10.0, 16),
                    block.x,
                    block.y,
                    con,
                    g,
                );
                // 头部高光
                draw_block([1.0, 1.0, 0.95, 0.4], Shape::Round(5.0, 16), block.x, block.y, con, g);
                // 一大一小错位眼睛
                // 左眼（大，略高）
                ellipse([0.15, 0.18, 0.22, 1.0], [base_x + 2.0, base_y + 5.0, 7.0, 9.0], con.transform, g); // 眼圈
                ellipse([0.95, 0.98, 1.0, 1.0], [base_x + 3.5, base_y + 6.5, 4.5, 6.0], con.transform, g); // 眼白
                ellipse([0.18, 0.18, 0.22, 1.0], [base_x + 5.0, base_y + 9.0, 2.0, 2.5], con.transform, g); // 瞳孔
                // 黑色泪痕
                ellipse([0.08, 0.08, 0.08, 0.7], [base_x + 6.0, base_y + 13.0, 1.2, 3.5], con.transform, g);
                // 右眼（小，略低，错位）
                ellipse([0.15, 0.18, 0.22, 1.0], [base_x + 11.0, base_y + 8.0, 5.0, 6.0], con.transform, g); // 眼圈
                ellipse([0.95, 0.98, 1.0, 1.0], [base_x + 12.0, base_y + 9.0, 2.8, 3.5], con.transform, g); // 眼白
                ellipse([0.18, 0.18, 0.22, 1.0], [base_x + 13.0, base_y + 11.0, 1.0, 1.3], con.transform, g); // 瞳孔
                // 嘴巴：不对称假笑
                line(
                    [0.25, 0.18, 0.22, 1.0],
                    2.0,
                    [base_x + 7.0, base_y + 16.0, base_x + 15.0, base_y + 14.0],
                    con.transform,
                    g,
                );
                // 嘴角裂口
                line(
                    [0.18, 0.08, 0.08, 1.0],
                    1.0,
                    [base_x + 15.0, base_y + 14.0, base_x + 17.0, base_y + 16.0],
                    con.transform,
                    g,
                );
                // 头部缝线
                line(
                    [0.18, 0.18, 0.22, 0.7],
                    1.0,
                    [base_x + 10.0, base_y + 4.0, base_x + 10.0, base_y + 16.0],
                    con.transform,
                    g,
                );
                for i in 0..4 {
                    let y = base_y + 6.0 + i as f64 * 2.5;
                    line(
                        [0.18, 0.18, 0.22, 0.7],
                        1.0,
                        [base_x + 9.0, y, base_x + 11.0, y + 1.0],
                        con.transform,
                        g,
                    );
                }
            } else {
                // 蛇身为灰蓝色，突出不健康感
                let base_x = (block.x as f64) * 20.0 + offset_x;
                let base_y = (block.y as f64) * 20.0 + offset_y;
                draw_block(
                    [0.45, 0.55, 0.65, 1.0],
                    Shape::Round(12.5, 16),
                    block.x,
                    block.y,
                    con,
                    g,
                );
            }
            idx += 1;
        }
    }
    pub fn restore_tail(&mut self) {
        let last = self.body.back().cloned();
        if let Some(blk) = last {
            self.body.push_back(blk);
        }
    }
}
