// crates/app/src/app.rs
use crate::game::state::{GameState, GameStatus};
use crate::render::draw::{CANVAS_HEIGHT, CANVAS_WIDTH};
use crate::render::game_loop::start_render_loop;
use dioxus::prelude::*;
use std::sync::{Arc, Mutex};

pub fn App() -> Element {
    // 1. 在本地堆内存中开辟原子锁空间存放单机状态
    let game_core = use_hook(|| Arc::new(Mutex::new(GameState::new())));

    // 2. 创建 Dioxus 数据绑定信号（通过 get/set 驱动 UI 刷新）
    let mut balance_sig = use_signal(|| 1000);
    let mut win_sig = use_signal(|| 0);
    let mut is_spinning_sig = use_signal(|| false);

    // ================== 所有权克隆点 1：Canvas 渲染循环 ==================
    let core_for_render_loop = game_core.clone();
    use_effect(move || {
        start_render_loop(core_for_render_loop.clone(), "slot-canvas");
    });

    // ================== 所有权克隆点 2：HTML 数据定时异步同步协程 ==================
    let core_for_ui_sync = game_core.clone();
    use_effect(move || {
        let core = core_for_ui_sync.clone();
        spawn(async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(50).await;
                //tokio::time::sleep(std::time::Duration::from_millis(50)).await; // 提高同步频率到 50ms，动画反馈更灵敏
                if let Ok(state) = core.lock() {
                    balance_sig.set(state.balance);
                    win_sig.set(state.win_amount);
                    match state.status {
                        GameStatus::Idle => is_spinning_sig.set(false),
                        _ => is_spinning_sig.set(true),
                    }
                }
            }
        });
    });

    // ================== 所有权克隆点 3：拉杆按钮点击事件 ==================
    let core_for_spin_click = game_core.clone();
    let handle_spin = move |_| {
        if let Ok(mut state) = core_for_spin_click.lock() {
            state.start_spin();
        }
    };

    // ✨ 核心修复点：提前在安全作用域中解包信号量，规避 rsx! 内部死锁机制
    let spinning = is_spinning_sig.read().clone();
    let btn_bg = if spinning { "#7f8c8d" } else { "#e67e22" };
    let btn_text = if spinning {
        "🎰 正在疯狂旋转..."
    } else {
        "🔥 启动摇杆！"
    };

    rsx! {
        div { style: "display: flex; flex-direction: column; align-items: center; font-family: sans-serif; background: #1a1a1a; color: white; padding: 30px; min-height: 100vh;",

            h2 { style: "color: #f1c40f; margin-bottom: 20px;", "🎰 纯单机 777 老虎机 🎰" }

            // 核心动画游戏区Canvas
            canvas {
                id: "slot-canvas",
                width: CANVAS_WIDTH,
                height: CANVAS_HEIGHT,
                style: "border: 6px solid #f1c40f; border-radius: 12px; background: #2c3e50; box-shadow: 0 8px 16px rgba(0,0,0,0.5);",
            }

            // 玩家本地数据面板
            div { style: "margin-top: 25px; display: flex; gap: 40px; font-size: 24px; font-weight: bold;",
                div { "💰 余额: {balance_sig} 🪙" }
                div { style: "color: #2ecc71;", "🎉 中奖: {win_sig}" }
            }

            // 摇杆控制按钮
            button {
                // ✨ 修复：直接使用安全的局部变量 btn_bg 填充样式
                style: "margin-top: 25px; padding: 15px 50px; font-size: 24px; cursor: pointer; border-radius: 30px; border: none; font-weight: bold; transition: all 0.2s; color: white; background-color: {btn_bg};",
                disabled: spinning,
                onclick: handle_spin,
                "{btn_text}"
            }
        }
    }
}
