use std::collections::VecDeque;
use std::slice::Iter;

use rand::Rng;
use serenity::model::prelude::*;
use serenity::prelude::*;

pub(crate) async fn message(ctx: Context, msg: Message) {
    if !msg.content.starts_with("snake") { return; }

    let mut split = msg.content.split(' ').skip(1);
    let mut dimensions = SnakeGame::default().board_size;
    if split.clone().count() == 2 {
        dimensions = (split.next().unwrap().parse().unwrap(),
                      split.next().unwrap().parse().unwrap());
    }
    let mut game = SnakeGame {
        snake: VecDeque::new(),
        direction: Direction::Up,
        board_size: dimensions,
        ..Default::default()
    };

    game.init();
    let board = game.get_board();
    for line in board {
        game.msgs.push(msg.channel_id.say(&ctx.http, line).await.unwrap());
    }
    for dir in Direction::iterator() {
        if let Some(last) = game.msgs.last() {
            last.react(
                &ctx.http,
                ReactionType::Unicode(dir.emoji().to_string()),
            ).await.expect("TODO: panic message");
        }
    }

    ctx.data.write().await.insert::<GameData>(game);
}

#[derive(Debug, Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn offset(&self, row_length: usize) -> i32 {
        match self {
            Direction::Up => -(row_length as i32),
            Direction::Down => row_length as i32,
            Direction::Left => -1,
            Direction::Right => 1,
        }
    }
    fn iterator() -> Iter<'static, Direction> {
        static DIRECTIONS: [Direction; 4] = [Direction::Left, Direction::Down, Direction::Up, Direction::Right];
        DIRECTIONS.iter()
    }

    fn emoji(&self) -> &str {
        match self {
            Direction::Up => "⬆️",
            Direction::Down => "⬇️",
            Direction::Right => "➡️",
            Direction::Left => "⬅️"
        }
    }
}

#[derive(Clone)]
struct SnakeGame {
    snake: VecDeque<usize>,
    apples: Vec<usize>,
    direction: Direction,
    msgs: Vec<Message>,
    board_size: (usize, usize),
}

struct GameData;

impl TypeMapKey for GameData {
    type Value = SnakeGame;
}

impl Default for SnakeGame {
    fn default() -> SnakeGame {
        SnakeGame {
            snake: VecDeque::new(),
            apples: Vec::new(),
            direction: Direction::Up,
            msgs: Vec::new(),
            board_size: (20, 10),
        }
    }
}

impl SnakeGame {
    async fn rerender(&self, ctx: &Context) {
        let board = self.get_board();
        for (i, msg) in self.msgs.iter().enumerate() {
            let mut msg = msg.clone();
            let new_content = board.get(i).unwrap();
            if new_content.eq(&msg.content) { continue; }
            msg.edit(&ctx.http, |e| {
                e.content(new_content)
            }).await.expect("Edit message not working");
        }
    }

    fn move_snake(&mut self, direction: Direction) {
        self.direction = direction;
        let index = (*self.snake.back().unwrap() as i32 + self.direction.offset(self.board_size.0)) as usize;
        self.snake.push_back(index);
        if self.apples.contains(&index) {
            let apple_index = self.apples.iter().position(|u| *u == index);
            if let Some(apple_index) = apple_index {
                self.apples.remove(apple_index);
            }
            let mut apple_loc;
            loop {
                apple_loc = rand::thread_rng().gen_range(0..self.board_size.0 * self.board_size.1);
                if !self.snake.contains(&apple_loc) { break; }
            }

            self.apples.push(apple_loc);
        } else {
            self.snake.pop_front();
        }
    }
    fn init(&mut self) {
        let num_cells = self.board_size.0 * self.board_size.1;
        self.snake = VecDeque::with_capacity(num_cells);
        self.snake.push_back(rand::thread_rng().gen_range(0..num_cells));
        self.apples.push(rand::thread_rng().gen_range(0..num_cells));
    }

    fn get_board(&self) -> Vec<String> {
        let mut out = Vec::new();
        for i in 0..self.board_size.1 {
            let mut str = String::new();
            for j in 0..self.board_size.0 {
                let index = i * self.board_size.0 + j;
                if let Some(mut snake_index) = self.snake.iter().position(|u| *u == index) {
                    snake_index = self.snake.len() - snake_index - 1;
                    let replace_with_char = if snake_index < 12 {
                        (":regional_indicator_".to_string())
                            + (&"jonasleonard"[snake_index..snake_index + 1])
                            + ":"
                    } else {
                        ":blue_square:".to_string()
                    };
                    str.push_str(replace_with_char.as_str());
                } else if self.apples.contains(&index) {
                    str.push_str(":lemon:")
                } else {
                    str.push_str(":black_large_square:");
                }
                if str.len() > 1900 { break; }
            }
            out.push(str)
        }
        let mut i = 0;
        let empty_string = String::new();
        let mut out_merged = Vec::new();
        while i < out.len() {
            let mut curr_str = out.get(i).unwrap().clone();
            loop {
                i += 1;
                let next = out.get(i).unwrap_or(&empty_string);
                if next.is_empty() { break; }
                if curr_str.len() + next.len() < 2000 {
                    curr_str.push('\n');
                    curr_str.push_str(next.as_str())
                } else {
                    break;
                }
            }
            out_merged.push(curr_str);
        }
        out_merged.last_mut().unwrap().push_str("\n\u{2800}");
        out_merged
    }
}

pub(crate) async fn reaction_add(ctx: Context, reaction: Reaction) {
    if reaction.user_id.unwrap() == ctx.cache.current_user_id() { return; }
    let mut data = ctx.data.write().await;
    let game = data.get_mut::<GameData>();
    if let Some(game) = game {
        for dir in Direction::iterator() {
            if reaction.emoji.unicode_eq(dir.emoji()) &&
                match game.msgs.first() {
                    None => false,
                    Some(msg) => reaction.channel_id == msg.channel_id
                } {
                let _ = reaction.delete(&ctx.http).await;
                game.move_snake(*dir);
                game.rerender(&ctx).await;
            }
        }
    }
}