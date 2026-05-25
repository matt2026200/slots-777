use crate::game::reel::{Reel, Symbol};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameStatus {
    Idle,
    Spinning,
    Stopping,
}

pub struct GameState {
    pub balance: u32,
    pub bet: u32,
    pub reels: [Reel; 3],
    pub status: GameStatus,
    pub win_amount: u32,
    pub state_timer: f64,
}

impl GameState {
    pub fn new() -> Self {
        let pool = vec![
            Symbol::Seven,
            Symbol::Cherry,
            Symbol::Bell,
            Symbol::Diamond,
            Symbol::Watermelon,
            Symbol::Cherry,
        ];

        Self {
            balance: 1000,
            bet: 10,
            reels: [
                Reel::new(pool.clone()),
                Reel::new(pool.clone()),
                Reel::new(pool.clone()),
            ],
            status: GameStatus::Idle,
            win_amount: 0,
            state_timer: 0.0,
        }
    }

    pub fn start_spin(&mut self) {
        if self.balance < self.bet || !matches!(self.status, GameStatus::Idle) {
            return;
        }

        self.balance -= self.bet;
        self.win_amount = 0;
        self.status = GameStatus::Spinning;
        self.state_timer = 0.0;

        for reel in self.reels.iter_mut() {
            reel.spin();
        }
    }

    pub fn tick(&mut self, dt: f64, symbol_height: f64) {
        self.state_timer += dt;

        match self.status {
            GameStatus::Spinning => {
                if self.state_timer >= 1.2 {
                    self.status = GameStatus::Stopping;
                    self.state_timer = 0.0;
                }
            }
            GameStatus::Stopping => {
                // 阶梯式依次拉起刹车闸
                if self.state_timer >= 0.0 && self.reels[0].target_speed > 0.0 {
                    self.reels[0].target_speed = 0.0;
                    self.reels[0].is_stopping = true;
                }
                if self.state_timer >= 0.3 && self.reels[1].target_speed > 0.0 {
                    // 💡 优化：把阻尼间隔缩短到 0.3 秒，配合全新的平滑吸附，层次感会更紧凑
                    self.reels[1].target_speed = 0.0;
                    self.reels[1].is_stopping = true;
                }
                if self.state_timer >= 0.6 && self.reels[2].target_speed > 0.0 {
                    self.reels[2].target_speed = 0.0;
                    self.reels[2].is_stopping = true;
                }

                // 💡 完美连通：当且仅当所有滚轮在阶段 2 中自己通过 Lerp 彻底把速度降到 0
                let mutex_all_stopped = self
                    .reels
                    .iter()
                    .all(|r| r.speed == 0.0 && r.target_snap_y < 0.0);

                if mutex_all_stopped {
                    // 💡 核心改动：删掉原来的暴力 round() 硬着陆硬写！
                    // 此时滚轮已经通过微积分平滑地呆在正确位置上了
                    for reel in self.reels.iter_mut() {
                        reel.is_stopping = false;
                    }

                    self.status = GameStatus::Idle;
                    self.state_timer = 0.0;

                    // 💵 触发判定
                    self.check_win(symbol_height);
                }
            }
            GameStatus::Idle => {}
        }

        for reel in self.reels.iter_mut() {
            reel.update(dt, symbol_height);
        }
    }

    fn check_win(&mut self, symbol_height: f64) {
        if symbol_height <= 0.0 {
            return;
        }

        // 💡 修复点 3：使用中置对齐环形索引器安全获取正中间展示的符号
        // 视窗通常展示 3 行（第0, 1, 2行），第1行就是绝对的Center中央中奖线
        let s1 = self.reels[0].get_symbol_at_row(1, symbol_height);
        let s2 = self.reels[1].get_symbol_at_row(1, symbol_height);
        let s3 = self.reels[2].get_symbol_at_row(1, symbol_height);

        web_sys::console::log_1(
            &format!("🎰 结算面板图案 -> [ {:?} | {:?} | {:?} ]", s1, s2, s3).into(),
        );

        if s1 == s2 && s2 == s3 {
            let multi = match s1 {
                Symbol::Seven => 100,
                Symbol::Diamond => 50,
                Symbol::Bell => 20,
                Symbol::Watermelon => 15,
                Symbol::Cherry => 10,
            };
            self.win_amount = self.bet * multi;
            self.balance += self.win_amount;
        } else if s1 == s2 || s2 == s3 || s1 == s3 {
            self.win_amount = self.bet;
            self.balance += self.win_amount;
        }
    }
}
