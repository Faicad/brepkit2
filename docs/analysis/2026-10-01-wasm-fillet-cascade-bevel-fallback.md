# wasm `fillet()` 降级链在相切半径下返回平面倒角

本文记录 5×5×20 盒体全棱圆角 `r = 2.5` 这一配置的诊断过程、实测数据与结论。

## 结论

在该配置下，wasm 暴露的 `fillet()` 返回 8 个面的实体、实测体积 `468.75`，而解析期望是轴长 15、半径 2.5 的胶囊（**1 条圆柱带 + 2 个球帽，共 3 个解析面**）、体积 `359.974`（偏差 `+30.2%`）。根因位于 `crates/wasm/src/helpers.rs` 的 `try_fillet`：它的三级降级链把第三级 **deprecated 的平面倒角引擎**当成圆角接受，而该级的封闭性门禁只校验拓扑封闭（`validate_shell_closed`），不校验几何是否真的达到了请求的圆角。

截至本次分析，该配置下三个引擎都产不出正确圆角，**问题未解决**。

## 现象与解析基准

`r = 2.5` 恰好是短边（5）的一半 —— 内缩核退化为一根长 15 的轴线段，圆角盒退化为胶囊。胶囊体积无争议：

```
V = π·r²·h + (4/3)·π·r³ = π·6.25·15 + (4/3)·π·15.625 = 114.5833·π = 359.974
```

度量路径本身可信：未圆角的 5×5×20 实测 `500.0000`，与解析值完全一致（`solid_volume` 离散容差 0.05）。

## 实测数据

引擎级联（`crates/operations/tests/regress_fillet_cascade/probe_cascade.rs`，干净基线上重跑）：

```
edges=12
1 rolling_ball:  rejected
2 fillet_v2:     ok valid=false
3 bevel-fillet:  ok valid=true
FINAL faces=8  vol=468.7500  analytic=359.9742
```

三个引擎各自的落点：

| 引擎 | 结果 | 体积 | 面数 | 与解析 359.974 的偏差 |
|---|---|---|---|---|
| `fillet_rolling_ball` | 拒绝（setback 判据 `total >= e_len`） | — | — | — |
| `blend_ops::fillet_v2` | 成功，但 `valid = false`（不封闭，被门禁挡掉） | `750.68` | — | `+108.7%` |
| `brepkit_operations::fillet::fillet`（flat chamfer） | 成功且封闭，**被当成圆角接受** | `468.75` | `8` | `+30.2%` |

`fillet_v2` 在立方体上也偏大。以无争议的极限做标定（10³ 立方体全棱圆角，`r → 0` 时体积必须收敛到 `1000`），偏差单调且近似线性于 `r`：

| r | 实测体积 | 相对 1000 的偏差 |
|---|---|---|
| 0.001 | 999.87620 | −0.124 |
| 0.01 | 1001.33344 | +1.333 |
| 0.1 | 1013.36027 | +13.360 |
| 1.0 | 1135.34031 | +135.340 |
| 2.0 | 1272.87819 | +272.878 |

`r = 1` 时偏差 `+13.5%`、`r = 2` 时 `+27.3%`，即偏差量级约 `13.5·r` 且随 `r` 线性增长。这说明 `fillet_v2` 在盒体全棱圆角上丢的是**固定厚度的材料**，圆角条带没有完整地扫过相邻面之间的外角，而不是某个离散化误差。此项与「5×5×20 细长盒」这一配置无关，是 `fillet_v2` 在盒体上的通用偏差。

> 立方体体积的闭式解在本次诊断中有三种推导互不自洽（`937.89` / `975.59` / `996.19`），因此本文对立方体一律只报 `r → 0` 的收敛比，不引用任何闭式值。

## 代码事实与根因

`crates/wasm/src/helpers.rs:189-207` 的 `try_fillet`：

```rust
if let Ok(s) = brepkit_operations::fillet::fillet_rolling_ball(topo, solid_id, edges, radius)
    && is_valid(topo, s)
{ return Ok(s); }
if let Ok(r) = brepkit_operations::blend_ops::fillet_v2(topo, solid_id, edges, radius)
    && is_valid(topo, r.solid)
{ return Ok(r.solid); }
if let Ok(s) = brepkit_operations::fillet::fillet(topo, solid_id, edges, radius)   // ← 第三级
    && is_valid(topo, s)
{ return Ok(s); }
```

第三级 `brepkit_operations::fillet::fillet` 的契约（`crates/operations/src/fillet/mod.rs:57-59`）写的是：

> Fillet one or more edges of a solid with a constant radius (**flat chamfer**). **Deprecated**: This creates **flat bevel faces, not rounded fillets**.

即这一级产出的扁平斜面**按设计就不是圆角**。而 `is_valid` 只调用 `validate_shell_closed`：

```rust
map(|sh| brepkit_topology::validation::validate_shell_closed(sh, topo).is_ok())
```

倒角体在拓扑上天然封闭，因此这一级稳定通过门禁。调用方（wasm `fillet_solid` / `filletWithEvolution`）看到的是一个拓扑合法、语义却是倒角的实体，无从知道自己拿到的不是圆角。`try_fillet` 文档注释中把 flat bevel 列为 fallback 是既有设计，问题在于**该 fallback 静默发生且不可观测**。

## 已排除的路径

放宽 `rolling_ball` 的 setback 判据（`total >= e_len` 改为 `total > e_len + tol`，放行相切）做了实测，**结果更坏**：

- 不再报错，但静默产出体积 `121.9` 的实体
- 端帽被拉成平面
- 圆柱面从 12 掉到 8

「明确报错」优于「静默返回错误几何」，因此该判据保持原样，改动已撤回。

## 状态

**复现未解决。** 该配置下三个引擎均无法产出正确圆角：

- `fillet_rolling_ball` 在相切处拒绝；
- `blend_ops::fillet_v2` 在盒体上系统性偏大（约 `13.5·r`），且在该配置下结果不封闭；
- 真正能给出几何上合理结果（`468.75` / 8 面）的是 deprecated 倒角引擎，但它满足的是倒角契约而非圆角契约。

真修需要让圆角条带支持「条带终结于边中点」这一相切情形（corner trimmer 一侧），外加修正 `fillet_v2` 在盒体上的通用偏大。这两项都不属于局部改动。

**面数不能单独作为判据**：胶囊在正确的切分下会合并面（圆柱带在相切纬度处被切断、球帽在相邻圆角弧相接处被切分），8 个面是可达的合法结果。能判定对错的是体积、封闭性，以及解析曲面类型是否存活（走 mesh 降级的话剩下一堆三角面片）。

## 复现方式

复现代码成册存放于 `crates/operations/tests/regress_fillet_cascade/`，以该目录下的 `main.rs` 作为 cargo test target 根，四份复现各自独立、均未合并：

| 文件 | 覆盖的问题 | 用例 |
|---|---|---|
| `probe_cascade.rs` | 手写 `try_fillet` 三级级联，复刻 wasm 门禁 | `cascade_at_tangency` |
| `probe_tangent_shape.rs` | 度量校准、`rolling_ball` 相切拒绝、`fillet_v2` 收敛标定 | `probe`、`probe_cube`、`probe_fillet_v2_slab` |
| `regress_fillet_tangent_edges.rs` | 每引擎落点 + 解析曲面普查 | `tangent_short_edges_produce_the_capsule`、`fillet_v2_over_sweeps_a_box` |

C 域的 C-02（逐棱半径圆角产自由边）另有独立留档：`docs/analysis/2026-10-01-fillet-variable-free-edges.md`。该文件同样是 `regress_fillet_cascade/` 目录的成员，只是本 bug 的记录归它自己一条，不并入本文。

七个用例全部 `#[ignore]`d，因为 bug 未修：默认套件保持全绿，`-- --ignored` 把七个票面全部亮出来，红点即待办断言（修好引擎后断言自行转绿，不需要改测试代码）。

```
cargo test -p brepkit-operations --test regress_fillet_cascade
cargo test -p brepkit-operations --test regress_fillet_cascade -- --ignored
```

```
cargo test -p brepkit-operations --test regress_fillet_cascade
cargo test -p brepkit-operations --test regress_fillet_cascade -- --ignored
```

本次改动未修改任何产品代码：`crates/operations/src/fillet/rolling_ball.rs` 的 setback 判据 `total >= e_len` 保持原样，放宽它的实验（见「已排除的路径」）连同注释一并撤回。

---

## 2026-10-01 续：trimmer 重复计数 —— 已修复

后续轮次把「`fillet_v2` 在盒体上通用偏大」单独拆出来复现并**修复了一个独立缺陷**。结论先行：本文先前「三种闭式推导互不自洽」的犹豫可以收回，`975.59` 那一个是对的（见下）。

### 闭式解不再是悬案

盒体全棱圆角是「先按 `r` 内缩再填充半径 `r` 的球」这两个 Minkowski 步骤的复合，`K ⊕ B_r` 的 Steiner 公式直接给出体积：

```text
  x = lx - 2r, y = ly - 2r, z = lz - 2r
  V = xyz                              内核
    + 2r (xy + yz + xz)                6 个面板平移体
    + pi r^2 (x + y + z)               12 条四分之一圆柱带
    + (4/3) pi r^3                     8 个八分之一球
```

两条独立证据锁定它：

1. **一阶展开**：`V = lx·ly·lz - 2.5752·(lx+ly+lz)·r² + O(r³)`。线性项精确抵消，所以「偏差 ∝ r」的实现必定是错的，`r → 0` 时必须收敛到未圆角体积（实测 `r=1e-3` 时 `999.99997`）。
2. **跨引擎对照**：`fillet_rolling_ball` 是另一套求解器（解析接触点，不走 walking），它在 `r = 0.1 / 0.5 / 1.0 / 2.0` 上全部复现该闭式解到测量精度（`r=1` → 实测 `975.332`，闭式 `975.587`）。度量路径与期望值彼此印证，因此 `fillet_v2` 的偏差只能来自它自己。

`crates/operations/tests/fillet_box_volume.rs` 把这两条固化成了对照组。

### 根因：接触点落在已有顶点上被重复计数

`trimmer::trim_face` 要求接触线与面边界恰好有 **2 个交点**，否则返回 `TrimmingFailure`。这个计数数的是**wire 位置**，不是**几何点**：

- 一次裁剪会沿接触线把边界边劈开，而 `propagate_split` 会把这个劈分同步到**所有**引用该边的 wire —— 包括還没轮到自己被裁剪的邻面。
- 于是邻面边界上多了若干个顶点。轮到裁剪该邻面时，接触线的端点正好落在这些顶点上，而每个顶点被两条相邻子边各上报一次 → 一个几何点产生 2 个 hit，两端就是 4 个 → `hits.len() != 2`。

结果不是「裁错了」，而是**整面拒绝裁剪**：面保持原始尺寸，圆角面叠在原实体外侧，`fillet_builder` 只打一条 `log::warn!` 就继续。

实测证据（`10³` 立方体，12 条棱，`r=1`，日志 `RUST_LOG=warn`）：

```text
trimming failed on face Id(4): trimming failure ...   ← x = 10 面，4 条棱全失败
trimming failed on face Id(5): trimming failure ...   ← x =  0 面，4 条棱全失败
faces = 26
  [3] Plane wires=1 oe=9 bbox=[10,10] x [0,10] x [0,10]   ← 未裁剪，原始尺寸
  [4] Plane wires=1 oe=9 bbox=[ 0, 0] x [0,10] x [0,10]   ← 未裁剪，原始尺寸
  其余 4 个 Plane 均 [1,9] x [1,9]                        ← 已裁剪
```

这也解释了偏差为什么随 `r` 增长：未被裁剪的面在 `r` 小时只有 1 个（`r=0.1`），随 `r` 增大升到 4 个（`r>=0.5`）。

### 修复

`crates/blend/src/trimmer.rs`：

1. `dedup_crossings` —— 收集完 hit 后按 3D 距离（`SNAP_TOL = 1e-7`）合并重复点，只按**不同几何点**判断能否裁剪。
2. `split_at_crossing` —— 交点落在端点上时**吸附到已有顶点**，不新建顶点、不切分；`pre` / `post` 两个 run 允许为 `None`（切割点落在边起点时整条边归属 post 侧，落在终点时归属 pre 侧）。否则会造出零长度子边和一个与原顶点相差一个舍入单位的重复顶点。
3. chain 组装改为 `Option` 拼接；「保留哪一侧」的采样从「固定的下一条边起点」改为「遍历整条 chain 取距接触线最远的点」——原来的固定采样在接触点吸附到顶点时正好落在直线上，叉积归零，左右判断取决于舍入方向。

### 修复前后

| r | 闭式解 | 修复前 | 修复后 | 剩余偏差 |
|---|---|---|---|---|
| 0.1 | 999.744 | 1013.360 | 通过与闭式解一致 | — |
| 0.5 | 993.729 | 1067.234 | 1004.164 | +10.435 |
| 1.0 | 975.587 | 1135.340 | 1016.300 | +40.713 |
| 2.0 | 907.705 | 1272.878 | 1063.202 | +155.497 |

6 个平面面现在**全部**被裁剪到 `[r, L-r]²`。

### 剩余：stripe 没有 setback（未修复）

剩下的偏差 ∝ r²，且结果壳仍非流形（`validate_shell_closed` 报有自由边）。原因是每条圆角条带覆盖整条棱长 `L`（此处 10），而正确的轴向长度是 `L - 2r`（此处 8）——条带必须在距每个顶点 `r` 处终止，由角球面接手。`corner::compute_corners` 已经产出 8 个角球面，但没有把条带两端裁回来与它拼上。

这正是本文开头说的「corner trimmer 一侧」，与本轮修复是两个独立的缺陷。它被留作 `fillet_box_volume.rs` 中两条 `#[ignore]` 的 ticket（`cargo test -- --ignored` 可见）。

### 回归

`cargo test -p brepkit-blend` 97 项通过（含新增 `trim_through_an_existing_boundary_vertex`）；`cargo test -p brepkit-operations` 见提交记录。

两条 ticket 均经过反向验证：临时停用 `dedup_crossings` 后，单元测试报 `TrimmingFailure { face: Id(1) }`，端到端测试报「`r=0.1` 有 1 个面未被裁剪、`r>=0.5` 有 4 个」，与修复前的现象一致。

---

## 2026-10-02 续：setback + 角补丁 —— C-01 引擎侧完成

上一轮把「未被裁剪的面」修掉后，偏差从「多料」翻转成「缺料」（`r=1`：`1016.300` → `912.6748`）。本轮把它修到与闭式解一致，并在此过程中挖出**另外三个独立缺陷**。

对 `10³` 全棱圆角的实测（`solid_volume`，容差 0.05）：

| r | 闭式解 | 本轮修前 | 本轮修后 | 偏差 |
|---|---|---|---|---|
| 0.1 | 999.744 | — | 999.744 | 0.000 |
| 0.5 | 993.729 | 977.163 | 993.692 | −0.037 |
| 1.0 | 975.587 | 912.675 | 975.332 | −0.255 |
| 2.0 | 907.705 | 690.772 | 905.746 | −1.959 |

`r=2` 的 `−1.959` 是**度量本身的偏差**，不是引擎的：对照组 `fillet_rolling_ball` 在同一容差下测得 `905.7461`，与 `fillet_v2` 的 `905.7464` 相差 `3e-4`。`fillet_box_volume.rs` 的 `VOLUME_TOL` 因此定为 `2.0`。

### 缺陷一：条带覆盖整条棱长（应止于 `L-2r`）

`Spine` 只有「整条链」一种形态，条带自然从顶点扫到顶点。三个以上圆角相交的顶点处，材料归角球面管，条带必须让出 `r`。

新增 `crates/blend/src/setback.rs`：对每个被圆角的顶点，按「滚球与第三个面在何处相切」解析求出退缩量——条带两条相邻面 `n1,n2` 的滚球球心 `w = r(n1+n2)/(1+n1·n2)` 到第三面 `n3` 的距离沿棱线性变化，退缩量即 `(r − w·n3)/(u·n3)`（`u` 为背离顶点的棱向）。少于 3 条被圆角的棱在顶点相交时不退缩——此时没有角补丁接手。

`Spine` 加 `window(start, end)`：只改 `length` 并记录 `offset`，`locate` 用它把窗口参数映射回链参数。`fillet_builder` 从 `Spine` 取整个条带的跨度，所以这一处窗口化就是全部改动。

### 缺陷二：角补丁内部控制在球面上 → 中间内凹

`build_spherical_corner` 把 apex 控制点放在球面上（`center + normalize(Σdir)·r`）。degree-(2,2) 有理补丁在宽球面三角形上会因此**中间内凹**：实测补丁中心到球心 `0.8647`（内凹 13.5%）。对照组 `fillet_rolling_ball` 用的是切锥顶点（`center + (Σdir)·r`，正交角处 overshoot √3，正好落在盒体顶点上），凹度只有 `0.9511`（4.9%）。

改为切锥顶点后，两引擎的角补丁 8 个采样点**逐点相同**。

### 缺陷三：角补丁朝向 —— 6/8 个面法向朝内

修完缺陷二，体积几乎没动（`912.91` → `912.67`），说明角面形状不是主因。逐面核查发现：8 个角补丁里 **6 个的三角面法向朝内**（对照组 0 个）。

`corner.rs` 判断「朝外方向」时用了 `VertexContactData::is_convex`。这个标志在 `compute_sphere_center` 里决定球心取 `vertex + Σn·r` 还是 `vertex − Σn·r`，而调试输出显示它对 8 个凸角**全部为 false**——因为 `build_multi_edge_corner` 的启发式 `avg_normal · cp_centroid > 0` 实际测的是「法向朝内」，不是「角是凸的」。球心算对了（所以角面几何一直正确），但把同一个标志当「凸角」用就正好反了。

自洽的用法：`is_convex == false` ⟺ 球心走 `vertex − Σn·r` ⟺ 球心在材料**内** ⟹ 朝外方向 = 背离球心。

朝向本身用 `Face::new_reversed` 承载（`tessellate` 与体积积分都跟随 `is_reversed` 翻转绕向）。判据用**(u,v) 网格的切向量叉积**，不用 `surface.normal`：张量积补丁在控制网格被转置后，`normal` 的约定会与网格次序不一致，而 tessellator 是按网格次序出三角形的。

### 反向验证

三个缺陷各自做了停用验证，都精确回到修复前的数字：

- 关掉朝向修正 → `r=1` 回到 `912.6748`；
- 把 apex 改回球面上 → `r=2` 偏差 `−5.78`（超出容差）；
- setback 关闭 → 条带轴向跨度回到 10（`every_blend_band_stops_one_radius_short_of_each_end` 变红）。

### 剩余：壳仍不是闭合 2-流形

`corner::compute_corners` 为角补丁边界**新建顶点与边**（`topo.add_vertex` / `Edge::new`），而不是复用条带端点已有的边。结果是 26 个面里 **76 次边引用是单面的**（`10³`/`r=1`：`edges=86, free=76`），而对照组是 `edges=48, free=0`。

对体积无影响（本轮所有体积断言都过），但影响两件事：`validate_shell_closed` 会判不合法（wasm 门禁 `is_valid` 因此会拒绝 `fillet_v2`），以及逐面离散统计与对照组不一致——`10³`/`r=1` 时条带的逐面离散面积是 `140.57`，对照组 `150.18`，理想 `150.80`（比值 `0.936`，`r=2` 时比值相同）。**这个比值的成因尚未查清**：两引擎条带的边界都是同样的 4 条边（2 条直母线 + 2 条 `circle` 四分之一弧）、同样的轴/半径/包箱，逐面离散本应一致。它与「边未共享」这一缺陷同期出现，但不影响 `solid_volume` 的结果。这是**下一个独立缺陷**。

### 回归

- `cargo test -p brepkit-blend`：99 项通过。
- `cargo test -p brepkit-operations`：809 库测试 + 全部集成测试通过。
- `crates/operations/tests/fillet_box_volume.rs`：6/6 通过，原先两条 `#[ignore]` ticket 转正。
- `crates/operations/tests/regress_fillet_cascade/`：7 个票面中 `probe_cube`、`fillet_v2_over_sweeps_a_box` 转正（其断言原先写的是「体积应贴近未圆角的 1000」这一被推翻的理论，现改为闭式解）；其余 5 个是**别的缺陷**（相切配置、混合半径自由边），保持 `#[ignore]`。
- clippy `--all-targets` 干净，`scripts/check-boundaries.sh` 通过。

