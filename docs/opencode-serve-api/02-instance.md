# Instance 实例

> [← 返回目录](README.md)

---

### `POST /instance/dispose`

- **OperationId**: `instance.dispose`
- **摘要**: Dispose instance
- **说明**: Clean up and dispose the current OpenCode instance, releasing all resources.

**请求参数**

| 位置 | 参数名 | 类型 | 必填 | 说明 |
|------|--------|------|------|------|
| query | `directory` | string | 可选 |  |

**响应**

| 状态码 | 说明 | 类型/引用 |
|--------|------|-----------|
| 200 | Instance disposed | `boolean` |

---
