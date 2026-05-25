use dioxus::prelude::*;
pub mod app;

mod game;
mod render;

fn main() {
    // 官方默认的启动代码，保持不动
    launch(app::App);
}
