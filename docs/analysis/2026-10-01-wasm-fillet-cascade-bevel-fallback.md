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
| `regress_fillet_mixed_radius.rs` | C-02：混合半径 / 逐棱半径圆角是否还水密 | `mixed_radius_fillets_stay_watertight` |

七个用例全部 `#[ignore]`d，因为 bug 未修：默认套件保持全绿，`-- --ignored` 把七个票面全部亮出来，红点即待办断言（修好引擎后断言自行转绿，不需要改测试代码）。

```
cargo test -p brepkit-operations --test regress_fillet_cascade
cargo test -p brepkit-operations --test regress_fillet_cascade -- --ignored
```

本次改动未修改任何产品代码：`crates/operations/src/fillet/rolling_ball.rs` 的 setback 判据 `total >= e_len` 保持原样，放宽它的实验（见「已排除的路径」）连同注释一并撤回。
