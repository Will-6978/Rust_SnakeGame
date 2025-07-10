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
    pub fn draw(&self, con: &Context, g: &mut G2d) {
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
                draw_block(
                    [1.0, 0.2, 0.2, 1.0], // 亮红色
                    Shape::Round(10.0, 16),
                    block.x,
                    block.y,
                    con,
                    g,
                );
                // 蛇头高光
                draw_block([1.0, 0.6, 0.6, 0.7], Shape::Round(5.0, 16), block.x, block.y, con, g);
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
    pub fn draw(&self, con: &Context, g: &mut G2d) {
        let mut is_head = true;
        for block in &self.body {
            if is_head {
                is_head = false;
                draw_block(
                    self.color_head,
                    Shape::Round(10.0, 16),
                    block.x,
                    block.y,
                    con,
                    g,
                );
                // 恐怖蛇头高光/裂口
                draw_block([0.9, 0.0, 0.0, 0.7], Shape::Round(4.0, 16), block.x, block.y, con, g);
                // 血眼
                use piston_window::ellipse;
                let cx = (block.x as f64) * 20.0 + 7.0;
                let cy = (block.y as f64) * 20.0 + 8.0;
                ellipse([0.8, 0.0, 0.0, 1.0], [cx, cy, 4.0, 4.0], con.transform, g);
                ellipse([0.8, 0.0, 0.0, 1.0], [cx+6.0, cy, 4.0, 4.0], con.transform, g);
                // 裂口笑脸
                use piston_window::line;
                let mx = (block.x as f64) * 20.0 + 8.0;
                let my = (block.y as f64) * 20.0 + 16.0;
                line([0.8, 0.0, 0.0, 1.0], 2.0, [mx, my, mx+8.0, my+2.0], con.transform, g);
            } else {
                draw_block(
                    [0.15, 0.0, 0.0, 1.0], // 更深色
                    Shape::Round(12.5, 16),
                    block.x,
                    block.y,
                    con,
                    g,
                );
            }
        }
    }
    pub fn restore_tail(&mut self) {
        let last = self.body.back().cloned();
        if let Some(blk) = last {
            self.body.push_back(blk);
        }
    }
}
