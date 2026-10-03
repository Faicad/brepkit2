# brepkit2 降级类代码审计（Degradation / Fallback Audit）

> 目的：把项目里所有"优选精确路径失败 / 不可用 / 不安全时，改用近似或尽力而为路径"的逻辑（即**降级类代码**）全部列出来，便于逐步修复。
> 生成时间：2026-10-01。范围：`crates/*/src/**/*.rs`（已排除 `*/tests/`、`*/examples/`）。
> 严重度排序：Tier 1 直接丢失解析几何 / 产生非流形 → Tier 2 数值近似、可能误判 → Tier 3 仅性能或安全网。

---

## 0. 怎么定位 / 度量这些降级

代码里已经埋了两类"探针"，可作为巡查锚点：

- **`log::debug!(target: "brepkit_approx", …)`**：所有显式的"降级已发生"日志都打这个 target。
  - 现有统计工具：`crates/operations/examples/approx_census.rs`（跑一遍就能数出每个降级被触发了多少次）。
- **`MESH_FALLBACK_COUNT`**（`crates/operations/src/boolean/mod.rs`）：全局原子计数器，每次进入布尔网格回退 +1；JS 侧通过 `crates/wasm/src/bindings/booleans.rs` 的 `meshFallbackCount` 暴露。
  - 相关函数：`mesh_fallback_count()`（`boolean/mod.rs` 顶部）。

修复时建议：先把 `approx_census` + `MESH_FALLBACK_COUNT` 跑出基线，收窄每个降级后再对比，确认"降级次数下降"而不是"换个方式崩"。

---

## Tier 1 — 丢失精确几何 / 解析曲面类型（优先修）

### 1. 布尔：GFA（精确 B-Rep）失败 → 网格共细化（mesh co-refinement）
- **位置**：`crates/operations/src/boolean/mod.rs`
  - 回退入口：`:865-889`（`if gfa_ok { return Ok } else { mesh_boolean_fallback(...) }`）
  - 回退实现：`mesh_boolean_fallback()` `:2405-2510`
  - 计数：`MESH_FALLBACK_COUNT` `:41`、`:2418`
  - 触发：`Err(e) =>` `:824`（GFA 抛错）或 `:817`（GFA 结果未通过验收门：Euler / 闭壳 / 流形）
- **降级行为**：把实体曲面化 → 共细化（triangle-triangle 交线）→ 重新组装。**所有解析曲面类型被销毁**（圆柱/圆锥/球/环变成三角面片，NURBS 变成更密面片）。`:2477-2494` 的注释承认网格回退会把两个物理分离孔用对角"桥"边粘成 8 字形 inner wire，可能非流形。
- **日志**：`:866` `brepkit_approx` "GFA unusable — using mesh … fallback; analytic surface types will be lost"；`:818` `log::warn!`。
- **修复方向**：
  1. 收窄 GFA 验收门里"误杀合法结果"的分支（尤其 `cut_safe` / `intersect_safe` 的多组件判定，`:734-766`）。
  2. 让多组件 / 多区域输入走更优的精确路由（见 §2），而不是默认掉网格。
  3. 网格结果 `boundary_edge_count > 0 || non_manifold_edge_count > 0` 时（`:2434`）目前只 `warn`，应改成"能报错就报错"或至少带回退计数，避免静默放行破壳。

### 2. 布尔：多区域 / 多组件特殊路由（绕过 GFA 限制的 workaround）
- **位置**：`boolean/mod.rs:838-863`；实现 `cut_multi_region_input` `:2833`、`fuse_multi_component_tool` `:2811`
- **降级行为**：GFA 的 pavefiller 不能整体处理多连通输入。Cut 按组件逐个切再合并；Fuse 对多组件 tool 逐个 fold。**每个子布尔仍可能各自触发 §1 的网格回退**——会放大 Tier-1 概率。
- **日志**：静默路由（无日志）。
- **修复方向**：把"分组件"当作精确计算的必要前置步骤保留，但确保子布尔优先用 GFA；只有当子布尔自身 GFA 也失败时才可以网格回退，且要计入 `MESH_FALLBACK_COUNT`。

### 3. 布尔：无解析分类器时的 AABB 严格包含回退
- **位置**：`boolean/mod.rs:2176-2205`
- **降级行为**：没有解析分类器时，用"AABB 严格大 10%（三轴）"判定嵌套包含，作为 `solid_strictly_inside` 的 no-classifier 回退；可能误判包含/不包，导致 Cut 走 `build_contained_cut_hollow` 捷径（`:158`）产生错误结果而非报错。
- **修复方向**：优先构造 `try_build_analytic_classifier`（§13）；AABB 仅作为最后兜底并打 `brepkit_approx`。

### 4. 布尔：`boolean_with_evolution` 的 EvolutionMap 退化为几何启发式
- **位置**：`boolean/mod.rs:1151-1167`
- **降级行为**：GFA 不可用（相同操作数 / GFA 错误 / 结果未验证）时，`EvolutionMap` 改为对标准布尔结果做几何启发式（按签名匹配输入/输出面），**不忠实（approximate, not faithful）**。
- **日志**：`:1154` `brepkit_approx` "faithful GFA provenance unavailable, using heuristic"。
- **修复方向**：仅影响可追溯性（拓扑溯源），不直接改几何；优先级低，但应在 GFA 恢复后自动回到 faithful 路径。

### 5. 圆角级联回退：滚动球 → 步行 NURBS → 平面斜切（倒角）★ 用户示例②
- **位置（wasm 路径调度）**：`crates/wasm/src/helpers.rs:161-208` `try_fillet`
  - ① `fillet_rolling_ball`（精确滚动球，解析接触）→ ② `blend_ops::fillet_v2`（= `FilletBuilder` 步行引擎，见 §6）→ ③ `crate::fillet::fillet`（**已弃用的平面斜切 / 倒角，chamfer-like 近似**）。三者都做闭壳校验，全失败则原样返回实体。
- **位置（Rust 路径）**：`crates/operations/src/fillet/mod.rs:77` `fillet`（独立弃用 API，**本身不是运行时回退**，但它被 `try_fillet` 当作最后兜底，所以在 wasm 路径上构成运行时降级）。
- **降级行为**：最后一级平面斜切把圆角彻底变成倒角（丢失 G1 连续与圆角几何）。
- **日志**：级联里 ② `fillet_builder.rs:740` 打 `brepkit_approx`；但 ③ 的平面斜切**目前没有显式近似日志**（仅 §2 注释 `:152-153` 写 "a flat bevel are fallbacks"）。
- **修复方向**：
  1. 至少让 ③ 的平面斜切在生效时打 `brepkit_approx` 告警（目前是静默几何降级，最危险）。
  2. 评估能否把 ③ 从默认级联里移除（改为显式选项），避免"圆角静默变倒角"。
  3. 优先让 ② `FilletBuilder` 在更多曲面对上走解析（见 §6）。

### 6. 圆角步行引擎内部：解析快路径 → 牛顿-拉夫森步行（近似 NURBS）
- **位置**：`crates/blend/src/fillet_builder.rs:725-744`（`try_analytic_fillet` 返回 `Some` 才用解析；否则落 walker）
- **解析函数**：`crates/blend/src/analytic.rs` `try_analytic_fillet`（`None` = walker 回退，覆盖 Cyl×Cone、垂直/斜轴 Cyl×Cyl、非共轴 Cone×Cone、斜轴 Sphere、退化 spine 等大量情形）
- **降级行为**：walker 用 `approximate_blend_surface` 把截面序列拟合成 NURBS，**丢失解析圆柱/环面等闭式类型**。
- **日志**：`:740` `brepkit_approx` "using Newton-Raphson walker (approximate NURBS blend surface)"。
- **修复方向**：扩展 `try_analytic_fillet` 闭式覆盖（加新的 `FaceSurface` 变体时编译器会强制在 `analytic.rs:187` 枚举处补分支）；walker 作为兜底保留但应可禁用。

### 7. 倒角解析路径（目前无步行回退，直接报错）
- **位置**：`crates/blend/src/chamfer_builder.rs:361-381`
- **降级行为**：`try_analytic_chamfer` 返回 `None` 时**不回退**，返回 `UnsupportedSurface`（v1 无步行回退）。
- **日志**：`:367` `brepkit_approx` "v1 has no walker fallback, returning UnsupportedSurface"。
- **注意**：这与 fillet 不同——chamfer 在这里是**硬失败**而非降级；若上层 `try_fillet` 调用则会落到 §5 的平面斜切。修复 fillet 级联时要把这个交互考虑进去。

### 8. Offset：NURBS 面偏移 → 采样重拟合（非精确解析偏移）
- **位置**：`crates/offset/src/offset.rs:135-162`
- **降级行为**：NURBS 面按 16×16 网格采样偏移点后 `interpolate_surface` 重拟合为 NURBS（环面/圆柱等本可精确偏移）。
- **日志**：`:136-138` `brepkit_approx` "offset … via 16x16 sampled-NURBS refit … not an exact analytic offset"。
- **修复方向**：对可解析偏移的曲面类型走解析；采样重拟合仅作为 NURBS 兜底。

### 9. Offset：自交裁剪失败 → 原始偏移曲面（可能含自交）
- **位置**：`crates/operations/src/offset_face.rs:247-269`（`trim_offset_self_intersections` 失败回退到未裁剪原始偏移曲面）
- **日志**：`:259-266` `brepkit_approx` + `log::warn!`。
- **修复方向**：裁剪失败时优先"缩小有效区重拟合"而非整面退回；高曲率区自交不被消除会污染下游布尔（§1）。

### 10. Offset：SSI 检测失败 → 网格采样裁剪（粗近似）
- **位置**：`crates/operations/src/offset_trim.rs:13-14, 61, 208-235`（`trim_via_sampling`）；`invalid_fraction > MAX_INVALID_FRACTION` 时报错。
- **降级行为**：解析 SSI 检测找不到曲线时回退网格采样自交检测与裁剪。
- **修复方向**：优先保留解析 SSI；采样分辨率提高 + 边界平滑。

### 11. Revolve：解析整圈旋转 → 分段旋转（面片化）
- **位置**：`crates/operations/src/revolve.rs:499-502`（`try_analytic_full_revolution` 返回 `Ok(None)` 则回退分段 revolve）
- **降级行为**：整圈且剖面平面/轴在面内/无内孔/全解析边时才能精确生成圆柱/锥/环/球；否则落通用分段旋转，解析类型变面片。
- **修复方向**：扩大解析整圈条件覆盖（非整圈 / 含样条 / 轴接触弧的情形）；分段作为兜底。

### 12. Loft：匹配曲线剖面 → 多边形路径（面片化）
- **位置**：`crates/operations/src/loft.rs:543-564`（`try_loft_matching_curved_profiles` 返回 `None` 回退）；`:743-748`（反向 ruled arc 构建失败回退 reversal flag）
- **降级行为**：当所有剖面同为平面/无孔/同边数/同曲线类型且至少一边为曲线时才能保留圆弧（解析 Cylinder/Cone/ruled-NURBS）；否则落多边形路径把弧边面片化。注释 `:560` 明确这点会触发下游布尔网格回退。
- **修复方向**：保解析弧边优先；面片化仅兜底。

### 13. 分类器：解析分类器 → 射线投射 / 缠绕数
- **位置**：`crates/algo/src/classifier/mod.rs:37-47`、`61-75`；`crates/operations/src/classify.rs:151-172`（`classify_point_robust` 在缠绕数 0.4–0.6 歧义带回退射线）
- **降级行为**：凸解析实体有 O(1) 解析点分类；否则回退多射线投射（采样边界面，NURBS 才曲面化）。`classify_point_robust` 在歧义带回退射线投射。
- **影响**：射线投射在薄壁 / 退化 / 掠射处可能误判内/外（见 `classify.rs:78-104` 的 grazing 问题），进而导致布尔分类错误（§1、§3）。
- **修复方向**：加强 `classify_coincident_coplanar` 类修补覆盖；解析分类器优先构造（§3 依赖它）。

---

## Tier 2 — 数值近似 / 可能误判（一般不丢曲面类型）

### 14. NURBS 曲面求交的多个数值回退
- **位置**：`crates/math/src/nurbs/intersection/surface_marching.rs:321-327, 435, 966`；`surface_seeding.rs:487, 658, 806, 830-831`
- **降级行为**：SSI 追踪中，解析二阶几何分析不可用时退回采样 / 扰动 / 投影，保不崩溃但损精度（可能漏/多交线）。
- **修复方向**：提高二阶切向分析鲁棒性；扰动搜索步长自适应。

### 15. 解析求交的采样 / 中点回退
- **位置**：`crates/math/src/analytic_intersection.rs:25, 49, 75, 2182`
- **降级行为**：环面相交（四次）/ 退化 / 相切时，闭式写不出则回退采样点链；相切取中点。
- **修复方向**：环面相交加闭式分支；相切中点回退打 `brepkit_approx`。

### 16. 曲线点投影：牛顿未收敛 → 最佳迭代
- **位置**：`crates/math/src/nurbs/projection.rs:149-150`（`curve_newton_refine` "Always returns a result — falls back to the best iterate"）
- **降级行为**：牛顿-拉夫森投影未达收敛则回退最佳迭代参数。
- **修复方向**：下游距离 / 分类受影响，需评估误用场景。

### 17. 极值 / NURBS 法向 / 边容差的 best-effort 回退
- **位置**：
  - `crates/geometry/src/extrema/mod.rs:12,14`（curve_curve / point_surface 解析失败 → 采样+Newton）
  - `crates/math/src/traits.rs:24`、`crates/topology/src/face.rs:70-71`（NURBS 法向退化点回退 `Vec3::Z`）；`crates/heal/src/fix/face.rs:230`（面法向归一化失败回退 Z）
  - `crates/topology/src/edge.rs:262-266`（`effective_tolerance`：边无容差时用顶点容差回退 = 放宽容差）
- **降级行为**：法向 / 容差在退化情形用固定向量或放大容差。
- **修复方向**：退化点坐标记录 + 下游裁剪/合并判定加权。

### 18. Tessellation：平面 CDT 失败 → 扇形三角化
- **位置**：`crates/operations/src/tessellate/planar.rs:885-893`（`fan_triangulate`）
- **降级行为**：CDT 因 Steiner 顶点丢三角形/裂缝时回退扇形三角化（仅用原边界顶点，构造流形但更粗）。
- **修复方向**：CDT 容差/种子点改进；扇形仅兜底。

### 19. Tessellation：顶点法向回退（面法向累加 → Z）
- **位置**：`crates/operations/src/tessellate/solid.rs:695-750`（`:720, :747` 回退 `Vec3::Z`）
- **降级行为**：无法从相邻面法向求顶点法向时，回退相邻三角形法向累加；再失败回退 Z（仅渲染法向，不影响几何拓扑）。
- **修复方向**：低优先（纯渲染）。

### 20. STEP 读取：权重列表部分解析 → 均匀权重
- **位置**：`crates/io/src/step/reader.rs:733-738`
- **降级行为**：有理 B 样条权重少于期望数量时，回退均匀权重 `vec![1.0; expected_count]`，不再传播维度不匹配错误（权重错误会使导入曲面形状失真，但几何仍为 NURBS）。
- **修复方向**：权重缺失应报错或至少打 `brepkit_approx`，而非静默均匀化。

### 21. Heal：容差放宽 / 跳过 / 位置回退
- **位置**：
  - `crates/operations/src/heal.rs:866-872`（平面等价比较放宽到 1e-4 rad / 1e-3 mm，允许网格衍生共面合并）；`crates/heal/src/analysis/surface.rs:189`（同）
  - `crates/operations/src/heal.rs:1005-1011`（顶点匹配先 VertexId，失败回退位置量化匹配——安全假阴性）
  - `crates/operations/src/heal.rs:1363-1373`（unify_faces 跳过边界边过多的合并组，性能保护）
  - `crates/operations/src/heal.rs:953`（"Can't verify — skip merge to be safe"）
- **降级行为**：修复阶段放宽判定或跳过修复，保稳健而非最精确（可能漏合并、保留碎片面，但不引入错误几何）。
- **修复方向**：放宽阈值按来源区分（B-Rep 来源用紧阈值，网格回退来源才用松阈值，见 §1）。

---

## Tier 3 — 纯性能 / 安全网（无几何损失）

### 22. 渲染：GPU 设备初始化 → 软件（CPU）回退
- **位置**：`crates/render/src/pipeline.rs:54-119`（`candidate_adapters` 先 `force_fallback_adapter:false` 再 `true`；`acquire_device` 逐个尝试，真实适配器创建设备失败则回退软件适配器）；`crates/render/src/error.rs:6-9`（`NoAdapter`）
- **降级行为**：优先真实 GPU，失败后请求 wgpu 软件适配器（如 lavapipe）。仅渲染性能损失，被渲染的 B-Rep 完全不变。
- **修复方向**：无需修（合理的运行时容错）。可加日志区分走了哪条适配器。

### 23. 布尔后处理 best-effort（不中止）
- **位置**：`crates/operations/src/boolean/mod.rs:533-554`（合并/统一边界边失败仅 `log::debug!`）；`:888`（`enforce_manifold_shell(...).unwrap_or(result)`）
- **降级行为**：修复子步骤出错时保留当前（可能不完美）结果而非终止。
- **修复方向**：`:888` 的 `unwrap_or(result)` 会放行带自由边的壳——建议对 `enforce_manifold_shell` 失败也计入降级度量。

### 24. Sweep 多类回退（数值稳健性）
- **位置**：`crates/operations/src/sweep.rs:138`（前向量选择回退）、`:466`（`Ok(None)` 回退通用 sweep）、`:1033`（叉积消失回退）
- **降级行为**：通用扫掠中若干启发式回退（选不平行切线的世界轴、路径非预期时回退通用扫掠、自相交处回退）。几何类型基本保留。
- **修复方向**：低优先。

---

## 用户示例对照表

| 用户示例 | 结论 | 文档定位 |
|---|---|---|
| ① 布尔 B-Rep 失败 → mesh | **确认 + 扩展** | §1（主回退）、§2（多组件放大）、§4（溯源退化） |
| ② 圆角失败 → 倒角 | **确认存在（wasm 级联）** | §5 `try_fillet`（`wasm/src/helpers.rs:161-208`）；纯 `fillet::fillet` 是独立弃用 API、非运行时回退，但被 `try_fillet` 当最后兜底 |
| 解析圆角 → 步行 NURBS | **确认** | §6（`fillet_builder.rs:725-744` + `analytic.rs` `try_analytic_fillet`） |
| 渲染 GPU → 软件 | **确认** | §22（`render/src/pipeline.rs:54-119`） |
| 分类器解析 → 射线/缠绕 | **确认** | §13 |
| heal 容差放宽/跳过 | **确认** | §21 |
| tessellate 自适应 → 均匀/扇形 | **确认** | §18、§19 |
| 求交解析 → marching/数值 | **确认** | §14、§15 |
| sketch GCS 求解器不可解回退 | **核查结论：未发现"不可解→近似解"回退** | `crates/sketch/src/gcs/solver.rs` 是 DogLeg 信任域求解器，不收敛仅返回 `converged:false`，由调用方决定；`constraint.rs:119-160` 的 `unwrap_or((NaN,NaN))`/`dummy()` 是把缺失句柄转 NaN 触发不收敛的防御，**不是**给近似解。属"硬失败/不收敛"，未列入降级清单 |
| offset 面-面相交回退 | **确认** | §8–§10（`offset.rs` 采样重拟合、`offset_face.rs` 原始曲面回退、`offset_trim.rs` 网格采样裁剪）；`offset/src/inter3d.rs:156` "NURBS fallback not yet implemented" 是缺失功能直接报错，非降级 |
| io 解析失败放宽/近似 | **确认** | §20（STEP 权重回退均匀） |
| curve/project 中点猜测 | **确认** | §16（投影最佳迭代）、§17（NURBS 法向 Z）；`algo/.../fill_face_info.rs:150` NURBS 投影失败静默回退域中点猜测 |
| topology 有效容差 / 面法向 Z | **确认** | §17（`edge.rs:262`、`face.rs:70-71`） |
| wasm JS 侧回退 | **确认** | §5 `try_fillet` 级联（最关键 wasm 侧运行时降级）；`booleans.rs` 暴露 `meshFallbackCount` |

---

## 建议的逐步修复顺序（按几何损失排序）

1. **§1 布尔网格回退**（丢失全部解析几何、可能非流形）：`MESH_FALLBACK_COUNT` 已可量化，先收窄 §1 验收门 + §2 多组件路由，目标让常见布尔不再掉网格。
2. **§5 圆角级联（含 wasm `try_fillet` 落到平面斜切）**：给 ③ 平面斜切加 `brepkit_approx` 告警；评估是否把它从默认级联移除。这是"静默几何降级"最高风险点。
3. **§8–§12 Offset / Revolve / Loft 解析 → 面片**：这些解析体/偏移被面片化后会把下游布尔推向 §1，应优先保解析。
4. **§13 分类器射线回退 + §3 AABB 包含回退**：影响布尔分类正确性，加强 `classify_coincident_coplanar` 类修补覆盖。
5. **Tier 2 / Tier 3**：多为数值精度与性能安全网，按 `approx_census` 实测频次按需加固，不紧急。

> 每条修复后：跑 `approx_census` + 读 `MESH_FALLBACK_COUNT`，确认对应降级触发次数下降、且相关 golden 测试（`crates/**/tests/*inmem.rs`、`*_fallback*.rs`）仍全绿。
