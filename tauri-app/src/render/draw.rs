use crate::game::state::GameState;
use web_sys::CanvasRenderingContext2d;

// 🎨 保持官方工程的标准画布参数
pub const CANVAS_WIDTH: f64 = 500.0;
pub const CANVAS_HEIGHT: f64 = 350.0;
pub const REEL_WIDTH: f64 = 120.0;
pub const SYMBOL_HEIGHT: f64 = 90.0;
pub const REEL_GAP: f64 = 25.0;

pub fn draw_game(ctx: &CanvasRenderingContext2d, state: &GameState) {
    // 1. 清空全局画布
    ctx.clear_rect(0.0, 0.0, CANVAS_WIDTH, CANVAS_HEIGHT);

    // 2. 绘制外饰红金外框
    ctx.set_fill_style_str("#c0392b");
    ctx.fill_rect(0.0, 0.0, CANVAS_WIDTH, CANVAS_HEIGHT);

    let start_x = 40.0;
    let ry = 30.0;
    let view_h = CANVAS_HEIGHT - 60.0; // 290.0 像素

    // 3. 遍历 3 个滚轮轴进行核心绘制
    for (i, reel) in state.reels.iter().enumerate() {
        let rx = start_x + i as f64 * (REEL_WIDTH + REEL_GAP);

        ctx.save();

        // 矩形窗口裁剪遮罩
        ctx.begin_path();
        ctx.rect(rx, ry, REEL_WIDTH, view_h);
        ctx.clip();

        // 滚轮底盘黑幕
        ctx.set_fill_style_str("#111111");
        ctx.fill_rect(rx, ry, REEL_WIDTH, view_h);

        // 设置文本渲染参数
        ctx.set_font("55px Arial");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_fill_style_str("#ffffff"); // 确保文本颜色为白色

        // 💡 【核心重构：无限流绘制消除黑窟窿】
        // 视窗高度 290.0，符号高 90.0。290 / 90 = 3.22 (可见 3~4 个)
        // 我们向下滚动时，为了防止顶部和底部在滑移瞬间露底，强制绘制 5 行图案。
        // 根据 y_offset 取模计算平滑像素滚动的微调偏移量
        let fine_offset = reel.y_offset % SYMBOL_HEIGHT;

        for row_idx in 0..5 {
            // 💡 修复点 1：通过环形发生器直接获取当前行安全的 Symbol 状态
            let symbol = reel.get_symbol_at_row(row_idx, SYMBOL_HEIGHT);

            // 💡 修复点 2：全新的环形移位物理坐标算法
            // 从视窗上方一个格子高度 (ry - SYMBOL_HEIGHT) 开始作为源点向下铺，并减去平滑位移
            let sy = ry + (row_idx as f64 * SYMBOL_HEIGHT) - fine_offset;

            // 💡 修复点 3：Canvas 2D 文字在视窗内居中对齐绘制
            let _ = ctx.fill_text(
                symbol.to_char(),
                rx + REEL_WIDTH / 2.0,
                sy + SYMBOL_HEIGHT / 2.0,
            );
        }

        ctx.restore();
    }

    // 4. 老虎机核心中线赢利判定指示线（保持原汁原味）
    ctx.set_stroke_style_str("#f1c40f");
    ctx.set_line_width(4.0);
    ctx.begin_path();
    ctx.move_to(15.0, CANVAS_HEIGHT / 2.0);
    ctx.line_to(CANVAS_WIDTH - 15.0, CANVAS_HEIGHT / 2.0);
    ctx.stroke();
}
