// crates/app/src/game/reel.rs
use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::SmallRng;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Symbol {
    Seven,
    Cherry,
    Bell,
    Diamond,
    Watermelon,
}

impl Symbol {
    pub fn to_char(&self) -> &'static str {
        match self {
            Symbol::Seven => "7",
            Symbol::Cherry => "🍒",
            Symbol::Bell => "🔔",
            Symbol::Diamond => "💎",
            Symbol::Watermelon => "🍉",
        }
    }
}

pub struct Reel {
    pub symbols: Vec<Symbol>,
    pub y_offset: f64,
    pub speed: f64,
    pub target_speed: f64,
    pub is_stopping: bool,
    pub target_snap_y: f64, // 💡 新增：平滑磁吸的目标锁定坐标
}

impl Reel {
    pub fn new(initial_symbols: Vec<Symbol>) -> Self {
        Self {
            symbols: initial_symbols,
            y_offset: 0.0,
            speed: 0.0,
            target_speed: 0.0,
            is_stopping: false,
            target_snap_y: -1.0, // -1 表示当前未进入吸附锁定状态
        }
    }

    pub fn spin(&mut self) {
        let seed_now = if let Some(window) = web_sys::window() {
            if let Some(perf) = window.performance() {
                perf.now() as u64
            } else {
                1234567
            }
        } else {
            7654321
        };

        let mut rng = SmallRng::seed_from_u64(seed_now);
        self.target_speed = rng.random_range(25.0..40.0);
        self.is_stopping = false;
        self.target_snap_y = -1.0; // 重置吸附目标
    }

    pub fn update(&mut self, delta_time: f64, symbol_height: f64) {
        if symbol_height <= 0.0 {
            return;
        }

        let total_height = self.symbols.len() as f64 * symbol_height;

        // 💡 核心优化：基于状态机的双阶段丝滑减速机制
        if self.target_snap_y < 0.0 {
            // ================= 阶段 1：自由旋转与标准物理摩擦减速 =================
            self.speed += (self.target_speed - self.speed) * 0.1;
            self.y_offset += self.speed * delta_time * 60.0;

            // 当收到刹车指令 (target_speed == 0) 且旋转速度已经慢到一定程度时，切入磁吸锁定
            if self.is_stopping && self.target_speed == 0.0 && self.speed < 12.0 {
                // 精准计算出顺着当前惯性滑行，最近的下一个完美格子对齐点
                let current_grid = (self.y_offset / symbol_height).floor();
                // 核心：锁定在前方 +1 或 +2 的格子，给减速留出优雅的缓冲距离
                self.target_snap_y = (current_grid + 1.0) * symbol_height;
            }
        } else {
            // ================= 阶段 2：黄金渐进缓动（消除突兀跳动） =================
            // 使用经典三次指数衰减（Lerp），让当前位置极其丝滑地“吸”向锁定的目标位置
            // 0.12 控制吸附速度，数值越小越柔和
            let dist = self.target_snap_y - self.y_offset;

            if dist.abs() > 0.2 {
                // 模拟接近终点时的平滑减速，不再硬写
                self.y_offset += dist * 0.12;
                self.speed = dist * 0.12; // 同步平滑降低速度，防止后端强制截断
            } else {
                // 彻底静止，安全软着陆
                self.y_offset = self.target_snap_y;
                self.speed = 0.0;
                self.target_speed = 0.0;
                self.is_stopping = false;
                self.target_snap_y = -1.0; // 释放锁
            }
        }

        // 维持环形有界移动
        if self.y_offset >= total_height {
            self.y_offset -= total_height;
            if self.target_snap_y >= 0.0 {
                self.target_snap_y -= total_height;
            }
        }
    }

    pub fn get_symbol_at_row(&self, row_idx: usize, symbol_height: f64) -> Symbol {
        let total_symbols = self.symbols.len();
        if total_symbols == 0 {
            return Symbol::Cherry;
        }

        let base_idx = (self.y_offset / symbol_height).floor() as i32;
        let mut target_idx = (base_idx + row_idx as i32) % total_symbols as i32;
        if target_idx < 0 {
            target_idx += total_symbols as i32;
        }

        self.symbols[target_idx as usize]
    }
}
