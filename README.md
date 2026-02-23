# Moe Priest

一个用 Rust + Bevy 构建的视觉小说（Visual Novel）引擎模板与工具集，目标是提供一个轻量、可扩展的框架，方便用 Bevy 的 ECS 与渲染能力快速搭建旮旯给木原型。

## 主要特性（示例）
- 基于 Bevy 的渲染与 UI，跨平台，支持WebGL / WebGPU
- 基于json配置文件驱动场景（文本、立绘、背景、动画、角色互动）
- 音乐/对话/立绘/背景切换与淡入淡出过渡
- 兼容[utage4](https://madnesslabo.net/utage)模板引擎
- Spine骨骼动画

## 依赖与环境
- Rust（建议使用 stable 通道）
- Cargo（随 Rust 一起安装）
- GPU 驱动与系统依赖（Bevy 依赖 wgpu；请确保你的系统图形驱动是最新的）

## 快速开始

1. 克隆仓库  
   git clone https://github.com/nyfair/moe_priest.git  
   cd moe_priest

2. 安装 Rust（若尚未安装）  
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh && rustup default stable  

3. 运行（开发模式）  
   cargo run

   或构建发行版  
   cargo build --release && ./target/release/moe_priest

## 演示


https://github.com/user-attachments/assets/64d1ab88-3eb6-4a72-b131-0bef3d241574
