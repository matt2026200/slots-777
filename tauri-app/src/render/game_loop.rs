// crates/app/src/render/game_loop.rs
use crate::game::state::GameState;
use crate::render::canvas::get_canvas_context;
use crate::render::draw::{SYMBOL_HEIGHT, draw_game};
use std::sync::{Arc, Mutex};

pub fn start_render_loop(game_state: Arc<Mutex<GameState>>, canvas_id: &'static str) {
    // 记录上一帧的时间戳
    let last_time = web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0);

    // 1. 建立原子锁盒子，用来存储每一帧新产生的 AnimationFrame 句柄
    let f: Arc<Mutex<Option<gloo_render::AnimationFrame>>> = Arc::new(Mutex::new(None));
    let f_clone = f.clone();

    // 2. 定义一个独立、可重复调用的“帧驱动”包装函数
    // 每次调用它，都会向浏览器申请下一次硬件刷新率（~60FPS）的渲染机会
    fn request_next_frame(
        state_ptr: Arc<Mutex<GameState>>,
        box_ptr: Arc<Mutex<Option<gloo_render::AnimationFrame>>>,
        id: &'static str,
        mut prev_time: f64,
    ) {
        let box_ptr_for_closure = box_ptr.clone();

        // 🚀 向浏览器发起动画帧注册
        let handle = gloo_render::request_animation_frame(move |time| {
            // 计算时间步长 dt
            let dt = ((time - prev_time) / 1000.0).min(0.1);
            prev_time = time;

            // 执行物理逻辑和画布渲染
            if let Ok(mut state_lock) = state_ptr.lock() {
                if let Some(ctx) = get_canvas_context(id) {
                    state_lock.tick(dt, SYMBOL_HEIGHT);
                    draw_game(&ctx, &state_lock);
                }
            }

            // ✨【核心修复】：无需调用不存在的 .request() 方法
            // 直接递归调用自己，并将产生的新句柄安全地续写进盒子里，让游戏保持平滑永动！
            request_next_frame(state_ptr, box_ptr_for_closure, id, prev_time);
        });

        // 将当前帧句柄放入原子锁中，维持其生命周期不被销毁
        if let Ok(mut lock) = box_ptr.lock() {
            *lock = Some(handle);
        }
    }

    // 3. 轰油门！传入初始状态，正式启动整个游戏世界线
    request_next_frame(game_state, f_clone, canvas_id, last_time);
}
