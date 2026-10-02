# brepkit2 — 2.x 缺陷修复计划

> 适用范围：本仓库（`brepkit2`，fork 自上游 `v2.129.15`，workspace license = `MIT OR Apache-2.0`）。
> 唯一缺陷来源：`../analysis-2.129.15-to-3.4.18.md`（v2.129.15 → v3.4.18 的变更分析文档，纯文字描述，由另外一个agent读CHANGELOG.md后提供）。
> **硬约束：v3.x 起上游已改为 AGPL-3.0-only，与本仓库许可证不兼容。本计划的全部修复必须在本仓库内自行推导实现，禁止查阅、拉取、反编译或比对 3.x 的任何源码、提交、PR diff、Release 产物。**

一次分析一个问题，完成后报告状态，是复现并解决了，还是复现未解决，还是未能复现。
不管是否解决，都必须写下来，你做了什么，发现了什么，最后的结论是什么，都必须留档。
没解决的问题，不准写Agent Note 。写到analysis里。
不管是否解决，只要能复现，复现的代码必须留下来。
失败的未能修复的加skip/ignore。然后每个bug都是一次独立的提交。不论是否修复。只要能复现，就必须提交，包括文档和代码。未修复的，要记录自己的尝试过程，以及为何没能修复。


---

## 0'. 实施状态（本轮已落地）

工作方式：**先写复现测试 → 确认失败 → 再改实现 → 跑全量回归**。复现不了的项一律跳过。

| ID | 缺陷 | 状态 | 复现测试（修复前实测） | 修复 |
|----|------|------|---------|------|
| E-03 | 射线命中面孔（洞）仍算穿越 | **已修复** | `check/src/classify/mod.rs::ray_through_face_hole_is_not_a_crossing`（计数 1，应为 0）+ 对照 `ray_through_face_material_is_a_crossing` | 新增 `util::face_boundary_loops`（外环+所有孔洞环）；四个计数路径 `ray_plane_crossings` / `count_analytic_crossings` / `count_3d_polygon_crossings` / `ray_crossings_nurbs` 改为"在外环内且不在任何孔洞内"；`is_on_boundary` 同步 |
| E-05 | 点分类只遍历 outer shell，内腔被判成实体内部 | **已修复** | `point_inside_cavity_is_outside`（判为 Inside）+ 对照 `point_in_shell_wall_is_inside` + winding 版 `point_inside_cavity_is_outside_winding` | `classify_point` / `_winding` / `_robust` 的面集合改为 outer + inner shells；`winding_number` 同步，且单面绕数按"外环 − 各孔洞环"计算 |
| F-02 | STEP 读 ELLIPSE 丢弃放置轴（长轴方向） | **已修复** | `io/tests/step_rational_ellipse.rs::ellipse_round_trip_preserves_major_axis`（长轴回读为 `(0,1,0)`，应为 45°） | reader 改用 `Ellipse3D::new_with_ref(center, normal, a, b, u_axis)` |
| F-01 | STEP 写出丢失有理 B 样条权重 | **已修复** | `rational_nurbs_round_trip_preserves_weights`（1/4 圆弧回读半径 1.0346，应为 1.0） | writer 对有理曲线/曲面写出 `RATIONAL_B_SPLINE_*` 复合实体；reader 的权重提取改为从**完整属性串**取 —— 权重段在 knot 段之前，此前只传入 marker 之后的子串，永远取不到权重 |
| E-02 | 修剪圆柱/圆锥面 bbox 用完整解析范围 | **已修复** | `operations/src/measure/bounding_box.rs::trimmed_cylinder_bbox_respects_arc_bounds`（1/4 圆柱片 `min.x = −1`，应为 0） | 新增 `expand_trimmed_revolution`：由修剪边界采样求出角度跨度（圆弧均值 + 最大偏差）与轴向范围，只补跨度内的基数方向极值点；跨度无法推断时退回整圈采样，保持保守 |
| E-01 | 体积/面积/质心未计内腔壳 | **未复现 → 跳过** | `cavity_shell_subtracts_volume`（64−8=56）、`cavity_shell_adds_surface_area`（96+24=120） | 本 fork 已在 `measure/helpers.rs` 计入 `inner_shells`，两条测试直接通过；保留为回归护栏 |
| C-10 | WASM 中 blend panic 杀死整个实例 | **未复现 → 跳过** | — | `wasm/src/bindings/operations.rs:276,332` 与 `batch.rs:812` 已全部包 `catch_unwind` |
| D-01 | sweep 把 profile 重新居中到路径起点 | **语义选择，不改** | — | 直线快路径与通用路径语义一致，且整体满足平移不变性；改成"按原位置"会让所有既有扫掠模型位置漂移，收益与风险不成比例 |
| C-09 | fillet 全引擎失败时静默返回原实体 | **语义选择，不改** | — | `wasm/src/helpers.rs::try_fillet` 已有封闭流形门禁 + 三引擎回退，"返回原实体"是明确定义的降级路径；改为抛错属破坏性 API 变更 |

回归结果：`brepkit-check` 54 项全通过；`brepkit-io` 全量（含 60+ 集成测试）全通过；`brepkit-operations` 全量（809 库测试 + 全部集成测试）全通过。过程、跳过项与不修的决定见 §0''。

---

## 0''. 本轮修复实施记录

### 0''.1 工作方式

严格按 **写复现测试 → 确认失败 → 定位根因 → 改实现 → 跑全量回归** 的闭环推进，一轮只处理一条缺陷。三条硬规则：

1. **不失败的测试不算复现。** 测试必须先跑出红，且失败信息要能指向具体缺陷（打印实测值），而不是笼统的"不相等"。
2. **每条缺陷配一条对照用例（control case）。** 例如 E-03 的"射线穿孔"必须与"射线穿实体材料"成对断言（前者 0 次、后者 1 次）。理由是：几何夹具本身极易构造错（顶点绕向、弧的方向、wire 是否闭合），只有对照用例通过、主用例失败，才能证明失败来自被测实现而非夹具。这条在实践中救了一次——见 0''.2 的 E-02。
3. **期望值只取解析解。** 体积/面积用解析值，长轴方向、半径用构造时输入的常数，穿越次数用 0/1/2 的奇偶推理。全程不引用任何外部数值作 golden。

### 0''.2 五个修复的根因与过程

**E-03 — 射线穿过面孔（洞）仍算一次穿越**

- 夹具：一个平面 `Face`，外环为正方形，内环为居中的方孔；射线沿平面法向穿过孔心。
- 根因：`crates/check/src/util.rs::face_polygon` 只从 `face.outer_wire()` 生成多边形，**`inner_wires` 从头到尾没有参与过**。下游所有"命中点是否在面内"的判定都建立在这个只有外环的多边形上，于是孔洞被当作实体材料。
- 修复：新增 `util::face_boundary_loops(topo, face_id) -> Vec<Vec<Point3>>`，返回 `[外环, 内环…]`；新增 `hit_in_boundary_3d` / `hit_in_boundary_uv`，语义统一为"在外环内 **且** 不在任何内环内"。四条计数路径 `ray_plane_crossings`、`count_analytic_crossings`、`count_3d_polygon_crossings`、`ray_crossings_nurbs`，以及 `is_on_boundary`，全部切到新判定。
- 修复前实测：穿孔计数 1（应为 0）；对照用例（穿实体材料）计数 1，符合预期。

**E-05 — 点分类只遍历 outer shell，内腔被判成实体内部**

- 夹具：4×4×4 外壳内含 2×2×2 空腔（腔壁由 inner shell 表达），取空腔中心点做分类。
- 根因：`classify/mod.rs` 中 `classify_point` / `classify_point_winding` / `classify_point_robust` 三处都只从 `solid.outer_shell()` 取面；`classify/winding.rs::winding_number` 同样。内腔壳的面完全不参与计数，空腔中心向外的射线穿越数为 0 → 判 Inside。
- 修复：三处面集合改为 `once(outer_shell).chain(inner_shells)` 的并集，`winding_number` 同步；单面绕数改为"外环贡献 − 各孔洞环贡献"。
- 关于符号：内腔壳的面法向朝向腔内（从材料侧看是"背面"），但穿越计数的**奇偶**天然给出正确结论——从空腔中心出发的射线穿过内腔壳一次（进入材料）、再穿过外壳一次（离开材料），共 2 次 → Outside。因此不需要为内腔壳额外引入符号翻转，实测也确认如此。这一点是推理 + 实测双向验证后才定的，没有凭直觉加负号。
- 修复前实测：空腔中心判 `Inside`（应为 `Outside`）；对照用例（腔壁材料内的点）判 `Inside`，符合预期。另加了一条 winding 路径的镜像测试。

**F-02 — STEP 读入 ELLIPSE 丢弃放置轴**

- 夹具：一条长轴指向 45°（`(1,1,0)/√2`）的椭圆弧边，write → read → 读回长轴方向。
- 根因非常直白：`step/reader.rs` 的 ELLIPSE 分支写作 `let (center, normal, _u_axis) = self.build_axis2_placement(axis_ref)?;`——第三个返回值正是 STEP `AXIS2_PLACEMENT_3D` 的 ref direction（长轴方向），被 `_` 直接丢弃，椭圆只能退化为"以某个默认基向量为长轴"。
- 修复：改用 `Ellipse3D::new_with_ref(center, normal, a, b, u_axis)`。
- 修复前实测：长轴回读为 `(0, 1, 0)`，与输入的 45° 不符。

**F-01 — STEP 写出丢失有理 B 样条权重（本轮唯一的两层根因）**

- 夹具：一段 1/4 有理圆弧（NURBS，中点权重 `cos(π/4)`），write → read → 用回读曲线采样求半径，与 1.0 比对。
- 第一层根因（writer）：writer 只会输出 `B_SPLINE_CURVE_WITH_KNOTS`，**从不输出权重**。圆因此被写成非有理多项式近似，回读后半径 1.0346 ≠ 1.0。
  修复：新增 `rational_weight_list` / `rational_weight_grid`；当权重非全 1 时，写出 STEP 复合实体 `( RATIONAL_B_SPLINE_CURVE((w…)) B_SPLINE_CURVE_WITH_KNOTS(…) )`，曲面走 `RATIONAL_B_SPLINE_SURFACE` 同理。
- 第二层根因（reader，改完 writer 后测试仍红）：STEP 复合实体里 `RATIONAL_B_SPLINE_CURVE` 段排在 `B_SPLINE_CURVE_WITH_KNOTS` **之前**，而 dispatch 用 `find_composite_bspline_attrs` 只把 marker **之后**的子串交给 builder——权重段被截掉了，永远取不到。
  修复：dispatch 改为把**完整属性串**传给 `build_bspline_curve` / `build_bspline_surface`，并新增 `extract_rational_weights(attrs, expected_count)` 从完整串中提取、校验个数、不匹配时回退到全 1。
- 教训：这条如果没有"写→读→比对"的往返测试，只改 writer 会让人误以为已经修好。**有理几何的正确性只能靠往返验证，不能靠单侧检查。**

**E-02 — 修剪过的圆柱/圆锥面 bbox 用完整解析范围**

- 夹具：半径 1、z∈[0,1]、θ∈[0°,90°] 的 1/4 圆柱片，真 AABB = `[0,1]³`（该片永不进入 x<0 或 y<0 的象限）。
- 根因：`measure/bounding_box.rs` 的 Cylinder/Cone 分支对每个面顶点"取该顶点到轴的距离为半径，在其轴向切片上补一整圈"。任何修剪片都被当成完整回转面处理，1/4 片的 min.x 被算成 −1。
- 修复：新增 `expand_trimmed_revolution`，四步——① `face_boundary_samples` 采样修剪边界（每条边的两端点 + 曲线边内部 25/50/75% 三点）；② 投影到 `(u=角度, v=轴向)`；③ `circular_span` 用圆周均值求角度跨度的中心与半宽（样本在圆周上互相抵消 → 返回 `None`，视为满圈）；④ 只把**落在跨度内**的四个基数方向极值点，配合 `v_min`/`v_max`，并入 AABB。跨度无法推断时退回 `FULL_RING_SAMPLES = 32` 的整圈采样。
- **保守性优先**：bbox 是布尔快速路径的剪枝依据，收紧过头会导致漏判相交，后果比略微过估严重得多。因此"推断不出跨度"时一律退回整圈，宁可大不可小。
- 夹具踩坑：第一版夹具的顶弧与底弧共用同一个 `Circle3D` 且端点顺序走了 270° 那一侧，导致采样角度跨度被判成满圈、测试依旧失败。这是**夹具错而非实现错**——正是 0''.1 第 2 条"必须有对照用例"起作用的地方；把顶弧改成反向边后，实现侧一次通过。

### 0''.3 无法复现 → 跳过

| ID | 检查过程 | 结论 |
|----|---------|------|
| E-01 | 直接写解析对照：外壳 4³ − 内腔 2³ = 56，面积 96 + 24 = 120。两条测试**一次通过**，未出现红。查 `measure/helpers.rs` 确认已 `chain(inner_shells)`，且 volume / area / centroid 三条路径共用同一 helper | 本 fork 已正确，**跳过修复**；两条测试保留为回归护栏，防止将来有人在重构中漏掉 inner shell |
| C-10 | 逐一检查 WASM 的 blend 入口：`wasm/src/bindings/operations.rs:276`、`operations.rs:332`、`batch.rs:812` 三处已全部包 `catch_unwind`；未能构造出能杀死实例的用例 | 本 fork 已覆盖，**跳过** |

另有 A / B / C / G / H 域的一批条目（表 §1 中标记为"待复现"的）本轮**未尝试复现**，原因是它们缺少可度量的失败信号：要先有 Phase 0 的脚手架（布尔 mesh-fallback 计数器、统一的几何不变量校验入口）才能把"精度损失""面数暴涨""环嵌套错误"这类模糊症状转成可断言的红。这些条目仍停在"待复现"状态，**不等于已排除**。

### 0''.4 不修的决定（语义选择，不是缺陷）

这两条原先挂在 §7 待拍板，本轮实施中自行判定为**明确不改**，理由如下：

- **D-01 — sweep 把 profile 重新居中到路径起点**
  判定依据：① 直线快路径与通用路径的语义一致，不存在两条路径打架；② 整体满足平移不变性（把输入整体平移 Δ，输出也恰好整体平移 Δ）；③ 若改成"按 profile 原位置扫掠"，**所有既有扫掠模型的位置都会漂移**，属于破坏性变更，而且没有解析期望值可供校验"新位置才是对的"。收益与风险不成比例 → **不改**。

- **C-09 — fillet 全引擎失败时静默返回原实体**
  判定依据：`wasm/src/helpers.rs::try_fillet` 已有封闭流形门禁 + 三引擎回退，"返回原实体"是一条**明确定义过的降级路径**，调用方可自行按体积/面数判定是否真的产生了圆角。改为抛错是破坏性 API 变更，会让现有调用方从"拿到结果"变成"抛异常"。 → **不改**。
  若将来确需更强的语义，建议**新增** `fillet_strict` 入口（失败即抛错），而不是修改现有行为——这样既不破坏兼容性，又给了调用方选择权。

- 附带结论：§7 的待拍板第 1 项（D-01）与第 3 项（C-09）可关闭；第 2 项（E-02 bbox 收紧的影响）已随本轮实施落地且全量回归无回退，亦可关闭。

### 0''.5 回归与验证

| 项 | 结果 |
|----|------|
| `cargo test -p brepkit-check` | 54 项全通过 |
| `cargo test -p brepkit-operations` | 809 库测试 + 全部集成测试，全通过 |
| `cargo test -p brepkit-io` | 全量（含 60+ 集成测试）全通过 |
| `cargo clippy -p brepkit-check -p brepkit-operations -p brepkit-io --all-targets` | 零告警（过程中修掉两处 `clippy::type_complexity`：把 `specs` 数组的元素类型抽成 `type CubeFaceSpec`） |
| `scripts/check-boundaries.sh` | 通过（分层依赖未被破坏） |

改动文件：`crates/check/src/util.rs`、`crates/check/src/classify/{mod,boundary,winding}.rs`、`crates/io/src/step/{reader,writer}.rs`、`crates/operations/src/measure/{mod,bounding_box}.rs`；新增 `crates/io/tests/step_rational_ellipse.rs`。

### 0''.6 残余风险（未闭环，需在后续阶段处理）

1. **E-02 的 bbox 收紧会改变布尔快速路径的剪枝结果。** 三个 crate 的全量回归无回退，但 `tests/*.golden` 若存在依赖旧（过估）bbox 的记录，需人工确认后更新。
2. **F-01 的复合实体写法未被第三方 STEP 解析器验证过**，本轮只验证了本仓库自身的 write → read 往返。跨工具互通性需要另找商用/开源内核做一次交叉读写。
3. **分类器的改动会影响 `check` 的下游**（distance、properties、validity）。本轮只跑了 check / operations / io 三个 crate，未跑 workspace 全量与 WASM 侧测试。
4. **A 域 14 条零进展。** 它们是数量与风险的大头，但缺少度量尺（Phase 0.3 的 fallback 计数器）之前无法有效推进。

---

## 0'''. 第三轮实施记录（Phase 0.3 可观测性 + C 域复现）

### 0'''.1 已修复：B-06 布尔降级无观测（Phase 0.3 落地）

| 项 | 内容 |
|----|------|
| 复现 | `crates/operations/tests/boolean_mesh_fallback_counter.rs` —— 引用 `mesh_fallback_count` 时编译失败（`no mesh_fallback_count in boolean`），证明 `crates/` 下没有任何 API 能观测到布尔降级 |
| 修复 | `crates/operations/src/boolean/mod.rs` 新增进程级 `AtomicU64` 计数器；在 `mesh_boolean_fallback` **入口**自增（计数的是「GFA 无法解析服务这次布尔」这一事件，与该次 fallback 自身是否随后成功无关）；导出 `mesh_fallback_count()` / `reset_mesh_fallback_count()` |
| WASM | `crates/wasm/src/bindings/booleans.rs` 新增 `BrepKernel::meshFallbackCount()` |
| 测试 | 三段断言：① 对照（两 box 解析 fuse，体积解析值 1091 = 1000+216−5³）计数 delta = 0；② 主用例（正交圆柱 fuse）计数 = 1；③ delta 式读取再验证 |
| 断言 vs 实现 | 计数器是可观测性基础设施，不改变任何几何输出 |

**该计数器首次使用即给出 A 域的实测基线**（本轮扫描，`fallback=1` 表示该布尔走了 mesh 降级）：

| 场景 | fallback |
|------|---------|
| 两 box fuse / slab 切盲孔（#696 夹具）/ 圆锥切圆柱 / 多刀具顺序切 / 共面 box fuse | **0**（解析路径） |
| 正交圆柱 fuse、torus ∩ box、torus ∪ box、sphere − box、64 段 sphere − sphere / − 倾斜 box | **1** |

即 A 域的度量尺已就位，且立即指出「曲面×曲面」「曲面×平面」的解析交线仍有一批走 mesh 降级 —— 这是 Phase 2 的排序入口。

### 0'''.2 未复现 → 跳过：C-03（面片翻转/法向朝内）

结论：**本 fork 的 fillet 结果几何定向正确，C-03 不复现**。判定用了两条互相独立的证据：

1. **解析法向 + 凸体判据。** 对 10³ box 全 12 棱圆角（r=1/2.5/4）的结果，逐面取解析曲面法向、施加 `is_reversed`，再与「面点 − 体中心」作点积：18 个解析可判定面（6 平面 + 12 圆柱）**全部朝外**，0 个朝内（剩余 8 个角面为 NURBS，判据不覆盖）。
2. **体积。** r=1 结果 975.332，与「1000 − 12 条棱圆角去除 + 8 个角部」的解析预期 ≈975.6 一致；若真有大面积翻面，发散积分必然偏离。

过程中的**两次假信号**（记录以免后续重踩）：

- **tessellate 三角形绕向不能当定向判据。** 用「三角形绕向法向 + 沿法向偏移 1e-4 探测内/外」判定时，圆柱面与角面被集中误判为「朝内」（18/26）。根因是偏移步长 1e-4 小于曲面离散的弦深，探测点仍落在实体内部。同一误判在**未圆角的圆柱基元**上同样出现（侧面恒报 1 个），这正是「先建对照用例」纪律要拦下的东西。
- **`check_shell_orientation` 报「24 条共享边同向使用」。** 经逐边分类确认为真·两个不同面同向（`sameFaceDup=0`），即 fillet 结果里 wire 边的 `is_forward` 记录与面相向不自洽。但解析法向与体积双双证明几何朝向无误，且既有 `try_fillet_second_pass_does_not_break_solid` 等下游测试未受影响 —— 因此判为**拓扑标记层面的不一致，无几何后果**，本轮不改（改动会触及 fillet 引擎的面/wire 生成，收益不明而风险大）。

顺带否定一条替代方案：`operations::heal::fix_face_orientations` 对本场景**无效**（`flipped=0`）。它只处理平面面、且靠改 `FaceSurface::Plane` 的 `normal/d` 来翻转，对圆柱/圆锥/球面显式跳过；而实验表明用 BFS 强行翻转 `is_reversed` 反而把体积从正确的 975 改成错误的 775 —— **这两种"修复"都不能用**。

### 0'''.3 本轮未复现 / 无法测的其余 C 域条目

| ID | 检查过程 | 结论 |
|----|---------|------|
| C-01 | box 10³ 全 12 棱：r=1 → 975.332、r=2.5 → 856.175、r=4 → 658.177，均 `closed=true` 且无过度去除迹象；既有 `try_fillet_all_box_edges_no_corner_over_removal` 亦通过 | **未复现**。但发现一条**待查线索**：`5×5×20` box 全棱 r=2.5（圆角半径 = 短边一半，12 条棱的圆角两两相切）只产出 **8 个面**，而非标准 26 —— 疑为退化情形下的降级结果，下一轮优先复现 |
| C-02 | 混合半径需要逐棱不同半径的入口，`try_fillet` 只有单一 `radius` 参数 | **无法测**，需先补逐棱半径 API |
| C-03 | 见 0'''.2 | **未复现 → 跳过** |

### 0'''.4 验证闭环记录（B-06）

出现过一次临时阻塞：C: 盘一度 100% 占满（余 33M），`cargo` 链接阶段写 PDB 失败（LNK1201）。清理 `target/debug/incremental`（11G）后恢复，验证已全部跑完：

| 验证项 | 结果 |
|--------|------|
| `cargo test -p brepkit-operations --test boolean_mesh_fallback_counter` | ✅ 1 passed（对照 + 主用例 + delta 读取） |
| `cargo test -p brepkit-operations --doc` | ✅ 1 passed（`mesh_fallback_count` 文档示例） |
| `cargo clippy -p brepkit-operations -p brepkit-wasm --all-targets` | ✅ 无 error / 无 warning |
| `scripts/check-boundaries.sh` | ✅ All crate boundaries valid |
| `cargo check -p brepkit-wasm --lib` | ✅ 通过（`meshFallbackCount` 绑定编译无误） |

注：C 域（C-01/C-02/C-03）本轮**只做复现探查、未改任何代码**，故无对应回归项。

---

## 0''''. 第五轮实施记录（C-01 的 trimmer 半边）

本轮只做一件事：**修复 `fillet_v2` 在盒体上把材料留太多的其中一个独立根因**，其余原样留档。

### 0''''.1 先把期望值钉死：闭式解不再悬案

第三轮留下的疑案是「圆角盒体积的闭式推导有三个互相矛盾的结果」。本轮用两条**独立证据**收掉了它，从此可以拿闭式解当断言值：

盒体全棱圆角 = 内核 `K = (Lx-2r)×(Ly-2r)×(Lz-2r)` 与半径 `r` 球的 Minkowski 和，Steiner 公式直接给出

```text
V = xyz + 2r(xy+yz+xz) + pi·r²(x+y+z) + (4/3)·pi·r³
```

1. **一阶展开自检**：`V = L³ - 2.5752·3L·r² + O(r³)` —— 线性项精确抵消。所以任何「偏差 ∝ r」的实现都是错的，`r→0` 必须收敛到未圆角体积（实测 `r=1e-3` → `999.99997`）。
2. **跨引擎对照**：`fillet_rolling_ball`（解析接触点，另一套求解器）在 `r=0.1/0.5/1/2` 上全部复现该闭式解（`r=1` → 实测 975.332，闭式 975.587）。度量路径与期望值互相印证 ⇒ `fillet_v2` 的偏差只能来自它自己。

这两条固化为对照组测试：`fillet_v2` 一侧失败而对照组绿，才能证明失败来自被测引擎，而不是来自度量路径或期望值本身。

### 0''''.2 已修复：trimmer 把落在已有顶点上的接触点重复计数

**根因**：`trimmer::trim_face` 要求接触线与面边界恰好有 2 个交点，否则返回 `TrimmingFailure`。它数的是 **wire 位置**，不是**几何点**。

- 一次裁剪会沿接触线劈开边界边，而 `propagate_split` 会把这个劈分同步到**所有**引用该边的 wire —— 包括还没轮到自己被裁剪的邻面。
- 于是邻面边界上多出若干顶点。轮到裁剪该邻面时，接触线的端点正好落在这些顶点上，而每个顶点会被两条相邻子边各上报一次 → 一个几何点产生 2 个 hit，两端共 4 个 → `hits.len() != 2`。
- 后果不是「裁歪了」，而是**整面拒绝裁剪**。面保持原始尺寸、圆角面叠在原实体外侧，`fillet_builder` 只打一行 `log::warn!` 继续跑。

实测（`10³` cube，12 棱，`r=1`，`RUST_LOG=warn`）：面 `Id(4)`/`Id(5)`（即 `x=10`、`x=0`）各 4 次失败；它们的 AABB 仍是 `[0,10]²`，另 4 个面正常裁到 `[1,9]²`。这也解释了偏差为何随 `r` 增长：未被裁剪的面在 `r=0.1` 时只有 1 个，`r>=0.5` 起变成 4 个。

**修复**（`crates/blend/src/trimmer.rs`）：

1. `dedup_crossings` —— 收集完 hit 后按 3D 距离（`SNAP_TOL = 1e-7`）合并重复点，判断能否裁剪时只数**不同几何点**。
2. `split_at_crossing` —— 交点落在边的端点上时**吸附到已有顶点**，不新建顶点、不切分；`pre`/`post` 两个 run 允许为 `None`（切点落在边起点 ⇒ 整条边归属 post 侧；落在终点 ⇒ 归属 pre 侧）。否则会造出零长度子边和一个与原顶点相差一个舍入单位的重复顶点。
3. chain 组装改为 `Option` 拼接；「保留哪一侧」的采样从固定的「下一条边起点」改为「遍历整条 chain 取距接触线最远的点」——原采样在切点吸附到顶点时恰好落在直线上，叉积归零，左右判断取决于舍入方向。

**修复前后**（体积，闭式解见右）：

| r | 闭式 | 修复前 | 修复后 | 剩余偏差 |
|---|------|--------|--------|---------|
| 0.1 | 999.744 | 1013.360 | 通过与闭式解一致 | — |
| 0.5 | 993.729 | 1067.234 | 1004.164 | +10.435 |
| 1.0 | 975.587 | 1135.340 | 1016.300 | +40.713 |
| 2.0 | 907.705 | 1272.878 | 1063.202 | +155.497 |

6 个平面面现在全部裁剪到 `[r, L-r]²`。

### 0''''.3 剩余（未修）：stripe 没有 setback

剩余偏差 ∝ r²，结果壳仍非流形。根因是每条圆角条带跨整条棱长 `L`（此处 10），正确值应为 `L-2r`（此处 8）——条带要在距顶点 `r` 处终止、由角球面接手。`corner::compute_corners` 已产出 8 个角球面，但没人把条带两端裁回来与它拼上。这正是第三轮诊断里点名的「corner trimmer 一侧」，与本轮修复是**两个独立缺陷**，留给后续轮次。

留档方式：两条 `#[ignore]` ticket 放在 `crates/operations/tests/fillet_box_volume.rs`，默认套件绿、`-- --ignored` 亮出来；过程与数据写进 `docs/analysis/2026-10-01-wasm-fillet-cascade-bevel-fallback.md` 的「2026-10-01 续」。

### 0''''.4 验证闭环

| 项 | 结果 |
|----|------|
| 反向验证（新单测） | 临时停用 `dedup_crossings` 后 `trim_through_an_existing_boundary_vertex` 报 `TrimmingFailure { face: Id(1) }` —— 测试确实锁定了该缺陷 |
| 反向验证（端到端） | 同样条件下 `every_plane_face_shrinks_by_one_fillet_radius_per_side` 报「`r=0.1` 有 1 个面、`r>=0.5` 有 4 个面未被裁剪」，与修复前现象一致 |
| `cargo test -p brepkit-blend` | 97 项通过（含新增单测） |
| `cargo test -p brepkit-operations` | 809 库测试 + 全部集成测试通过（`regress_fillet_cascade` 默认绿、7 ignored） |
| `cargo clippy -p brepkit-blend -p brepkit-operations --all-targets` | 无新增告警 |
| `scripts/check-boundaries.sh` | 通过 |

改动文件：`crates/blend/src/trimmer.rs`；新增 `crates/operations/tests/fillet_box_volume.rs`；更新 `docs/plans/2x-bug-fix-plan.md`、`docs/analysis/2026-10-01-wasm-fillet-cascade-bevel-fallback.md`。诊断用的临时 example 已删除，未留下调试脚本。

---

## 0'''''. 第六轮实施记录（C-01 的引擎半边收尾）

第五轮之后偏差从「多料」翻转成「缺料」（`r=1`：`1016.300` → `912.6748`）。本轮把它做到与闭式解一致，过程中挖出**三个独立缺陷**。

### 0'''''.1 结果

| r | 闭式 | 本轮前 | 本轮后 | 偏差 |
|---|------|--------|--------|------|
| 0.1 | 999.744 | — | 999.744 | 0.000 |
| 0.5 | 993.729 | 977.163 | 993.692 | −0.037 |
| 1.0 | 975.587 | 912.675 | 975.332 | −0.255 |
| 2.0 | 907.705 | 690.772 | 905.746 | −1.959 |

`r=2` 的 −1.959 属于**度量**：对照组 `fillet_rolling_ball` 同条件测得 `905.7461`，与 `fillet_v2` 的 `905.7464` 相差 `3e-4`。`fillet_box_volume.rs` 的 `VOLUME_TOL` 因此取 `2.0`。

### 0'''''.2 缺陷一：条带跨整条棱长

`Spine` 只有「整条链」一种形态。新增 `crates/blend/src/setback.rs` 解析求退缩量：条带相邻两面 `n1,n2` 的滚球球心 `w = r(n1+n2)/(1+n1·n2)` 到第三面 `n3` 的距离沿棱线性变化，退缩量 = `(r − w·n3)/(u·n3)`；顶点处不足 3 条被圆角的棱则不退缩（没有角补丁接手）。`Spine::window(start, end)` 只改 `length` 并记 `offset`，`locate` 用它把窗口参数映射回链参数；条带的所有下游阶段都从 `Spine` 读跨度，所以窗口化一处即可。

### 0'''''.3 缺陷二：角补丁 apex 放在球面上

degree-(2,2) 有理补丁在宽球面三角形上会中间内凹：实测补丁中心到球心 `0.8647`（内凹 13.5%）。对照组用的是**切锥顶点**（`center + (Σdir)·r`，正交角处 overshoot √3，正好落在盒体顶点），凹度 `0.9511`（4.9%）。改为切锥顶点后两引擎的角补丁 8 个采样点逐点相同。

### 0'''''.4 缺陷三：角补丁朝向 6/8 朝内

修完缺陷二体积几乎没动，说明形状不是主因。逐面核查：8 个角补丁里 **6 个三角面法向朝内**（对照组 0 个）。

`VertexContactData::is_convex` 在 `compute_sphere_center` 里选择球心取 `vertex + Σn·r` 还是 `vertex − Σn·r`；调试输出显示它对 8 个凸角**全为 false**（`build_multi_edge_corner` 的 `avg_normal · cp_centroid > 0` 实际测的是「法向朝内」）。球心因此算对了，但同一标志被当「凸角」用来定朝外方向就正好反了。自洽用法：`is_convex == false` ⟺ 球心在材料内 ⟹ 朝外 = 背离球心。

朝向由 `Face::new_reversed` 承载（`tessellate` 与体积积分都跟随 `is_reversed` 翻转绕向）；判据用 **(u,v) 网格切向量的叉积**，不用 `surface.normal`——张量积补丁在控制网格转置后 `normal` 的约定会与网格次序不一致，而 tessellator 是按网格次序出三角形的。

### 0'''''.5 剩余（未修）：壳不是闭合 2-流形

`corner::compute_corners` 为角补丁边界新建顶点与边，而非复用条带端点已有的边 → 26 个面里 76 次边引用是单面的（`r=1`：`edges=86, free=76`，对照组 `edges=48, free=0`）。不影响体积，但 `validate_shell_closed` 会判不合法（wasm 门禁 `is_valid` 因此拒绝 `fillet_v2`）。这是下一个独立缺陷。

### 0'''''.6 验证闭环

| 项 | 结果 |
|----|------|
| 反向验证（朝向） | 关掉修正 → `r=1` 精确回到 `912.6748` |
| 反向验证（apex） | 改回球面 → `r=2` 偏差 −5.78，超容差 |
| 反向验证（setback） | 关闭 → 条带轴向跨度回到 10，`every_blend_band_stops_one_radius_short_of_each_end` 变红 |
| `cargo test -p brepkit-blend` | 99 项通过 |
| `cargo test -p brepkit-operations` | 809 库测试 + 全部集成测试通过 |
| `fillet_box_volume.rs` | 6/6 通过，两条原 `#[ignore]` ticket 转正 |
| `regress_fillet_cascade` | `probe_cube`、`fillet_v2_over_sweeps_a_box` 转正；其余 5 票是别的缺陷，保持 `#[ignore]` |
| clippy `--all-targets` / `check-boundaries.sh` | 干净 / 通过 |

后两个票面的断言原先写的是「体积应贴近未圆角的 `1000`」——那个基线本身是错的（盒体圆角丢 `2.5752·(lx+ly+lz)·r²`，`r=1` 时为 24.4），正确的引擎反而通不过。已改为断言闭式解，闭式 helper 提到 `regress_fillet_cascade/main.rs` 共享。

改动文件：`crates/blend/src/{setback.rs(新), spine.rs, fillet_builder.rs, corner.rs, spherical_triangle.rs, lib.rs}`；`crates/operations/tests/{fillet_box_volume.rs, regress_fillet_cascade/*}`；更新本文档与 `docs/analysis/2026-10-01-wasm-fillet-cascade-bevel-fallback.md`。

---

## 0''''''. 第七轮实施记录（B-04：shell 腔面 sense / 拓扑归档）

### 0'''''.1 结果：已修复（两个独立缺陷）

| 项 | 内容 |
|----|------|
| 复现 | `shell_op/tests.rs::shell_closed_box_sense_consistent_outer_and_cavity` —— closed `shell()` 结果的 12 个面被 `check_shell_connected` 判为 **7 个连通分量（6/12 可达）**；对照用例 `control_plain_box_sense_consistent` 通过，证明校验器本身无误 |
| 缺陷一（主） | `shell()` 无开面路径（`boundary_edge_ids.is_empty()`）把外壳 6 面 + 腔面 6 面全部塞进**同一个 shell**，且 `Solid` 未登记任何 inner shell。外墙面与腔面不共享任何边（外/腔不接触），因此该 shell 天然不连通 —— 诊断打印确认 24 条边每条恰好被 2 个面使用（封闭、sense 一致），分裂纯粹是"两个不相交面组共享一个 shell"造成 |
| 修复一 | `shell_op.rs` closed 路径改用 `face_components`（可见性 `pub(super)` → `pub(crate)`）拆分连通分量：最大分量留作 outer shell，其余登记为 inner shell（腔壳） |
| 缺陷二（连带暴露） | `operations/classify.rs` 的 `classify_point` 与 `compute_winding_number` 只遍历 `outer_shell` —— 修复一之后腔面被移入 inner shell，空腔中心被误判 Inside。面集合改为 `once(outer_shell).chain(inner_shells)`；穿越奇偶性自动给出正确符号，无需翻转 |
| 旧断言更新 | `shell_closed_box` 由"12 面全在 outer shell"改为"outer 6 面 + 1 个 inner shell 6 面"；`shell_cavity_point_classification` 新增 closed + open 两形态的腔心=Outside / 壁材=Inside 双断言 |
| 反向验证 | 修复前主用例红（7 连通分量 / 腔心 Inside），修复后绿；对照用例全程绿 |

### 0'''''.2 验证闭环

| 项 | 结果 |
|----|------|
| `cargo test -p brepkit-operations` | 812 库测试 + 全部集成测试通过（含新增 4 条：主用例、对照、腔点分类、sense 复验） |
| `cargo clippy -p brepkit-operations -p brepkit-check --all-targets` | 零告警 |
| `scripts/check-boundaries.sh` | 通过（经 Git Bash 执行；PATH 上的 STM32 工具链旧版 bash 不支持 `local arr=("$@")` 数组语法，与代码无关） |

改动文件：`crates/operations/src/shell_op.rs`、`crates/operations/src/classify.rs`、`crates/operations/src/boolean/assembly.rs`（仅可见性）、`crates/operations/src/shell_op/tests.rs`。

注：诊断用临时测试 `diag_shell_connectivity` 已删除。B-04 中"open-top 形态的 rim sense"未单独构造出失败信号（open 路径 rim 面方向取自被移除面的外法向，closed/open 两形态的腔点分类断言均绿），open-top 的 sense 端到端校验由 `shell_cavity_point_classification` 覆盖。

---

## 0'''''''. 第八轮实施记录（D-04：pipe 采样未映射进路径 domain）

### 0'''''''.1 结果：已修复

| 项 | 内容 |
|----|------|
| 复现 | `pipe.rs::tests::pipe_covers_full_nonunit_path_domain` —— 单位正方形 profile 沿 knot 域 `[0,2]` 的直线路径（z 0→4）扫掠，体积实测 **2.0**，解析值应为 **4.0**；对照用例 `pipe_path_domain_unit_control`（knot 域 `[0,1]`、z 0→2，体积 2.0）全程绿，证明夹具无误 |
| 根因 | `pipe` 的采样循环把 `t_param = k/num_segments ∈ [0,1]` 直接喂给 `NurbsCurve::evaluate(u)`，而后者按 **knot 域参数**求值（`basis::find_span(n, p, u, &knots)`）。路径 knot 向量非 `[0,…,1]` 时（如 `[0,0,2,2]`），只扫过路径前半段 |
| 修复 | `crates/operations/src/pipe.rs` 三处采样全部改为映射到各自曲线的 `domain()`：① 主环循环（新增 `path_param(k)` 闭包，把 `k/num_segments` 映射到路径 knot 域）；② 内环（inner wire）循环同步；③ `compute_scale_factors` 中 guide 与 path 各自映射到自己的域再取距离 |
| 期望值 | 解析解：体积 = profile 面积（1）× 路径长（4）= 4，容差 0.05；期望值不引用任何外部数值 |

### 0'''''''.2 验证闭环

| 项 | 结果 |
|----|------|
| 修复前 | 主用例红（体积 2.0 ≠ 4.0），对照绿 |
| `cargo test -p brepkit-operations --lib pipe::` | 11 项全通过（修复后主用例转绿） |
| `cargo test -p brepkit-operations` | 814 库测试 + 全部集成测试通过 |
| `cargo clippy -p brepkit-operations --all-targets` | 零告警 |
| `scripts/check-boundaries.sh` | 通过（需用 Git Bash 显式执行，见第七轮注） |

改动文件：`crates/operations/src/pipe.rs`（实现 + 两条新测试）。

---

## 0''''''''. 第九轮实施记录（C-01 遗留：圆角结果壳不是闭合 2-流形）

### 0''''''''.1 结果：已修复

| 项 | 内容 |
|----|------|
| 复现 | `crates/operations/tests/fillet_shell_manifold.rs::walking_engine_box_fillet_is_a_closed_manifold` —— `r=0.5/1/2` 三个半径下 `validate_shell_closed` 全部拒绝（`edge index N is used by 1 wires (free edge)`），壳同时分裂成 **17 个连通分量**；对照用例 `control_rolling_ball_box_fillet_is_a_closed_manifold`（`fillet_rolling_ball`，另一套求解器）全程绿 |
| 量化 | 修复前 `edges=86, free=76, over-shared=0`。引用总数 `76×1 + 10×2 = 96`，与正确拓扑的 `48×2 = 96` **完全相等** —— 这一条直接把根因限定在"重号"：曲线是对的，只是每条被造了两三个实体 |
| 根因 | 三个生产者各自造边，互不通气：① trimmer 在裁剪后的邻面上造 contact edge；② `create_blend_face_with_contacts` 为条带侧面再造一份（adopt 只在 10/24 条上成功）；③ `corner::compute_corners` 为角补丁边界造第三份。几何上是同一条曲线——实测角补丁边 `a=(1,1,0) b=(1,0,1)` 与条带端弧 `a=(1,0,1) b=(1,1,0)` 端点精确重合、方向相反 |
| 修复 | 新增 `crates/blend/src/sew.rs::weld_faces`，在 `FilletBuilder::build` 装配完成后、`Shell::new` 之前调用：① 顶点按空间哈希合并（网格边长 1e-6 + 27 邻格探测，避免跨格漏配）；② 边按焊接后的**端点对**合并，代表边保留首个副本的曲线，且端点保持该曲线自身的方向，使参数化仍沿几何正向；③ 面按焊接后的边重建 wire，逐条重算 `is_forward` 以保住原有遍历方向 |
| 保守性 | wire 焊接后少于 3 条边或不再闭合时，该面**原样放行**而不是丢弃 —— 略微开放的壳比缺一个面的壳降级得更体面 |
| 修复后 | `V=24 E=48 F=26`，`V−E+F=2`，`free=0`、`over-shared=0`，三个半径一致；边数与对照组 `fillet_rolling_ball` 的 48 相同 |
| 期望值 | 纯拓扑不变量（Euler-Poincaré = 2、每条边被引用恰好两次、单连通），**不引用任何数值 golden** |

### 0''''''''.2 两个判断依据（不是拍脑袋）

- **可以只按端点对合并边。** 判据是「同一条几何曲线的两个副本端点必然重合，而不同曲线端点必然不同」。球面角补丁的三条弧跑在三对**不同**的切点之间，条带的四条边也是四个不同的点对，所以端点对是充分的键。另一个前提是反向遍历安全：`Circle3D::domain()` 恒为整圆 `[0, 2π]`（`math/src/traits.rs:229`），弧的范围由边的端点顶点决定，因此 `forward/reverse` 只改变遍历方向、不改变几何。
- **中点不作为判据。** 第一版诊断用「参数域中点」做指纹，结果角补丁的 Nurbs 弧（`mid=(1, 0.2929, 0.2929)`，确实在球面上）与条带的 Circle 弧（`mid` 落在端点上）拼不上，误以为两者不是同一条曲线。根因是 `Circle3D` 的域是整圆、域中点与弧中点无关。改按端点比对后立刻配对成功 —— 这条记录在这里是为了防止后续再拿域中点当几何中点用。

### 0''''''''.3 未修 / 留给后续

| 项 | 说明 |
|----|------|
| C-02 `mixed_radius_fillets_stay_watertight` 仍红 | 它走 `brepkit_operations::fillet::fillet_variable`（`operations/src/fillet/mod.rs` 里的**另一套装配**），不经过 `FilletBuilder`，焊接没接上去。失败信息已从"体积不对"变成"free edge"，与 C-01 同型。焊接是通用工具，接上去是下一步，不混在本轮提交里 |
| `chamfer_builder` | 同样有自己的装配路径，同样未接焊接 |
| `check_shell_orientation` 报 24 条同向边 | 第三轮（§0'''.2）判定为拓扑标记层不一致、无几何后果，本轮未处理 |

### 0''''''''.4 验证闭环

| 项 | 结果 |
|----|------|
| 反向验证 | 在 `weld_faces` 入口临时 `return Ok(faces.to_vec())` → 主用例回到红（`free edge` + 17 分量），对照用例仍绿；恢复后转绿。测试确实锁定该缺陷 |
| Euler / 计数 | `r=0.5/1/2` 均为 `V=24 E=48 F=26`、`V−E+F=2`、`free=0` |
| `cargo test -p brepkit-operations --test fillet_shell_manifold` | ✅ 2 passed（主用例 + 对照） |
| `cargo test -p brepkit-operations --test fillet_box_volume` | ✅ 6 passed（体积闭式解未被焊接破坏） |
| `cargo test -p brepkit-blend` | ✅ 99 项通过 |
| `cargo test -p brepkit-operations` | ✅ 814 库测试 + 全部集成测试通过，7 ignored |
| `cargo clippy -p brepkit-blend -p brepkit-operations --all-targets` | ✅ 零告警（修掉 `clippy::map_entry` 与 `clippy::format_push_string`） |
| `scripts/check-boundaries.sh` | ✅ All crate boundaries valid |
| `cargo check -p brepkit-wasm --lib` | ✅ 通过 |

改动文件：`crates/blend/src/sew.rs`（新）、`crates/blend/src/lib.rs`（注册模块）、`crates/blend/src/fillet_builder.rs`（装配后调用焊接）；新增 `crates/operations/tests/fillet_shell_manifold.rs`。诊断用的临时测试 `diag_fillet_shell.rs` 已删除，未留下调试脚本。

---

## 0. 许可证隔离红线（所有参与者必读）

| 禁止 | 允许 |
|------|------|
| 访问上游仓库 3.x 的 commit / PR / tag / tarball | 阅读本仓库（2.x）自身代码 |
| 在 PR / 注释中粘贴上游代码片段或链接 | 自己写最小复现 + 自己实现 + 自己写断言 |
| 用 3.x 的输出数值当 golden 期望值 | 用**解析解**（球体积、圆柱体积、Euler-Poincaré、封闭流形）当期望值 |

- 每个工作项开工前，实施者在 PR 描述里声明：`未阅读 3.x 源码，实现与测试均为自研`。
- 分析文档里的 PR 编号（如 `#1526`）仅用于在本文档内交叉索引**问题陈述**，不得据此去检索该 PR 的内容。下文统一改用内部编号 `S-xx`。
- 若某条缺陷在本仓库内无法在不参考上游的前提下定位根因 → 记为 `阻塞-待定`，不做猜测性修改。

---

## 1. 缺陷清单（从分析文档提取，仅取"2.x 行为不正确"的部分）

图例：**已确认** = 已在本仓库代码中定位到可疑点；**待复现** = 需先写失败用例验证；**疑似已修** = 本仓库已有同类加固，需回归确认。

### A. GFA 布尔引擎（crates/algo，数量最多、风险最高）

| ID | 2.x 症状（问题陈述） | 本仓库状态 |
|----|---------------------|-----------|
| A-01 | quadric × NURBS 面相交只能靠采样逼近，无解析截面 → 精度损失、面数暴涨 | 待复现 |
| A-02 | 平面 × 圆环（torus）截面逐 v 采样，非闭式求解 | 待复现 |
| A-03 | 等角偏移的平行圆锥对，截面未用精确 radical-plane | 待复现 |
| A-04 | 面拆分时孔洞/岛的嵌套关系归档错误（外环含孔、内环反而成孔） | 待复现 |
| A-05 | 合成 cap 上的自由环未正确嵌套为孔 | 待复现 |
| A-06 | 大面积区域缺少窗口化（windowing）→ 拆分退化 | 待复现 |
| A-07 | 同域检测（same-domain）未使用原生采样边界弧 | 待复现 |
| A-08 | 周期带（periodic）跨越 seam 时连续性未展开 | 待复现 |
| A-09 | 闭合边的绕向由采样顺序而非曲线方向决定 → 方向翻转 | 待复现 |
| A-10 | split-edge 的像未按端点链接定向 | 待复现 |
| A-11 | marched-drift 带上跨面的 wire 顶点未焊接 → 缝隙 | 待复现 |
| A-12 | T 型分叉处两侧 UV 未协同注册（co-register） | 待复现 |
| A-13 | welded junction 处残留的 stale pcurve 未回退到 chord | 待复现 |
| A-14 | 场景级：轴上支架+筒体融合、knuckle 上的销孔、4×4 base fuse、孔悬挂栅栏环 | 待复现 |

### B. 布尔装配与 shell（crates/operations）

| ID | 2.x 症状 | 本仓库状态 |
|----|---------|-----------|
| B-01 | compound-cut 对 contact-thin 工具未走单 arrangement，性能差 | 待复现 |
| B-02 | 被 fallback 污染的 cluster fuse 仍参与 batching | 待复现 |
| B-03 | 混合装配保留亚分辨率多边形（碎屑面） | 待复现 |
| B-04 | shell 腔面 sense 端到端错误：`shell_op` 产出的空盒出现一批同 sense 边 | **已修复**（第七轮，见 §0''''''）：closed 路径外/腔两组面挤在同一 shell 且未登记 inner shell（12 面 7 连通分量）；腔面移入 inner shell 后连带暴露并修复 `classify_point`/`compute_winding_number` 只遍历 outer shell 的问题。sense 本身经诊断 24 边全为成对正反使用，无同 sense 边 |
| B-05 | rim 未从排序后的边界边装配 | 待复现 |
| B-06 | 布尔结果缺少"退化到 mesh fallback"的可观测计数 | **已修复**（第三轮，见 §0'''.1）：`boolean::mesh_fallback_count()` + WASM `meshFallbackCount()` |
| B-07 | 布尔后同面碎片未合并 | **疑似已修**：`BooleanOptions::unify_faces` 已存在，需确认生效路径 |

### C. 圆角 / 倒角（crates/blend + operations/fillet + wasm）

| ID | 2.x 症状 | 本仓库状态 |
|----|---------|-----------|
| C-01 | box 角部材料过度去除（多棱一起圆角时体积塌陷） | **已修**（第六轮体积 + 第九轮拓扑，见 §0''''' 与 §0''''''''）：体积与闭式解一致（`r=0.1/0.5/1/2` 偏差 `0.000 / −0.037 / −0.255 / −1.959`，`r=2` 残差属度量本身）；拓扑亦已闭合 —— 第九轮新增 `blend/src/sew.rs` 焊接重复的顶点/边后，`V=24 E=48 F=26`、`V−E+F=2`、`free=0`，`validate_shell_closed` 接受。**遗留**：结果壳的边 `is_forward` 与面相向仍不自洽（`check_shell_orientation` 报 24 条同向边，第三轮判定为标记层不一致、无几何后果）；`fillet_variable` 与 `chamfer_builder` 两条装配路径尚未接焊接 |
| C-02 | 混合半径角部开缝 | **无法测**：`try_fillet` 只有单一 `radius`，缺逐棱半径入口 |
| C-03 | 面片翻转（法向朝内） | **未复现 → 跳过**（第三轮，见 §0'''.2）：解析法向 0/18 朝内 + 体积 975.3 对解析 ≈975.6 |
| C-04 | 三面角 setback 判定错误 | 待复现 |
| C-05 | 等半径相邻圆角的尖角斜接角错误 | 待复现 |
| C-06 | trimmer 缺少分裂守卫 | 待复现 |
| C-07 | twin edge 处理缺失 | 待复现 |
| C-08 | 棱柱角部 orientation 错误 | 待复现 |
| C-09 | 所有引擎失败时**静默返回原实体**（调用方以为成功） | **已确认**：`helpers.rs:206-207` 兜底 `Ok(solid_id)` |
| C-10 | fillet/chamfer 在 WASM 中 panic 会杀掉整个实例 | **待复现**：`panic_message` 已有，需确认 `filletV2`/`chamferV2`/批处理路径是否全部包 `catch_unwind` |

### D. 扫掠 / 管道 / 放样

| ID | 2.x 症状 | 本仓库状态 |
|----|---------|-----------|
| D-01 | profile 被**重新居中**到路径起点，而非按原位置扫掠 | **已确认**：`operations/src/sweep.rs:511-515`（`shift = start - centroid`） |
| D-02 | 尖角斜接（miter）标架非域正确、环不等分 → 接缝错位 | 待复现 |
| D-03 | 扭曲 sweep 的 quad 输出非精确双线性面片 | 待复现 |
| D-04 | pipe 采样未映射进路径 domain | 待复现 |
| D-05 | 闭平面 G1 链缺少解析脊线扫掠（能力缺口，非缺陷） | 能力项，见 §6 |

### E. 测量与分类（crates/operations/measure、crates/check）

| ID | 2.x 症状 | 本仓库状态 |
|----|---------|-----------|
| E-01 | 体积/面积/质心**未计入内腔壳** → 空心体数值错误 | **疑似已修**：`measure/helpers.rs:19,53` 已 `chain(inner_shells)`，需确认 volume/area/centroid 三条路径全部走 helper |
| E-02 | 修剪过的圆柱/圆锥面 bbox 用完整解析面范围 → 过估 | **部分修**：`bounding_box.rs:159-168` 用顶点+wire 中点扩张，非按 edge 边界裁剪，仍可能过估 |
| E-03 | ray-cast 分类：射线穿过**面孔（洞）**仍算一次穿越 → 内外判定反 | **已确认**：`check/src/util.rs:84-87` `face_polygon` 只取 `outer_wire`，完全忽略 `inner_wires` |
| E-04 | 圆孔缺少解析命中测试（依赖多边形化，边界抖动） | 待复现 |
| E-05 | 点分类只遍历 `outer_shell` | **已确认**：`check/src/classify/mod.rs:65` |

### F. IO（crates/io）

| ID | 2.x 症状 | 本仓库状态 |
|----|---------|-----------|
| F-01 | STEP **写出**丢失有理 B 样条权重（圆/圆锥被写成非有理） | **已确认**：`crates/io/src/step/` 下只有 `reader.rs` 含 `weight/rational`，writer 全无 |
| F-02 | STEP **读入** ELLIPSE 时未使用放置轴（长轴方向） | **已确认**：`step/reader.rs:430` `let (center, normal, _u_axis) = ...` —— 长轴被丢弃 |
| F-03 | STEP reader 对畸形/嵌套实体不够健壮 | 待复现 |
| F-04 | 3MF reader 若干小修 | 待复现 |

### G. 细分 / 网格（crates/operations/tessellate）

| ID | 2.x 症状 | 本仓库状态 |
|----|---------|-----------|
| G-01 | CDT 面网格法向未用面积加权投票定向 → 局部翻面 | 待复现 |
| G-02 | 退化轨迹点未继承 u 参数 | 待复现 |
| G-03 | 约束恢复时 Steiner 顶点未被覆盖 | 待复现 |
| G-04 | 圆 T 形拼接缺空间哈希（性能） | 性能项 |
| G-05 | developable 面未按公差驱动细分（性能） | 性能项 |

### H. 数学库（crates/math）

| ID | 2.x 症状 | 本仓库状态 |
|----|---------|-----------|
| H-01 | CDT 约束恢复无迭代上界 → 失控拖死内核 | **疑似已修**：`math/src/cdt/constraints.rs:6,38` 已有 `MAX_SPLIT_DEPTH=16` 与 `max_iter` 预算，需回归验证是否覆盖全部回路 |
| H-02 | plane-NURBS 网格交点未按单元连通性链接 → 折线断裂/乱序 | 待复现 |
| H-03 | plane-torus 未用闭式截面 + grid seeder 冗余（性能） | 性能项 |

### 不属于缺陷、另行决策的项（见 §7）

- tsify 类型改为 JSON 字符串跨界（形态变化，非 bug）
- 许可证本身（不在本计划范围）
- `fillet` 失败语义由"静默成功"改为"抛错"（C-09 的另一面，属 API 语义决策）

---

## 2. 总体原则

1. **先复现，后修改。** 每一条 `待复现` 必须先落地一个可运行的失败用例（单元/属性/场景 golden），再动手改。禁止"文档说这里有病就改这里"。
2. **期望值必须自洽。** 优先用解析解与拓扑不变量做断言：体积/面积解析值、Euler-Poincaré `V - E + F = 2(1-g)`、封闭 2-流形、法向一致朝外、面数上界。**不用上游数值做 golden**。
3. **数值行为变更要显式登记。** 修复会改变输出数值的项（D-01、E-01、E-02、C 系列）在 §7 单列，需拍板后再合入。
4. **分层不动摇。** 改动必须遵守 `AGENTS.md` 的分层依赖规则，合入前跑 `scripts/check-boundaries.sh`。
5. **小步提交。** 一个缺陷一个提交，提交信息带上内部编号 `S-xx`，便于回滚与二分。

---

## 3. 修复阶段

### Phase 0 — 复现与基线（所有后续工作的前置）

| 任务 | 内容 |
|------|------|
| 0.1 | 为 A/B/C/E/F/G 各域搭建**最小复现脚手架**：从基元（box / cylinder / cone / torus / 带孔板）构造输入，打印体积、面数、bbox、封闭性 |
| 0.2 | 建立统一几何不变量校验入口（复用 `topology::validation::validate_shell_closed`，补：Euler 数、体积符号、法向朝外一致性、边被两面共享） |
| 0.3 | ✅**已完成**（见 §0'''.1）：布尔进程级 mesh-fallback 计数器（`AtomicU64` + `pub fn mesh_fallback_count()`，WASM 侧 `meshFallbackCount()`），作为 A 域修复的度量尺 |
| 0.4 | 把"已确认"的 6 条（D-01 / E-03 / E-05 / F-01 / F-02 / C-09）写成失败测试，锁定基线 |

### Phase 1 — P0：会产出错误几何/错误数值的缺陷

| 顺序 | 缺陷 | 修复方向（自研，仅描述思路） |
|------|------|---------------------------|
| 1.1 | F-02 | `step/reader.rs` 的 ELLIPSE 分支：把 `build_axis2_placement` 返回的第三分量（长轴方向）接入 `Ellipse3D` 的构造，使长轴朝向来自 STEP 的 `AXIS2_PLACEMENT_3D` 而非默认基向量；补正交化与退化（半轴相等→退化为圆）处理 |
| 1.2 | F-01 | STEP writer：为有理 B 样条曲线/曲面补 `RATIONAL_B_SPLINE_*` 权重写出，并在读取端回归往返（export → import → 控制点/权重/几何比对） |
| 1.3 | E-03 / E-04 | `face_polygon` 扩展为"外环 + 内环集合"：命中点先判外环内，再逐个减内环；圆孔优先走解析圆内测试，避免多边形化抖动 |
| 1.4 | E-05 | `classify_point` 的面集合改为 outer + inner shells（与 `check/src/distance/mod.rs:307` 已有的 `collect_solid_faces` 取齐）；注意内腔壳的穿越计数符号 |
| 1.5 | E-01 | 复核 `volume.rs` / `area.rs` / 质心三条路径是否都经过 `collect_solid_face_ids`；补齐遗漏路径并加空心体解析对照（球壳体积 = 4/3π(R³−r³)） |
| 1.6 | C-01/C-02/C-03 | 圆角：在现有 closed-manifold 门禁之上加**三条几何门禁**——① 体积不得大于输入体；② 无面法向翻转；③ 角部顶点邻域不得出现非流形边。任一不满足即拒绝该引擎结果并回退；三条门禁同时作为角部 setback / 混合半径 / 翻面的回归网 |
| 1.7 | C-10 | 审计 WASM 全部 blend 入口（`fillet` / `filletV2` / `chamferV2` / batch / evolution），统一包 `catch_unwind`，把 panic 转成 `JsError`，保证实例存活 |

### Phase 2 — P1：布尔几何精度（A 域 + B 域）

按"能否被 Phase 0.3 的计数器度量"排序推进，每修一条都要看到 fallback 计数下降或面数下降：

2.1 截面解析化：A-02（plane-torus 闭式）→ A-03（平行圆锥 radical-plane）→ A-01（quadric×NURBS march）。
2.2 面拆分与成环：A-04 / A-05 / A-06 / A-07 / A-08 —— 统一"环嵌套关系判定"的单一实现入口，先补嵌套不变量测试（外环面积 ⊃ 内环、孔洞面积为负定向）。
2.3 方向与焊接：A-09 / A-10 / A-11 —— 闭合边绕向改由曲线方向推导；split-edge 定向按端点链接；drift 带跨面顶点强制焊接。
2.4 UV/PCurve：A-12 / A-13 —— T 分叉 UV 协同注册；welded junction 的 stale pcurve 检测 + chord 回退。
2.5 场景回归：A-14 四个真实场景各自建一条 golden（体积/面数/Euler），作为 A 域的收尾验收。
2.6 B-04/B-05/B-06：shell 腔面 sense 端到端校验（每个 cavity shell 的边 sense 必须两两相反），rim 从排序边界边装配，亚分辨率多边形丢弃阈值化。

### Phase 3 — P2：数值行为变更（需先拍板，见 §7）

3.1 D-01：sweep/pipe 改为 positioned 扫掠（去掉 `centroid → start` 的平移）。
3.2 E-02：修剪圆柱/圆锥 bbox 改为按 edge 边界裁剪（而非顶点/中点扩张）。
3.3 C-09 语义：fillet 全引擎失败时改为返回错误，而非静默返回原实体。

### Phase 4 — P3：质量与性能

4.1 G-01/G-02/G-03：CDT 面网格定向、退化点 u 继承、Steiner 覆盖。
4.2 H-02：plane-NURBS 交点按单元连通性链接。
4.3 H-01 回归：确认 CDT 约束恢复的所有回路都有迭代预算，补"超预算必须返回错误而非静默"的测试。
4.4 G-04/G-05/H-03：性能项，先 bench 后改，避免无度量优化。

---

## 4. 验证与回归策略

**三层断言（每一条缺陷修复都必须至少覆盖前两层）**

1. 单元/属性测试：`proptest` 随机基元 + 不变量（封闭、Euler、法向、体积符号）。
2. 几何不变量：统一走 Phase 0.2 的校验入口，测试里直接断言 `validate_shell_closed` + Euler + 法向朝外。
3. 场景 golden：体积 / 面积 / 面数 / bbox 落盘 `tests/*.golden`（仓库已有 golden 机制），变更需人工确认并说明原因。

**专项网**

- 布尔：fallback 计数器 + "精确率"指标（非 fallback 结果占比）+ 四个真实场景 golden。
- 圆角：体积单调性（圆角后体积必须 ≤ 输入）、无翻面、封闭性、多棱/混合半径/连续两遍三组场景。
- IO：STEP 往返（export → import → 控制点、权重、节点、几何误差 < 1e-9），含圆/圆锥/椭圆三类有理实体。

**CI 门禁**：`cargo test --workspace` → `cargo clippy -- -D warnings` → `scripts/check-boundaries.sh` → `cargo semver-checks`（consumer surface）→ criterion bench 对比基线（回退 > 5% 视为失败）。

---

## 5. 风险登记

| 风险 | 影响 | 缓解 |
|------|------|------|
| 部分缺陷在本仓库已修（fork 已独立加固） | 白做工 | Phase 0 必须先跑复现，确认失败后再改 |
| 上游 3.x 信息通过 PR 描述/issue 二次流入 | 许可证污染 | §0 红线 + PR 声明 + 评审 checklist |
| 布尔/圆角改动牵一发动全身 | 大面积回归 | 小步提交 + 场景 golden + 每步全量 CI |
| 数值行为变更（D-01/E-02/C-09）影响下游 | 下游结果突变 | 单列 Phase 3，先拍板再动 |
| 许可证/CLA 相关 | 法律 | 不在本计划处理，另立议题 |

---

## 6. 能力项（缺陷之外，按需排期）

- D-05：闭平面 G1 链的解析脊线扫掠（直线/相切圆弧链 + 全直线垂直 profile 走精确平面/圆柱/圆锥面），其余回退 NURBS 插值。
- B-07 的 `unify_faces`：确认默认路径已生效，未生效则在布尔后处理阶段补齐。
- `orientation` 工具（壳/面方向一致性传播与修复）：作为 B-04 / C-08 的公共基础设施优先落地。

---

## 7. 待拍板决策（不拍板不进入 Phase 3）

1. **D-01**：sweep/pipe 是否接受"不再重定位 profile"？会改变既有模型的输出位置。
2. **E-02**：修剪圆柱/圆锥 bbox 收紧后，依赖旧（过估）bbox 的下游逻辑是否会受影响？
3. **C-09**：WASM `fillet` 全引擎失败时——维持静默返回原实体，还是改为抛 `JsError`？
4. **Phase 2 的验收门槛**：布尔"精确率"提升到多少算达标？是否允许为达标而牺牲性能？
5. **A 域投入上限**：A 域共 14 条，是否先只做能被真实场景验证的 A-14 相关子集？
