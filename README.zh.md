<div align="center">

# brepkit2

[English](README.md) | 中文

Solid modeling kernel for Rust and WebAssembly.

> **这是一个 fork。** `brepkit2` 是 Andy Mai 的 [brepkit](https://github.com/andymai/brepkit) 的独立 fork，基于上游 tag `v2.129.15`（2026-08-07）切出，此后独立开发。上游采用 MIT OR Apache-2.0 双许可；本 fork 保留该许可、原始版权声明与 crate 命名。本项目与上游作者无隶属关系，也未获得其背书。变更首先落在本仓库——见 [CHANGELOG.md](CHANGELOG.md)。

[![CI](https://github.com/Faicad/brepkit2/actions/workflows/ci.yml/badge.svg)](https://github.com/Faicad/brepkit2/actions/workflows/ci.yml) [![npm](https://img.shields.io/npm/v/@faicad/brepkit2-wasm)](https://www.npmjs.com/package/@faicad/brepkit2-wasm) [![Last release](https://img.shields.io/github/release-date/Faicad/brepkit2?label=last%20release)](https://github.com/Faicad/brepkit2/releases) [![Commit activity](https://img.shields.io/github/commit-activity/m/Faicad/brepkit2?label=commits%2Fmonth)](https://github.com/Faicad/brepkit2/commits/main) [![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license) [![Rust 1.88+](https://img.shields.io/badge/rust-1.88%2B-orange.svg)](https://www.rust-lang.org/) [![unsafe denied](https://img.shields.io/badge/unsafe-denied-success.svg)](#why-a-cad-kernel)

**[架构](#architecture)** · **[性能](#performance)** · **[快速上手](#getting-started)** · **[已知限制](#known-limitations)** · **[参与贡献](./CONTRIBUTING.md)**

</div>

一个精确几何引擎，同时服务 Rust 与 JavaScript。切割实体、测量、导出，一步到位。

```rust
use brepkit_operations::primitives::{make_box, make_cylinder};
use brepkit_operations::boolean::{boolean, BooleanOp};
use brepkit_operations::measure::solid_volume;
use brepkit_io::step::write_step;
use brepkit_topology::Topology;

let mut topo = Topology::new();

// Primitives are anchored at the origin, so this cylinder rounds off the
// block's corner. Use `transform_solid` to place it somewhere else.
let block = make_box(&mut topo, 30.0, 20.0, 10.0)?;
let cutter = make_cylinder(&mut topo, 5.0, 15.0)?;
let notched = boolean(&mut topo, BooleanOp::Cut, block, cutter)?;

// Measure and export
let vol = solid_volume(&topo, notched, 0.1)?;
let step = write_step(&topo, &[notched])?;
```

```js
import { BrepKernel } from '@faicad/brepkit2-wasm';

const kernel = new BrepKernel();

// Primitives are anchored at the origin, so this cylinder rounds off the
// block's corner. Use `transformSolid` to place it somewhere else.
const block = kernel.makeBox(30, 20, 10);
const cutter = kernel.makeCylinder(5, 15);
const notched = kernel.cut(block, cutter);

// Measure and export
const vol = kernel.volume(notched, 0.1);
const step = kernel.exportStep(notched); // Uint8Array
```

<a id="why-a-cad-kernel"></a>

## 为什么需要一个 CAD 内核？

brepkit2 是一个从零开始用 Rust 编写的 B-Rep 实体建模内核。它以 WebAssembly 为目标平台，同一内核既能跑在浏览器里，也能跑在桌面端。`unsafe` 被 lint 拒绝，`unwrap` 和 `panic` 同样如此。所有公开操作都返回 `Result`。

它源于开发 [gridfinitylayouttool.com](https://gridfinitylayouttool.com) 的经历——当时浏览器里可用的参数化 CAD 方案要么是专有的，要么是从大型 C++ 代码库编译而来的。

几何是精确的。布尔运算在解析曲面和 NURBS 曲面上进行，并在运算全程保留这些曲面类型——圆柱经布尔运算后仍是圆柱，而不是一堆三角形。这使面数保持在较低水平，且往返转换无损。

## 状态

brepkit2 处于积极开发中。核心建模已经稳固。下表每个特性都标注了 stable、beta、planned 或 experimental，缺口部分见[已知限制](#known-limitations)。

| 类别                    | 特性                                                                         | 状态         |
| ----------------------- | ---------------------------------------------------------------------------- | ------------ |
| **图元**                | 长方体、圆柱、圆锥、球、圆环、椭球                                           | Stable       |
| **图元**                | 凸包、Minkowski 和（凸输入）                                                 | Stable       |
| **布尔运算**            | 平面、圆柱、圆锥、球、NURBS 上的并、差、交                                   | Stable       |
| **布尔运算**            | 批量 fuse-all（感知不相交的并集）                                            | Stable       |
| **布尔运算**            | 圆环布尔（长方体 ± 圆环、共轴圆环）                                          | Beta         |
| **修改器**              | 圆角（恒定 + 变半径）、倒角（walking 引擎）                                  | Stable       |
| **修改器**              | 抽壳（空心实体）                                                             | Stable       |
| **修改器**              | 偏移面、偏移实体、加厚、镜像、阵列                                           | Stable       |
| **修改器**              | 拔模（平面面）                                                               | Beta         |
| **扫掠**                | 拉伸（平面 + NURBS 轮廓）                                                    | Stable       |
| **扫掠**                | 旋转、扫掠、放样、管道（平面轮廓）                                           | Stable       |
| **扫掠**                | 螺旋扫掠                                                                     | Stable       |
| **扫掠**                | 放样、扫掠、管道、旋转的非平面轮廓                                           | Beta         |
| **构造**                | Coons 补片面填充、缝合、反修剪                                               | Stable       |
| **剖切**                | 截面面、按平面分割                                                           | Stable       |
| **测量**                | 包围盒、面积、体积、质心                                                     | Stable       |
| **测量**                | 点到实体、实体到实体距离、点分类                                             | Stable       |
| **制图**                | 隐藏线边缘投影                                                               | Stable       |
| **几何**                | NURBS 求值、导数、节点操作、拟合、投影                                       | Stable       |
| **几何**                | 解析交集（平面 × 圆柱/圆锥/球精确；圆环采样）                                | Stable       |
| **几何**                | 曲面-曲面求交（解析 + marching）                                             | Stable       |
| **几何**                | 曲线-曲线求交（Bezier 裁剪）                                                 | Stable       |
| **网格化**              | 自适应偏差、CDT、解析曲面优化                                                | Stable       |
| **修复**                | 形体修复（线框、面、壳修复）、缝合、验证                                     | Stable       |
| **I/O**                 | STEP 导入/导出（解析保真往返）                                               | Stable       |
| **I/O**                 | STL、3MF、OBJ、PLY、glTF（`.glb`）导入/导出                                  | Stable       |
| **I/O**                 | IGES 导入/导出                                                               | Experimental |
| **草图**                | 2D 约束求解器（DogLeg）                                                      | Stable       |
| **特征识别**            | 孔、槽、倒角、圆角                                                           | Beta         |
| **装配体**              | 层级、变换、物料清单                                                         | Beta         |
| **演变追踪**            | 布尔运算中的面溯源                                                           | Beta         |
| **去特征**              | 移除平面面                                                                   | Beta         |
| **渲染**                | 离屏 wgpu 渲染至图像 + face-id 缓冲（`brepkit-render`）                      | Experimental |

<a id="known-limitations"></a>

## 已知限制

部分领域仍在成熟中，基于它们构建之前值得了解：

- **布尔回退。** 大多数布尔运算走精确路径，保留解析曲面与 NURBS 曲面。困难配置会回退到基于网格的布尔：共面接触、共轴解析曲面、极薄几何或面数极多的情况。回退结果仍是可用、非退化的实体，但会对曲面进行网格化，且不保证水密。
- **圆环布尔。** 长方体-圆环与共轴圆环的情况可以工作且体积正确。一般圆环-圆环以及圆环与其他曲面的求交存在已知缺口，可能回退到网格化。
- **非平面轮廓。** 放样、扫掠、管道接受非平面曲面的轮廓，并对四边环的非平面截面边界用双线性端盖封闭（超过四条边的边界或非平面截面上的孔尚不支持）。旋转接受非平面轮廓曲面；整周旋转的边界不限，但部分旋转的端盖仍要求平面边界。光滑、缩放/引导、多截面扫掠变体也接受非平面轮廓；只有斜接角变体仍要求平面轮廓（其平分面关节面否则会非平面）。
- **IGES 是实验性的。** 导出写入平面与 NURBS 曲面，但跳过解析曲面，并把圆和椭圆边近似为折线。导入仅重建平面占位面。B-Rep 交换请用 STEP。
- **惯性张量。** 任何实体都能计算体积、面积、包围盒和质心。完整的惯性张量仅有解析图元的闭式公式，且未通过建模 API 或 WASM API 暴露。
- **Beta 子系统。** 特征识别、装配体、演变追踪和去特征可用但仍在成熟。去特征目前只处理平面面。

<a id="scope"></a>

## 范围

brepkit2 有意不做：

- **不在内核中捆绑视口。** 内核输出精确几何与网格化网格；相机、光照与着色属于调用方（Three.js 之类）。可选的 `brepkit-render` crate 提供带 face-id 缓冲的离屏 wgpu 渲染，用于测试和无头验证，任何核心操作都不依赖它。
- **不做刀路规划或切片。** 导出 STEP、STL 或 3MF，把输出交给 CAM 工具或切片器。
- **不用网格建模。** 内核基于精确 B-Rep 几何。细分曲面、多边形网格与体素不在范围内。
- **不提供 GUI。** brepkit2 是一个库。围绕它构建 UI（如 [gridfinitylayouttool.com](https://gridfinitylayouttool.com)）是应用程序的职责。
- **不模拟物理。** 测量（体积、面积、质心）包含在内。应力分析、碰撞检测与动力学不在其中。

<a id="architecture"></a>

## 架构

分层 Cargo workspace。每个 crate 只依赖同一层或更低的层，CI 强制执行边界。

| 层级 | Crate                | 职责                                                                                          |
| ---- | -------------------- | ---------------------------------------------------------------------------------------------- |
| L0   | `brepkit-math`       | 点、向量、矩阵、NURBS 曲线与曲面、几何谓词、CDT、凸包                                          |
| L1   | `brepkit-geometry`   | 曲线采样（均匀、偏差、弧长、曲率）、极值、解析转 NURBS 转换                                    |
| L1   | `brepkit-topology`   | Arena 分配的 B-Rep：vertex、edge、wire、face、shell、solid，附 edge-to-face 邻接索引           |
| L2   | `brepkit-algo`       | General Fuse 布尔引擎：pave filler、面分类、实体装配                                           |
| L2   | `brepkit-blend`      | 基于 walking 的圆角与倒角，支持恒定、变与自定义半径规律                                        |
| L2   | `brepkit-heal`       | 形体修复：分析、修复、升级、缝合、容差管理、可配置管线                                         |
| L2   | `brepkit-check`      | 点分类、验证、属性（体积、面积、质心）、距离                                                   |
| L2   | `brepkit-offset`     | 经全局面-面求交的实体偏移与加厚                                                                |
| L2   | `brepkit-sketch`     | 基于 DogLeg 信赖域法的 2D 参数化约束求解器（GCS）                                              |
| L3   | `brepkit-operations` | 布尔、圆角、倒角、拉伸、旋转、扫掠、放样、抽壳、偏移、测量、网格化                             |
| L3   | `brepkit-io`         | 导入导出：STEP、IGES、STL、3MF、OBJ、PLY、glTF                                                 |
| L4   | `brepkit-wasm`       | 经 wasm-bindgen 的 JavaScript API，带批量执行与检查点/恢复                                     |
| L4   | `brepkit-render`     | 离屏 wgpu 渲染至彩色图像 + face-id 缓冲。可选，无任何依赖                                       |

<a id="performance"></a>

## 性能

中位耗时继承自上游基准测试套件（5 次迭代，Node.js，Linux x86_64）。WASM 为单线程。原生基准使用 criterion。

| 操作                     | brepkit2 (WASM) | OCCT (WASM) | 加速比 | brepkit2 (native) |
| ------------------------ | -------------- | ----------- | ------- | ---------------- |
| fuse(box, box) (×10)     | 0.5 ms         | 43.7 ms     | 87x     | 122 µs           |
| cut(box, cylinder) (×10) | 28.3 ms        | 64.3 ms     | 2.3x    | 9.3 ms           |
| box + chamfer            | 0.2 ms         | 5.4 ms      | 27x     | 46 µs            |
| box + fillet             | 0.3 ms         | 6.2 ms      | 21x     | 127 µs           |
| multi-boolean (16 holes) | 4.7 ms         | 30.1 ms     | 6.4x    | 2.8 ms           |
| mesh sphere (tol=0.01)   | 7.1 ms         | 51.9 ms     | 7.3x    | 6.0 ms           |
| exportSTEP (×10)         | 0.9 ms         | 14.3 ms     | 16x     | n/a              |

每一行引用的数据在计时比较前都在两个内核上做过输出校验：fuse、chamfer 与 sphere 体积完全一致；cut、fillet 与 multi-boolean 体积误差在 0.004% 以内。相同容差下球体网格密度相当（9,800 vs 10,176 三角形）。`intersect(box, sphere)` 行被排除：brepkit2 当前对该配置保留的是错误的球体区域（一个公开的未修复缺陷），其约 200x 的计时数据不具备可比性。

布尔运算保留解析曲面，因此链式操作中面数保持在低水平。一个九步复合布尔最终稳定在 72 个面，而基于网格的方法会达到约 7,000 个。混合（blend）同理：两平面间直边上的圆角保留精确圆柱面，而不是它的 NURBS 近似。

> OCCT 对比使用 [occt-wasm](https://www.npmjs.com/package/occt-wasm)（编译为 WebAssembly 的 OpenCASCADE 构建）。两个内核都在 Node.js 中单线程运行。布尔与 `exportSTEP` 行按十次操作的批次计时。WASM 数据是 `kernel-comparison.bench.test.ts` 的中位数（5 次迭代），对照本地 `cargo xtask wasm-build` 打包并在 require 路径上做哈希校验。原生数据来自 `cargo bench -p brepkit-operations --bench cad_operations`，mesh-sphere 行除外——该行以与 WASM 行相同的参数（`tessellate_solid_with_tolerance`，deflection 0.01，angular 0.1 rad）经 `crates/operations/examples/perf_probe.rs` 测量——criterion 套件的球体用例逐面网格化，不具可比性。测量于 2026-08-06 在上游 brepkit main（2.129.8 之后，含 display-sphere 网格化修复）完成；产生这些数字的测试装置不在本 fork 中，引用前请用 `cargo bench -p brepkit-operations --bench cad_operations` 重新测量。

<a id="data-exchange"></a>

## 数据交换

| 格式          | 类型  | 导入    | 导出   |
| ------------- | ----- | ------- | ------ |
| STEP          | B-Rep | ✓       | ✓      |
| STL           | Mesh  | ✓       | ✓      |
| 3MF           | Mesh  | ✓       | ✓      |
| OBJ           | Mesh  | ✓       | ✓      |
| PLY           | Mesh  | ✓\*     | ✓      |
| glTF (`.glb`) | Mesh  | ✓       | ✓      |
| IGES          | B-Rep | preview | lossy  |

STEP 往返保留精确几何。解析曲面（平面、圆柱、圆锥、球、圆环）以原生 STEP 曲面实体写出而非网格化，读回时仍是相同的曲面类型。NURBS 曲面同样保留，line、circle、ellipse 与 NURBS 边也是如此。

网格格式导出网格化三角形。glTF 为二进制 `.glb`，不含材质或场景图。IGES 是实验性的，见[已知限制](#known-limitations)。

\* PLY 导入在 Rust crate 中可用，但尚未在 WASM API 中暴露。

<a id="getting-started"></a>

## 快速上手

### 作为 WASM 包

```bash
npm install @faicad/brepkit2-wasm
```

```js
import { BrepKernel } from '@faicad/brepkit2-wasm';

const kernel = new BrepKernel();
const solid = kernel.makeBox(10, 20, 30);
```

本 fork 不提供更高层的 TypeScript API。

### 作为 Rust 依赖

尚未发布到 crates.io。目前请使用 git 依赖：

```toml
[dependencies]
brepkit-math = { git = "https://github.com/Faicad/brepkit2" }
brepkit-topology = { git = "https://github.com/Faicad/brepkit2" }
brepkit-operations = { git = "https://github.com/Faicad/brepkit2" }
brepkit-io = { git = "https://github.com/Faicad/brepkit2" }        # optional
```

### 从源码构建

需要 Rust 1.88 或更新版本。

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --all-targets -- -D warnings
cargo fmt --all

# WASM (with I/O)
cargo build -p brepkit-wasm --target wasm32-unknown-unknown --release

# WASM (smaller, no I/O)
cargo build -p brepkit-wasm --target wasm32-unknown-unknown --release --no-default-features

# API docs
cargo doc --workspace --no-deps --open
```

<a id="roadmap"></a>

## 路线图

大方向，不含日期。

- **布尔鲁棒性。** 加固圆环与混合曲面布尔，缩小回退到网格化的输入集合。
- **扫掠泛化。** 把非平面轮廓支持扩展到斜接角扫掠、超过四条边的截面边界，以及非平面边界的部分旋转。
- **WASM 并行网格化。** 原生构建已支持逐面并行网格化，将其通过线程带到 WASM 目标。
- **装配元数据。** 颜色、图层、材质与 PMI，用于更丰富的数据交换。
- **无损 IGES。** 真正的 B-Rep 导入与解析曲面导出。
- **文档。** API 参考、教程与架构指南。

## 使用 brepkit2 的项目

[提交 PR](https://github.com/Faicad/brepkit2/pulls) 添加你的项目。

<a id="license"></a>

## 许可协议

在以下许可下任选其一：

- [Apache License, Version 2.0](./LICENSE-APACHE)
- [MIT License](./LICENSE-MIT)

由你决定。
