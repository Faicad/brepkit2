# Terminology

Project glossary for brepkit2 — a B-Rep modeling kernel in Rust exposed to JavaScript via WASM.
Terms are listed by area. "Code" means the term appears as an identifier in the codebase and should be kept in English in translations; the 中文 column is for prose/docs.


## Topology (brepkit-topology)

| English | 中文 | Notes |
|---|---|---|
| Topology | Topology | The arena owner struct; all entities live in it, referenced by ID |
| arena | 竞技场 / arena | Contiguous storage handing out typed `Id<T>` handles; O(1) lookup |
| `Id<T>` | `Id<T>` / 句柄 | Typed handle; `Id<Vertex>` cannot look up an `Edge` |
| Vertex | 顶点 | Position + tolerance radius (points within tolerance are the same vertex) |
| Edge | 边 | Curve segment bounded by start/end vertices; start == end means a closed (degenerate) edge, e.g. a full circle |
| EdgeCurve | 边曲线 | Enum of edge geometry: `Line`, `NurbsCurve`, `Circle`, `Ellipse` |
| Wire | 线框 / Wire | Ordered sequence of `OrientedEdge`s; open (path) or closed (loop) |
| OrientedEdge | 有向边 | Edge + traversal direction flag |
| Face | 面 | One outer wire + zero or more inner wires (holes) on a surface |
| FaceSurface | 面曲面 | Enum of face geometry: `Plane`, `Nurbs`, `Cylinder`, `Cone`, `Sphere`, `Torus` |
| outer wire | 外环 | A face's outer boundary loop |
| inner wire | 内环 / 孔环 | Hole/void loop inside a face |
| Shell | 壳 | Set of faces; closed shell bounds a volume, open shell is a sheet |
| Solid | 实体 | Outer shell + zero or more inner shells (cavity shells) |
| outer shell | 外壳 | The solid's exterior boundary |
| inner shell | 内壳 / 腔体壳 | Cavity/void shell produced by `shell_op` or boolean cuts |
| cavity | 内腔 | Void inside a solid bounded by an inner shell |
| Compound | 复合体 | Multiple disjoint solids treated as one modelling entity (e.g. boolean split result) |
| CompSolid | 复合实体 | Solids sharing connected boundary faces |
| AdjacencyIndex | 邻接索引 | Edge-to-face and face-neighbor maps; detects non-manifold/boundary edges |
| PCurve | 参数曲线 (pcurve) | Edge's 2D curve in the face's surface parameter space; kept in a `PCurveRegistry` keyed by (edge, face) |
| non-manifold | 非流形 | Topology where an edge has ≠ 2 adjacent faces |
| manifold | 流形 | Every edge has exactly two adjacent faces |
| Explorer | Explorer / 遍历器 | Shape explorer iterating an entity's children |

## Geometry & Math (brepkit-math, brepkit-geometry)

| English | 中文 | Notes |
|---|---|---|
| NURBS | NURBS / 非均匀有理B样条 | Native geometry representation for curves and surfaces |
| control point | 控制点 | |
| weight | 权重 | Non-unit weights make a curve/surface rational |
| rational | 有理 | Rational B-spline carries weights; STEP round-trip must preserve them (RATIONAL_B_SPLINE_* complexes) |
| knot / knot vector | 节点 / 节点向量 | |
| degree | 次数 | |
| B-spline | B样条 | |
| Bezier | 贝塞尔 | Decomposition target for NURBS; Bezier clipping intersection |
| power basis | 幂基 | Polynomial Horner evaluation form |
| analytic surface | 解析曲面 | Plane/Cylinder/Cone/Sphere/Torus with exact closed forms |
| analytic curve | 解析曲线 | Line/Circle/Ellipse/Parabola/Hyperbola |
| LSPIA | LSPIA | Least-squares progressive iterative approximation (curve/surface fitting) |
| AABB | 轴对齐包围盒 | Axis-aligned bounding box |
| OBB | 有向包围盒 | Oriented bounding box (PCA + SAT) |
| BVH | BVH / 层次包围盒 | Bounding volume hierarchy |
| CDT | 约束Delaunay三角剖分 | Constrained Delaunay triangulation |
| convex hull | 凸包 | |
| filtered predicates | 过滤式精确谓词 | Exact geometric predicates (orient2d/3d) with filtering |
| tolerance | 容差 | `Tolerance` struct: linear 1e-7, angular 1e-12; compare via `approx_eq`, never `==` |
| chord deviation | 弦高偏差 | Arc discretization criterion |
| deflection | 弦偏差 / 偏差量 | Adaptive sampling criterion |
| extrema | 极值 | Point-curve / curve-curve / point-surface distance problems |
| parameter space | 参数空间 | UV space of a surface |
| UV | UV | Surface parameters (u, v) |
| seam | 接缝 | Periodic surface closure line (cylinders, tori) |
| degenerate | 退化 | Zero-length edge, collapsed pole, etc. |

## Algorithms

| English | 中文 | Notes |
|---|---|---|
| GFA | GFA | Global Face Approximation boolean engine (brepkit-algo) — the boolean kernel |
| Pave / PaveBlock | Pave / Pave 分块 | GFA data structures for splitting intersecting faces |
| PaveFiller | PaveFiller | GFA orchestrator; phases VV/VE/EE/VF/EF/FF (vertex/edge/face intersection phases) |
| face classification | 面分类 | Determining kept/discarded/same parts of faces in a boolean |
| FaceInfo | FaceInfo | Per-face classification state in GFA |
| boolean | 布尔运算 | Union / cut / intersect |
| fuse | 求并 / 融合 | Boolean union |
| cut | 求差 / 裁剪 | Boolean cut |
| intersect | 求交 | Boolean intersection |
| mesh fallback | 网格回退 | Boolean falling back from exact B-Rep to triangulated mesh (surfaces degrade to planes) — a failure symptom, not a feature |
| mesh boolean | 网格布尔 | Co-refinement mesh boolean in operations |
| analytic classifier | 解析分类器 | Seven analytic point-classification variants in algo |
| ray casting | 射线投射 | Point-in-solid classification by ray crossing counts |
| winding number | 环绕数 | Signed solid-angle point classifier |
| majority vote | 多数投票 | Three irrational ray directions vote on inside/outside |
| fillet | 圆角 | Rolling-ball blend on edges |
| chamfer | 倒角 | Straight-cut edge treatment |
| blend | 过渡面 / blend | Fillet band surface (stripe) between contact curves |
| spine | 脊线 | Edge chain with arc-length parameterization driving a blend |
| radius law | 半径规律 | Constant / linear / S-curve / custom radius along a fillet |
| vertex blend / corner | 顶角过渡 | Corner solver where ≥ 3 fillets meet |
| rolling ball | 滚球 | Classic fillet construction metaphor |
| walking | 行走法 | Newton-Raphson marching to trace the blend band |
| offset | 偏移 | Solid/face/wire offsetting |
| thick solid / shell | 抽壳 / 加厚 | Hollowing a solid (`shell_op`, `thick_solid`) |
| draft | 拔模 | Tapering faces |
| sweep | 扫掠 | Profile along a path |
| loft | 放样 | Interpolation across sections |
| revolve | 旋转 | Profile swept around an axis |
| extrude | 拉伸 | Profile along a straight direction |
| pipe | 管道 | Circular sweep along a path |
| helical sweep | 螺旋扫掠 | Sweep along a helix |
| pattern | 阵列 | Linear / circular replication |
| mirror | 镜像 | |
| section | 剖切 | Cross-section of a shape by a plane |
| defeaturing | 去特征化 | Removing features from a shape |
| feature recognition | 特征识别 | Detecting holes, fillets etc. in a shape |
| healing | 修复 / shape healing | Analysis + fixing of invalid topology (gaps, ordering, SameParameter, small faces) |
| sewing | 缝合 | Stitching free edges of shells together |
| unify same domain | 合并同域面 | Merging coplanar / co-cylindrical faces |
| upgrade | 升级 | Heal-stage improvements (convert to Bezier/BSpline, remove internal wires) |
| tessellation | 网格化 / 三角剖分 | Meshing faces to triangles |
| deflection (tessellation) | 弦差 | Max deviation between mesh and surface |
| Coons patch | Coons 曲面 | Face filling method |
| GCS | GCS / 几何约束系统 | Geometric constraint system in brepkit-sketch |
| DogLeg | DogLeg | Trust-region solver used by GCS |
| DOF | 自由度 | Degrees of freedom analysis in the sketch solver |
| constraint | 约束 | Sketch constraint (10 variants) with Jacobians |

## I/O & File Formats

| English | 中文 | Notes |
|---|---|---|
| STEP | STEP | ISO 10303-21 B-Rep exchange format (AP203) |
| IGES | IGES | Legacy B-Rep exchange format |
| 3MF | 3MF | Mesh container format |
| STL | STL | Triangle mesh format |
| OBJ | OBJ | Mesh format |
| PLY | PLY | Mesh format |
| glTF | glTF | Mesh/scene format |
| round-trip | 往返 | Write then read must reproduce identical geometry (analytic round-trip check) |
| B_SPLINE_CURVE_WITH_KNOTS | — | STEP entity for B-spline curves |
| RATIONAL_B_SPLINE_CURVE / SURFACE | — | STEP weighted complexes that carry rational weights |
| ADVANCED_FACE / CLOSED_SHELL / MANIFOLD_SOLID_BREP | — | STEP topology entities produced by the writer |
| fixture | 测试夹具 | Captured real-world file used as a regression input |

## Validation & Measurement (brepkit-check, operations/measure)

| English | 中文 | Notes |
|---|---|---|
| classification | 点分类 | Inside / Outside / OnBoundary relative to a solid |
| validation | 校验 | CheckId-driven validation report (wire/shell/solid/vertex/edge/face checks) |
| Euler check | 欧拉检查 | Euler characteristic consistency of a solid |
| properties | 几何属性 | Volume, area, center of mass via Gauss integration (Huygens' theorem) |
| GProps | GProps | Geometric property accumulator |
| golden file | 黄金文件 | Checked-in expected output for regression comparison |
| AABB (measure) | 包围盒测量 | Bounding-box measurement; must cover inner-shell (cavity) faces |

## WASM / JS API (brepkit-wasm)

| English | 中文 | Notes |
|---|---|---|
| BrepKernel | BrepKernel | The single wasm-bindgen exposed class; all bindings are its methods |
| binding | 绑定 | `#[wasm_bindgen]` method exposing a kernel operation to JS |
| `#[wasm_binding]` | `#[wasm_binding]` | Proc macro wrapping bindings for panic safety |
| js_name | js_name | camelCase JS name attribute, e.g. `myOperation` |
| entity handle | 实体句柄 | Entity IDs returned to JS as `u32` numbers |
| resolve_* | resolve_* | Handle-resolution helpers (id → arena entity) |
| batch / executeBatch | 批量执行 | Batch dispatch of multiple kernel operations |
| checkpoint | 检查点 | Save/restore kernel state |
| JsError | JsError | JS-visible error type; WasmError converts via `From` |
| contract test | 契约测试 | Tests through `execute_batch()` (JsError can't be constructed on non-wasm targets) |
| tsify | tsify | serde→TypeScript type generation for result structs |

## Rendering (brepkit-render)

| English | 中文 | Notes |
|---|---|---|
| offscreen render | 离屏渲染 | wgpu rendering to image, no window |
| face-id buffer | 面ID缓冲 | Picking buffer mapping pixels to faces |
| compute mesher | 计算着色器网格化 | GPU compute mesher for analytic quadrics |
| viewer | 查看器 | Interactive viewer behind the `window` feature |

## Workflow & Conventions

| English | 中文 | Notes |
|---|---|---|
| snapshot then allocate | 先快照后分配 | Borrow-checker pattern: read all data into locals before mutating the arena |
| exhaustive match | 穷尽匹配 | `EdgeCurve`/`FaceSurface` matches have no `_` wildcards; adding a variant flags every site |
| ripple effect | 连锁修改 | Adding an enum variant requires updating all match sites (compiler-driven) |
| solid_faces | solid_faces | `topology::explorer::solid_faces` — flattens outer + inner shells; default over per-shell iteration |
| boundary violation | 层级违规 | Cargo dependency crossing layer rules; fails `check-boundaries.sh` |
| conventional commit | 规范化提交 | commitlint-enforced commit format |
| profiling profile | profiling 配置 | Release profile with debug symbols, no LTO, for flamegraphs |
| proptest | 属性测试 | Property-based testing |
