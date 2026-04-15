use rand::prelude::*; // 引入 Rng trait 和 thread_rng

/// 游戏下注逻辑
pub fn spin(bet: i32) -> (String, i32) {
    let mut rng = thread_rng(); // 获取线程本地随机数生成器
    let win = if rng.gen_bool(0.5) { bet * 2 } else { 0 }; // 50% 中奖
    ("777".into(), win)
}