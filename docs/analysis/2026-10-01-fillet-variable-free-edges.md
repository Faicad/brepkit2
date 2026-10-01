# 逐棱半径圆角 `fillet_variable` 产出自由边（C-02）

本文记录 C-02 的诊断过程、实测数据与结论。C-02 原本登记在 bug plan 里，阻塞项写的是「缺逐棱半径 API」；本次实测推翻了那条登记线索，真实缺陷位于该路径的收尾装配。

## 结论

`brepkit_operations::fillet::fillet_variable`（配合 `FilletRadiusLaw`）**连均匀半径的对照用例都产不出水密实体** —— 自由边出现在对照里，不在混合半径的主用例里。这说明缺陷不在「逐棱半径的角点裁剪」（那条假设需要一个 corner trimmer 层面的新能力），而在 `fillet_variable` 与单半径路径**共用**的收尾装配段。

**复现未解决，未改动任何产品代码。** 真修需要先弄清 `fillet_variable` 的收尾装配与 `fillet_v2` 已验证路径之间的结构差异，属于非局部改动。

## 做了什么

在 `crates/operations/tests/regress_fillet_cascade/regress_fillet_mixed_radius.rs` 里把复现钉成一张票，两个用例方向：

- **对照**：10×10×10 盒体 12 条棱全部给 `FilletRadiusLaw::Constant(1.0)`，用 `validate_shell_closed` 验水密、用体积断言锚定解析值（该配置解析体积 `975.59`）。均匀半径在语义上与已可用的单半径路径等价，这一步本该必过。
- **主用例**：同一盒体，三条轴向各给一个不同半径（`0.5` / `1.0` / `1.5`），断言水密且总体积不增（圆角只能从实体上去料）。

用例带 `#[ignore]`，因为 bug 未修：默认套件保持全绿，`cargo test --test regress_fillet_cascade -- --ignored` 把票面亮出来，红点即待办断言（修好后断言自行转绿，不需要改测试代码）。

## 发现了什么

### 阻塞项的登记线索是错的

plan 把 C-02 记为「缺逐棱半径 API」，按这条线索去做应该先补 API 入口。实测：`fillet_variable` 与 `FilletRadiusLaw::Constant` 都在，输入是一个 `Vec<(EdgeId, FilletRadiusLaw)>`，测试能正常构造并提交给引擎。入口完备，可调用，缺的是**调用之后的结果不合法**。

### 均匀半径对照就已经失败

按「逐棱半径才需要在角点做互相裁剪」的常识，只有主用例该挂。实测顺序相反：均匀半径那一侧先挂出自由边（外壳校验报某条 edge 只挂了 1 条 wire，即只有单边 wire、缺对侧），混合半径挂在后一次校验。

这个方向很关键 —— 它把问题从「角点裁剪」挪到了**共用收尾装配**。角点裁剪只在半径不同时才需要，而共用装配段两种情形都要走。所以修好角点裁剪不等于修好 C-02，修好共用装配才是。

### 与 C-01 的关系

两者共用一条圆角装配链：C-01 卡在「相切处圆角条带如何终结」，C-02 卡在「收尾装配是否把条带接上」。C-01 的复现走 `fillet_rolling_ball` / `fillet_v2` / 倒角降级三级，`fillet_variable` 不在该链上，因此两处的红点互不覆盖。

### 已定位的收尾路径

- `crates/operations/src/fillet/mod.rs` 中 `fillet_variable` 的调度与收尾；
- `crates/operations/src/boolean/assembly.rs` 的 `assemble_solid_mixed`，即混合/逐棱半径的装配实现。

两条都已读到，但**只做定位、不改代码**，原因见下。

## 为何没能修复

按「只处理 C-01」的范围约束，本轮的目标是把 C-02 独立复现并留档，不动它的产品代码。此外即使放开范围，这一处也不适合在此收敛：

- 均匀半径即失败，说明缺陷面比登记的「逐棱」宽，需要先给 `fillet_variable` 补一条与单半径路径的对照基线，把「共用装配」和「角点裁剪」两段拆开分别验证；
- 没有基线就无法区分「哪里掉的条带」和「掉的是不是该掉的那条」，直接改装配段风险是静默产出看似合法、实际错面的实体。

因此本轮的交付是复现 + 票面 + 定位，代码保持原样。

## 复现方式

```
cargo test -p brepkit-operations --test regress_fillet_cascade -- --ignored
```

默认跑（不带 `--ignored`）七个用例全部跳过，全绿。`--ignored` 跑出 C-02 的这张票，红点为水密断言与它的诊断串。
