---
name: "wechat-comments-cli"
description: "微信公众号留言管理CLI工具 (Rust版本)。Invoke when user needs to manage WeChat official account comments, configure credentials, or interact with the comment API."
---

# 微信公众号留言管理CLI工具

这是一个基于Rust开发的微信公众号留言管理CLI工具，支持所有留言管理API功能，所有凭证采用AES-256-GCM加密存储。

## 功能特性

- 🔐 **加密存储**: APPID和APPSECRET采用AES-256-GCM加密存储
- 📝 **完整API**: 支持所有留言管理API
- 📄 **JSON输出**: 所有命令返回JSON格式数据
- 🔄 **Token缓存**: Access Token自动缓存刷新
- 🚀 **零密码**: 配置后无需输入密码，直接使用

---

## 一、配置命令

### 1. 配置凭证 (首次使用必须执行)

#### 方式1：交互式配置（推荐）
```bash
wechat-comments config
```
运行后会提示输入APPID和APPSECRET。

#### 方式2：命令行参数配置
```bash
wechat-comments config --app-id "wx1234567890abcdef" --app-secret "1234567890abcdef1234567890abcdef"
```

#### 示例输出：
```json
{
  "success": true,
  "message": "配置已保存并加密",
  "config_dir": "C:\\Users\\用户名\\.config\\wechat-comments"
}
```

### 2. 查看配置状态
```bash
wechat-comments status
```

#### 示例输出：
```json
{
  "config_exists": true,
  "config_dir": "C:\\Users\\用户名\\.config\\wechat-comments",
  "message": "配置文件存在"
}
```

### 3. 清除配置
```bash
wechat-comments clear
```

#### 示例输出：
```json
{
  "success": true,
  "message": "配置已清除"
}
```

---

## 二、msg_data_id 详细说明

### 什么是 msg_data_id？
`msg_data_id` 是微信公众号群发消息的唯一标识符。每一次群发消息（图文、视频、语音等）都会有一个唯一的 `msg_data_id`。

### 如何获取 msg_data_id？

#### 方法1：通过群发接口返回值获取
当你调用微信公众号的 **群发消息接口** 时，返回的 JSON 中包含 `msg_data_id`：

```json
{
  "errcode": 0,
  "errmsg": "send job submission success",
  "msg_id": 2345678901,
  "msg_data_id": 1234567890
}
```

#### 方法2：通过获取群发状态接口获取
调用 `getmasssendjob` 接口可以获取已发送的群发消息状态，其中包含 `msg_data_id`：

```json
{
  "msg_id": 2345678901,
  "msg_status": "SEND_SUCCESS",
  "msg_data_id": 1234567890
}
```

#### 方法3：通过素材管理接口获取
某些素材管理接口也会返回 `msg_data_id`，具体参考微信官方文档。

### 实际使用示例

假设你群发了一篇图文消息，返回的 `msg_data_id` 是 `1234567890123456`，那么你可以这样使用：

```bash
# 打开这篇文章的评论
wechat-comments open --msg-data-id 1234567890123456

# 查看这篇文章的评论列表
wechat-comments list --msg-data-id 1234567890123456
```

### 关于多图文消息的 index 参数
如果你的群发消息是 **多图文消息**（一次发送多篇文章），那么需要使用 `--index` 参数来指定具体是第几篇文章：

- `--index 0` 表示第一篇文章（默认）
- `--index 1` 表示第二篇文章
- `--index 2` 表示第三篇文章
- 以此类推

示例：
```bash
# 打开多图文消息中第二篇文章的评论
wechat-comments open --msg-data-id 1234567890123456 --index 1
```

---

## 三、留言管理命令

### 命令通用格式

配置完成后，所有命令无需输入密码，直接使用：

```bash
wechat-comments <command> [options]
```

---

### 1. 打开文章评论

**用途**：打开已群发文章的评论功能。

#### 命令格式：
```bash
wechat-comments open \
  --msg-data-id <MSG_DATA_ID> \
  [--index <多图文索引>]
```

#### 示例1：打开单图文文章评论
```bash
wechat-comments open --msg-data-id 1234567890123456
```

#### 示例2：打开多图文第二篇文章评论
```bash
wechat-comments open \
  --msg-data-id 1234567890123456 \
  --index 1
```

#### 返回示例：
```json
{
  "errcode": 0,
  "errmsg": "ok"
}
```

---

### 2. 关闭文章评论

**用途**：关闭已群发文章的评论功能。

#### 命令格式：
```bash
wechat-comments close \
  --msg-data-id <MSG_DATA_ID> \
  [--index <多图文索引>]
```

#### 示例：
```bash
wechat-comments close --msg-data-id 1234567890123456
```

#### 返回示例：
```json
{
  "errcode": 0,
  "errmsg": "ok"
}
```

---

### 3. 查看评论列表

**用途**：查看指定文章的所有评论或精选评论。

#### 命令格式：
```bash
wechat-comments list \
  --msg-data-id <MSG_DATA_ID> \
  [--index <多图文索引>] \
  [--begin <开始位置>] \
  [--count <获取数量>] \
  [--comment-type <评论类型>]
```

#### 参数说明：
| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--begin` | 起始位置，0表示第一条 | 0 |
| `--count` | 获取数量，最大50 | 10 |
| `--comment-type` | 评论类型：0=精选评论，1=全部评论 | 1 |

#### 示例1：查看全部评论（默认）
```bash
wechat-comments list --msg-data-id 1234567890123456
```

#### 示例2：查看精选评论
```bash
wechat-comments list \
  --msg-data-id 1234567890123456 \
  --comment-type 0
```

#### 示例3：分页获取评论
```bash
# 第一页（0-10条）
wechat-comments list \
  --msg-data-id 1234567890123456 \
  --begin 0 \
  --count 10

# 第二页（10-20条）
wechat-comments list \
  --msg-data-id 1234567890123456 \
  --begin 10 \
  --count 10
```

#### 返回示例：
```json
{
  "errcode": 0,
  "errmsg": "ok",
  "total": 25,
  "comment": [
    {
      "comment_id": 10000001,
      "openid": "o1234567890abcdef1234567890",
      "content": "这篇文章写得太好了，学到了很多知识！",
      "create_time": 1776691200,
      "is_elected": true,
      "nickname": "读者小明",
      "logo_url": "https://wx.qlogo.cn/mmopen/...",
      "reply": [
        {
          "reply_id": 20000001,
          "content": "感谢您的支持！",
          "create_time": 1776692000
        }
      ]
    },
    {
      "comment_id": 10000002,
      "openid": "o1234567890abcdef1234567891",
      "content": "请问这个功能怎么使用？",
      "create_time": 1776691300,
      "is_elected": false,
      "nickname": "读者小红",
      "logo_url": "https://wx.qlogo.cn/mmopen/...",
      "reply": []
    }
  ]
}
```

#### 返回字段说明：
| 字段 | 说明 |
|------|------|
| `comment_id` | 评论唯一ID，用于后续操作（标记精选、删除、回复等） |
| `openid` | 用户的OpenID |
| `content` | 评论内容 |
| `create_time` | 评论创建时间戳 |
| `is_elected` | 是否为精选评论 |
| `nickname` | 用户昵称 |
| `logo_url` | 用户头像URL |
| `reply` | 作者回复列表（可多条） |

---

### 4. 标记精选评论

**用途**：将指定评论标记为精选评论，显示在评论区最前面。

#### 命令格式：
```bash
wechat-comments mark-elect \
  --msg-data-id <MSG_DATA_ID> \
  --comment-id <评论ID> \
  [--index <多图文索引>]
```

#### 示例：
```bash
# 先查看评论列表获取 comment_id
wechat-comments list --msg-data-id 1234567890123456

# 假设返回的 comment_id 是 10000001，标记为精选
wechat-comments mark-elect \
  --msg-data-id 1234567890123456 \
  --comment-id 10000001
```

#### 返回示例：
```json
{
  "errcode": 0,
  "errmsg": "ok"
}
```

---

### 5. 取消精选评论

**用途**：将已标记为精选的评论取消精选状态。

#### 命令格式：
```bash
wechat-comments unmark-elect \
  --msg-data-id <MSG_DATA_ID> \
  --comment-id <评论ID> \
  [--index <多图文索引>]
```

#### 示例：
```bash
wechat-comments unmark-elect \
  --msg-data-id 1234567890123456 \
  --comment-id 10000001
```

#### 返回示例：
```json
{
  "errcode": 0,
  "errmsg": "ok"
}
```

---

### 6. 删除评论

**用途**：删除指定的用户评论。

#### 命令格式：
```bash
wechat-comments delete \
  --msg-data-id <MSG_DATA_ID> \
  --comment-id <评论ID> \
  [--index <多图文索引>]
```

#### 示例：
```bash
wechat-comments delete \
  --msg-data-id 1234567890123456 \
  --comment-id 10000001
```

#### 返回示例：
```json
{
  "errcode": 0,
  "errmsg": "ok"
}
```

---

### 7. 回复评论

**用途**：作者回复用户的评论。

#### 命令格式：
```bash
wechat-comments reply \
  --msg-data-id <MSG_DATA_ID> \
  --comment-id <评论ID> \
  --content "回复内容" \
  [--index <多图文索引>]
```

#### 示例1：回复简单内容
```bash
wechat-comments reply \
  --msg-data-id 1234567890123456 \
  --comment-id 10000001 \
  --content "感谢您的支持和认可！"
```

#### 示例2：回复带换行的内容
```bash
wechat-comments reply \
  --msg-data-id 1234567890123456 \
  --comment-id 10000002 \
  --content "您好！\n这个功能的使用方法是这样的：\n1. 首先...\n2. 然后...\n3. 最后..."
```

#### 示例3：带 emoji 的回复
```bash
wechat-comments reply \
  --msg-data-id 1234567890123456 \
  --comment-id 10000003 \
  --content "太棒了！🎉 感谢您的分享！"
```

#### 返回示例：
```json
{
  "errcode": 0,
  "errmsg": "ok",
  "reply_id": 20000001
}
```

---

### 8. 删除回复

**用途**：删除作者已发表的回复。

#### 命令格式：
```bash
wechat-comments delete-reply \
  --msg-data-id <MSG_DATA_ID> \
  --comment-id <评论ID> \
  --reply-id <回复ID> \
  [--index <多图文索引>]
```

#### 如何获取 reply_id？
从 `list` 命令返回的 `reply` 数组中获取：

```json
"reply": [
  {
    "reply_id": 20000001,  // 这就是 reply_id
    "content": "感谢您的支持！",
    "create_time": 1776692000
  }
]
```

#### 示例：
```bash
wechat-comments delete-reply \
  --msg-data-id 1234567890123456 \
  --comment-id 10000001 \
  --reply-id 20000001
```

#### 返回示例：
```json
{
  "errcode": 0,
  "errmsg": "ok"
}
```

---

## 四、完整工作流程示例

### 场景：公众号运营人员管理文章评论

#### 步骤1：首次配置凭证
```bash
wechat-comments config
```
按提示输入：
- APPID: `wx1234567890abcdef`
- APPSECRET: `1234567890abcdef1234567890abcdef`

#### 步骤2：群发文章后获取 msg_data_id
假设群发消息后返回：
```json
{
  "msg_data_id": 9876543210987654,
  "msg_id": 1234567890
}
```

#### 步骤3：打开文章评论
```bash
wechat-comments open --msg-data-id 9876543210987654
```

#### 步骤4：查看所有评论
```bash
wechat-comments list \
  --msg-data-id 9876543210987654 \
  --count 20 \
  --comment-type 1
```

#### 步骤5：精选优质评论
```bash
wechat-comments mark-elect \
  --msg-data-id 9876543210987654 \
  --comment-id 10000001

wechat-comments mark-elect \
  --msg-data-id 9876543210987654 \
  --comment-id 10000003
```

#### 步骤6：回复用户评论
```bash
wechat-comments reply \
  --msg-data-id 9876543210987654 \
  --comment-id 10000002 \
  --content "您好！感谢您的提问。关于这个问题，我们的建议是..."
```

#### 步骤7：删除不当评论
```bash
wechat-comments delete \
  --msg-data-id 9876543210987654 \
  --comment-id 10000005
```

#### 步骤8：查看当前精选评论
```bash
wechat-comments list \
  --msg-data-id 9876543210987654 \
  --comment-type 0
```

---

## 五、参数速查表

### 全局参数

| 参数 | 简写 | 说明 | 默认值 | 适用命令 |
|------|------|------|--------|----------|
| `--msg-data-id` | `-m` | 消息数据ID（群发消息ID） | 必填 | open, close, list, mark-elect, unmark-elect, delete, reply, delete-reply |
| `--index` | `-i` | 多图文索引 | 0 | 上述所有命令（可选） |
| `--comment-id` | `-c` | 评论ID | 必填 | mark-elect, unmark-elect, delete, reply, delete-reply |
| `--reply-id` | `-r` | 回复ID | 必填 | delete-reply |
| `--content` | `-o` | 回复内容 | 必填 | reply |
| `--begin` | `-b` | 起始位置 | 0 | list |
| `--count` | `-n` | 获取数量，最大50 | 10 | list |
| `--comment-type` | `-t` | 评论类型：0=精选，1=全部 | 1 | list |
| `--app-id` | `-a` | APPID | 可选 | config |
| `--app-secret` | `-s` | APPSECRET | 可选 | config |

---

## 六、错误码参考

| 错误码 | 说明 |
|--------|------|
| 0 | 成功 |
| -1 | 系统繁忙 |
| 40001 | 获取 access_token 时 AppSecret 错误 |
| 40014 | 不合法的 access_token |
| 42001 | access_token 已过期 |
| 45009 | 已达到回复上限 |
| 45010 | 留言条数达到上限 |
| 45011 | 留言内容超过限制 |
| 48001 | api功能未授权 |
| 49003 | 留言不存在 |
| 49004 | 留言不是当前公众号的 |
| 49005 | 回复内容超过限制 |
| 49006 | 留言已被删除 |

---

## 七、安装方法

### 方式1：Cargo安装（开发者推荐）
```bash
cargo install --path .
```

### 方式2：编译后安装
```bash
cargo build --release
```

Windows：复制 `target/release/wechat-comments.exe` 到 PATH 目录  
Linux/macOS：复制 `target/release/wechat-comments` 到 `/usr/local/bin/`

### 方式3：直接运行（开发测试）
```bash
cargo run -- config
cargo run -- list --msg-data-id 1234567890
```

---

## 八、安全说明

1. **凭证加密**: APPID和APPSECRET使用AES-256-GCM加密存储
2. **自动密钥生成**: 配置时自动生成256位随机密钥，无需用户设置密码
3. **密钥分离**: 密钥存储在 `key.bin` 文件，加密凭证存储在 `credentials.enc` 文件
4. **Token缓存**: Access Token单独存储，带过期时间自动刷新
5. **安全建议**: 
   - 妥善保管配置目录
   - 不要将 `key.bin` 文件暴露给他人
   - 如需迁移配置，需同时复制 `key.bin` 和 `credentials.enc`

---

## 九、配置文件存储位置

- **Windows**: `C:\Users\<用户名>\.config\wechat-comments\`
- **macOS**: `~/.config/wechat-comments/`
- **Linux**: `~/.config/wechat-comments/`

包含文件：
- `key.bin` - AES-256加密密钥（32字节二进制文件）
- `credentials.enc` - 加密的凭证文件（Base64编码的JSON）
- `token.json` - Access Token缓存文件（明文JSON）

---

## 十、注意事项

1. **权限要求**: 公众号需具备留言功能权限
2. **认证要求**: 必须是已认证的服务号
3. **消息类型**: 只有已群发的消息才能打开评论
4. **评论限制**: 每篇文章最多精选100条评论
5. **回复限制**: 每条评论最多回复1次（作者回复）
6. **字符限制**: 评论和回复内容最大600字节
7. **配置迁移**: 迁移配置时，需要同时复制 `key.bin` 和 `credentials.enc` 两个文件

---

*参考微信官方文档：https://developers.weixin.qq.com/doc/subscription/guide/product/comments.html*
