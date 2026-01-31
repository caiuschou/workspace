# Crush Agent Skills 实现

> 基于 `thirdparty/crush` 源码整理。Crush 实现 [Agent Skills 开放标准](https://agentskills.io)，通过 `SKILL.md` 技能包扩展 Agent 能力。

## 逻辑说明（一句话版）

- **技能是什么**：一个文件夹里放一个 `SKILL.md`，前面写名字和简介（YAML），后面写具体操作说明（Markdown）。
- **怎么被发现**：启动时按配置的路径（如 `~/.config/crush/skills`、项目里的 `./skills`）扫一遍，凡是有 `SKILL.md` 的文件夹就当成一个技能。
- **Agent 怎么知道**：把这些技能的名字、简介、文件路径拼成一段 XML，塞进系统提示词，告诉模型「当前有哪些技能、每个技能的文件在哪」。
- **Agent 怎么用**：用户任务和某个技能的简介对上了，模型就用「读文件」工具去读那个技能的 `SKILL.md`，按里面的说明干活；读技能目录下的文件时不做大小限制、也不额外要权限。

**技能匹配**：是否用某个技能、用哪一个，完全由 **LLM 在推理时**根据用户任务和 prompt 里的技能简介（description）决定；没有单独的检索服务或 embedding 匹配。

**上下文与规模**：进系统提示词的只有每个技能的**元信息**（name + description + location），完整 SKILL.md 是模型决定用该技能后才按需读取。技能越多，`<available_skills>` 这段就越长，系统提示的 token 会线性增加，属于「用上下文换 LLM 自选技能」的取舍；若技能数量很大，可考虑只配置当前需要的路径（如项目内 `./skills`）以控制长度。

## 1. 规范与约定

- **规范**：https://agentskills.io
- **技能定义**：每个技能是一个**目录**，内含 `SKILL.md` 文件
- **SKILL.md**：必须以 YAML frontmatter 开头（`---` … `---`），包含 `name`、`description`；frontmatter 之后为 Markdown 正文，即技能说明（Instructions）

## 2. 代码结构

| 路径 | 职责 |
|------|------|
| `internal/skills/skills.go` | 解析 SKILL.md、校验、发现目录、生成 prompt 用 XML |
| `internal/skills/skills_test.go` | Parse / Validate / Discover / ToPromptXML 单元测试 |
| `internal/config/config.go` | `Options.SkillsPaths []string` |
| `internal/config/load.go` | 默认 SkillsPaths、`GlobalSkillsDirs()` |
| `internal/agent/prompt/prompt.go` | 调用 `skills.Discover` 与 `ToPromptXML`，写入 prompt 数据 |
| `internal/agent/templates/coder.md.tpl` | 注入 `<available_skills>` 与 `<skills_usage>` |
| `internal/agent/tools/view.go` | 对 skills 路径下的文件放宽读限制、权限逻辑 |
| `internal/agent/coordinator.go` | 创建 View 工具时传入 `Options.SkillsPaths` |

## 3. 核心类型与常量（skills.go）

```go
const (
	SkillFileName          = "SKILL.md"
	MaxNameLength          = 64
	MaxDescriptionLength   = 1024
	MaxCompatibilityLength = 500
)

type Skill struct {
	Name          string            // 必填，与目录名一致，小写字母数字+单连字符
	Description   string            // 必填
	License       string            // 可选
	Compatibility string            // 可选
	Metadata      map[string]string // 可选
	Instructions  string            // frontmatter 之后的正文
	Path          string            // 技能目录路径
	SkillFilePath string            // SKILL.md 完整路径
}
```

- **name**：`^[a-zA-Z0-9]+(-[a-zA-Z0-9]+)*$`，不得以连字符开头/结尾，不得连续连字符；长度 ≤ 64；若提供 Path，则需与目录名一致（不区分大小写）
- **description**：必填，长度 ≤ 1024
- **compatibility**：长度 ≤ 500

## 4. 发现路径

- **配置项**：`options.skills_paths`（`crush.json`）
- **默认全局目录**（`GlobalSkillsDirs()`）：
  - 若设置 `CRUSH_SKILLS_DIR`，则仅使用该路径
  - 否则：
    - Unix：`$XDG_CONFIG_HOME/crush/skills` 或 `~/.config/crush/skills`
    - Windows：`%LOCALAPPDATA%\crush\skills` 等
  - 以及：`<configBase>/agents/skills`
- **加载逻辑**（load.go）：若用户未配置 `SkillsPaths`，会先设为空切片，再在默认合并阶段把 `GlobalSkillsDirs()` 加入 `Options.SkillsPaths`，因此默认会扫描上述全局目录；用户配置的 `skills_paths` 会与默认目录合并（去重）。

## 5. 数据流

1. **配置**：`cfg.Options.SkillsPaths` 来自 crush.json 的 `options.skills_paths` 与默认全局目录
2. **发现**：`internal/agent/prompt/prompt.go` 中对 `SkillsPaths` 做路径展开后调用 `skills.Discover(expandedPaths)`，得到 `[]*Skill`
3. **注入 prompt**：`skills.ToPromptXML(discoveredSkills)` 生成 `<available_skills>` XML，写入 `PromptDat.AvailSkillXML`
4. **模板**：`coder.md.tpl` 中若 `AvailSkillXML` 非空，则输出该 XML 及 `<skills_usage>` 说明（何时读 SKILL.md、按 location 激活、脚本/资源同目录等）
5. **读文件**：Agent 使用 **View** 工具读文件；`view.go` 中通过 `isInSkillsPath(absFilePath, skillsPaths)` 判断是否在技能目录下；若是，则对该文件（如 SKILL.md）不应用普通文件大小限制、且不因“在工作目录外”而额外要权限，从而便于按 location 读取技能内容。

## 6. SKILL.md 解析（skills.go）

- **Parse(path)**：读文件 → 用 `splitFrontmatter` 拆出 frontmatter 与 body → YAML 反序列化到 `Skill`，`Instructions` 取 body 的 trim；`Path` 为目录，`SkillFilePath` 为完整路径
- **splitFrontmatter**：要求内容以 `---\n` 开头，且存在 `\n---` 结束；中间为 frontmatter，之后为 body
- **Validate()**：校验 name/description 必填与长度、name 正则、name 与目录名一致、compatibility 长度

## 7. Discover（skills.go）

- **Discover(paths)**：对每个 path 使用 `fastwalk.Walk`（Follow 符号链接），找出所有名为 `SKILL.md` 的文件；对每个文件调用 `Parse` 与 `Validate`，通过则加入结果；用 `seen` 去重（同一 path 只处理一次）

## 8. ToPromptXML（skills.go）

- 将 `[]*Skill` 转为 XML 字符串，供 prompt 使用：
  - 根节点 `<available_skills>`
  - 每个技能：`<skill>` 内 `<name>`、`<description>`、`<location>`（即 `SkillFilePath`）；对内容做 XML 转义

## 9. 配置示例（crush.json）

```json
{
  "$schema": "https://charm.land/crush.json",
  "options": {
    "skills_paths": [
      "~/.config/crush/skills",
      "./project-skills"
    ]
  }
}
```

## 10. 示例技能来源

官方 README 建议从 [anthropics/skills](https://github.com/anthropics/skills) 获取示例技能，克隆后将其中的 `skills/*` 放到上述目录即可。

---

| 任务 | 状态 |
|------|------|
| 整理 Crush 与 OpenCode 关系、文档索引 | 已完成 |
| 整理 Agent Skills 实现（规范、代码结构、数据流、配置） | 已完成 |
