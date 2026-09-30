# Skill: brepkit2 Git Operations

在 brepkit2 仓库内执行 git 操作时，遵守以下约定。

## Workflow

1. 先 `git status --short` 看全局，分类未提交改动（已 commit / 新文件 / 修改）
2. 逐个 diff 审视，不盲目 `add -A`；按逻辑分组 `git add`，一次提交一个 coherent 变更
3. 提交前回顾**整个 diff**（不只最近几轮对话），按下方 Commit Message 规范写 message
4. `git commit` 会自动触发 husky pre-commit（fmt / clippy / taplo / machete）；任一失败则提交中止，先修再提交

## Commit 规则

**未跑测试时不准主动 commit。** 提交代码前必须跑测试，或者用户明确说"commit"、"提交"或类似指令后才能 `git commit`。

**不准单独提交文档。** 文档（AGENTS.md / docs/ / book/ / README 等）必须与它所描述 / 对应的代码改动放在同一次 commit 里，不要在没有相关代码改动时单独 `git commit` 文档。文档改动应随其实现一起入库。

**Commit message 不带任何 Co-Authored-By 尾部信息。** Conventional commits（英文），由 commitlint 强制。

## ⚡ 用户命令"提交代码"时的速度与范围铁律

1. **快，以最简步骤完成。** 用户一开口要求提交，立即按 `git status --short` → 确认范围 → `git add` → `git commit` 执行，整体尽量一分钟内完成。禁止深度分析、反复核对 diff、逐文件推演归属、先跑 CI / 决策确认。用户明确说"提交"就是直接指令，不要再请示分析。
   - 例外：需要时只做一次快速 `git status` 判断范围，不展开。
2. **只提交本次任务的代码 + 本次任务相关的文档。** 工作区若混杂了其它任务的改动（别的计划的草稿 `docs/plans/*`、其他人的删除 / 修改），**只 add 本次任务涉及的文件，不要 `git add -A` 一把全提交**；只有确认整个工作区都属于本次任务时才可 `git add -A`。
3. **误提交无关文件时的纠正**（允许，且速度优先）：`git reset --soft HEAD~1` 回撤本次 commit（工作区与暂存内容不动），再 `git restore --staged -- <无关文件>` 把它们拆出暂存区（文件保留在工作区，不丢），最后重新 `git commit`。**严禁 `git reset --hard` / `git restore <目录>`**。

## Commit Message 规范

- Conventional commits：`type(scope): subject`（英文，imperative）
- 常用 type：`feat` / `fix` / `docs` / `refactor` / `test` / `chore` / `perf` / `build` / `ci`
- scope 可用 crate 名（如 `operations`、`wasm`）省略 `brepkit-` 前缀
