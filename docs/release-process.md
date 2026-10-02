# Release Process — @faicad/brepkit2-wasm

本文档描述 npm 包 `@faicad/brepkit2-wasm` 的发版流程。版本号和 CHANGELOG 由
git 历史自动推导（Conventional Commits 语义化版本），无需手工维护版本号。

> 2026-10-01：原上游 brepkit 使用 release-please（GitHub Actions 自动发版 PR），
> fork 时该配置已被移除（见 `6a65110`），改为本地的 xtask 流程。

## 版本号规则（Conventional Commits → SemVer）

xtask 扫描**自上次版本号变更以来的所有 commit**（以 `crates/wasm/Cargo.toml`
中 `version =` 行最后一次被修改的 commit 为基线），取最高级别的 bump：

| Commit 类型 | Bump |
|-------------|------|
| `feat!:` / `fix!:` / 任意类型带 `!` / body 含 `BREAKING CHANGE` | **major**（默认钳制为 minor，见下） |
| `feat` | **minor** |
| `fix` / `perf` / `refactor` / `revert` | **patch** |
| `chore` / `docs` / `test` / `ci` / `style` / `build` | 不触发 bump |

- 只有 maintenance 类 commit（无 bump 触发）时，release 会拒绝执行。
- **major 钳制**：fork 继承上游版本线（当前 2.129.x），breaking change 默认
  只 bump minor，避免与上游语义冲突。需要真 major 时加 `--allow-major`。
- 当前未触发 bump 的 commit 也会随同版本一起发布（它们已包含在 diff 里）。

## CHANGELOG 生成

遵循 [Keep a Changelog](https://keepachangelog.com/) 格式，写入根目录
`CHANGELOG.md`：

- 新版本块插在 `## [Unreleased]` 之下；Unreleased 里已有的手写内容会并入
  新版本块。
- commit 分组：`feat`→**Added**，`fix`/`perf`→**Fixed**，
  `refactor`/`revert`→**Changed**，`docs`→**Documentation**。
- `chore`/`test`/`ci`/`style`/`build` 不进 CHANGELOG（不影响包使用者）。
- 每条带短 hash，breaking 条目加 `**BREAKING**` 前缀。
- `crates/wasm/CHANGELOG.md` 是上游遗留的占位文件，指向根 CHANGELOG，不维护。

## 发版命令

### 标准流程（推荐）

```powershell
# 1. 预演：看会算出什么版本、CHANGELOG 长什么样，不改任何文件
.\scripts\publish-wasm.ps1 -AutoVersion -DryRun

# 2. 确认无误后正式发版（bump 版本 + 更新 CHANGELOG + 构建 + 发布）
.\scripts\publish-wasm.ps1 -AutoVersion
```

可选参数：

| 参数 | 作用 |
|------|------|
| `-Tag next` | 发到 `next` dist-tag（首发建议，冒烟通过后再 promote 到 latest） |
| `-AllowMajor` | 允许 breaking change 升 major |
| `-Version 3.0.0` | 手工指定版本号，跳过自动计算 |
| `-DryRun` | 只算版本 + 构建校验，不改文件、不发布 |
| `-Otp 123456` | npm 2FA 一次性密码 |
| `-Provenance` | 仅 CI（需要 OIDC 环境），本地勿用 |

等价的底层调用：

```bash
cargo xtask wasm-release --dry-run   # 版本计算 + CHANGELOG 预览 + 构建校验
cargo xtask wasm-release             # 同上但会写 Cargo.toml 和 CHANGELOG.md
```

`wasm-release` 只负责版本与构建；smoke 测试和 `npm publish` 由
`publish-wasm.ps1` 统一执行（dist-tag / OTP / 发布记录都在脚本侧）。

### 纯构建 / 手工版本发布

```powershell
.\scripts\publish-wasm.ps1 -DryRun        # 用当前 Cargo.toml 版本构建+校验
.\scripts\publish-wasm.ps1                # 发布当前版本（不 bump）
```

## 发版后必做

1. **提交版本变更**（脚本不会自动 commit）：

   ```bash
   git add crates/wasm/Cargo.toml CHANGELOG.md
   git commit -m "chore(release): v2.129.16"
   ```

2. **验证 registry**：脚本已内置发布后验证（warn-only，CDN 延迟只警告不报
   错）；也可手动确认：

   ```bash
   npm view @faicad/brepkit2-wasm dist-tags --registry https://registry.npmjs.org/
   ```

   注意：本机网络对 `registry.npmjs.org` 有 TLS 拦截时，可用
   `https://cdn.jsdelivr.net/npm/@faicad/brepkit2-wasm@<tag>/package.json`
   验证。

3. **promote dist-tag**（如果发到了 `next`）：

   ```bash
   npm dist-tag add @faicad/brepkit2-wasm@2.129.16 latest
   ```

## 管线内部结构

```
scripts/publish-wasm.ps1            ← 入口（pack 断言 / publish / registry 验证 / 发布记录）
  └─ cargo xtask wasm-release       ← 版本计算 + Cargo.toml/CHANGELOG 写入 + 构建
       ├─ xtask/src/release.rs      ← git log 解析、semver 计算、CHANGELOG 生成
       └─ xtask/src/wasm.rs         ← wasm-pack 双 target 构建、wasm-opt、合并、校验
```

- 版本单一事实来源：`crates/wasm/Cargo.toml` 的 `version`。pkg 的
  `package.json` 由 wasm-pack 自动生成，xtask 校验两者一致。
- npm 包名固定为 `@faicad/brepkit2-wasm`（`xtask/src/wasm.rs::NPM_PKG_NAME`），
  与 crate 名 `brepkit-wasm`（未发布）解耦。
