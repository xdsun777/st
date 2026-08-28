# 个人刷题工具（st）

个人自用的本地离线刷题 App，支持自定义 CSV 题库导入、五种题型刷题、错题本、收藏、统计与备份恢复。**无账号、无网络、无广告**，所有数据保存在本机。

- 桌面端：Windows `.exe` / Linux `AppImage`（Tauri 2）
- 移动端：Android `.apk`（自用，不上应用商店）

## 功能特性

| 模块 | 功能 |
|---|---|
| 题库管理 | CSV 批量导入（字段校验 + 错误行提示）、题库集增删改、单题增删改、标签管理与筛选 |
| 刷题 | 顺序 / 随机模式；单选、多选、判断、填空、简答五种题型；上一题/下一题；中途退出自动保存进度、再次进入可续做 |
| 判分 | 单选/多选/判断/填空机器判分；填空支持手动覆写；简答手动标记对错 |
| 错题本 | 答错自动归集（按题去重、累计做错次数）、错题复习、移出错题仅清标记 |
| 收藏 | 刷题页一键收藏/取消收藏；收藏独立于错题 |
| 统计 | 总题量、刷题量、正确/错误数、整体正确率、分题库统计 |
| 备份 | 全量导出 `.qpbackup` 备份包；恢复前二次确认，覆盖本地全部数据 |

## 技术栈

- **应用外壳**：Tauri 2（Rust）
- **前端**：Vue 3 + TypeScript + Tailwind CSS + Pinia
- **数据库**：SQLite（Rust 侧 `sqlx` 连接池，六张表：`question_bank` / `question` / `tag` / `question_tag` / `answer_record` / `practice_session`）
- **CSV 解析**：PapaParse（前端解析校验，Rust 侧事务批量写入）

## 目录结构

```
├── src/                      # 前端
│   ├── api.ts                # Tauri Command 封装（前端不直接操作数据库/文件）
│   ├── types/index.ts        # TS 类型定义（与后端模型对齐）
│   ├── utils/judge.ts        # 判分纯函数（填空/多选/单选/判断）
│   ├── utils/csv.ts          # CSV 解析与字段校验
│   ├── stores/app.ts         # Pinia 全局状态 + 全局 Toast
│   ├── components/           # 题目卡片、各弹窗、标签组件
│   └── views/                # 题库/刷题/错题/收藏/统计/备份 六个页面
└── src-tauri/
    ├── src/commands/         # Rust 命令层（bank/question/tag/practice/record/stats/backup）
    ├── src/db.rs             # 建表迁移
    ├── src/models.rs         # 数据模型
    └── gen/android/          # Android Gradle 工程
```

## 快速开始

### 环境要求

- Node.js ≥ 20
- Rust stable（最新版）
- Linux 桌面运行需要 WebKitGTK 4.1（`webkit2gtk-4.1`）
- 构建 Android 需要 Android SDK + NDK（仅桌面端可跳过）

### 安装与运行

```bash
npm install          # 安装前端依赖
npm run tauri dev    # 桌面端开发运行（自动编译 Rust 并启动 Vite）
```

### 桌面端打包

```bash
npm run tauri build
```

### Android APK

```bash
# 首次初始化 Android 工程（需已配置 ANDROID_HOME 与 NDK）
npm run tauri android init

# 打 release APK（仅 arm64-v8a，现代手机通用）
npm run tauri android build -- --target aarch64

# 安装到已连接设备
adb install src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
```

> Android release 签名使用自签名 keystore：`src-tauri/gen/android/app/st-release.keystore`（密码见同目录 `keystore.properties`，已加入 `.gitignore`）。
> **请务必自行备份这两个文件**，丢失后将无法覆盖安装新版本（需卸载重装）。

## CSV 数据集格式要求

### 文件要求

- 仅支持 `.csv` 文件，建议 **UTF-8 编码**
- 第一行为表头，字段顺序不限

### 字段说明

| 字段 | 是否必填 | 说明 |
|---|---|---|
| 题目类型 | **必填** | `single`（单选）/ `multi`（多选）/ `judge`（判断）/ `essay`（简答）/ `fill`（填空） |
| 题干 | **必填** | 题目文本 |
| 选项 | 否 | 单选/多选使用，**英文逗号分隔**（含逗号时用 CSV 引号包裹），如 `"MySQL,Redis,MongoDB"`；简答/填空留空 |
| 正确答案 | **必填** | 单选 `A`；多选 `A,B`；判断 `true`/`false`；填空多答案用英文分号 `;` 分隔；简答填参考答案 |
| 解析 | 否 | 答案解析 |
| 标签 | 否 | 多个标签英文逗号分隔，如 `数据库,第三章` |
| 所属题库集 | 否 | 按名称自动创建/归入题库集；留空则归入导入时选择的默认题库集 |

### 示例（`data.csv`）

```csv
题目类型,题干,选项,正确答案,解析,标签,所属题库集
single,以下哪种数据库属于关系型数据库？,"MySQL,Redis,MongoDB,ElasticSearch",A,"MySQL 是典型的关系型数据库。",数据库,基础题库
multi,以下哪些是面向对象编程的特性？,"封装,继承,多态,泛型",A,B,C,"面向对象三大特性：封装、继承、多态。",编程范式,基础题库
judge,HTTPS 使用对称加密和非对称加密混合的方式进行加密传输。, ,true,,网络协议,进阶题库
fill,在 Linux 系统中，用于查看当前目录下所有文件的命令是____。, ,ls -a,,Linux,基础题库
essay,请简述什么是 RESTful API？, ,RESTful API 是一种基于 HTTP 协议的软件架构风格…,,网络协议,进阶题库
```

### 导入规则

1. 仅支持 `.csv` 文件，其他格式直接拒绝
2. **题干为空、类型非法、答案为空** → 该行跳过，导入完成后展示错误行号与原因
3. **重复题干**（同一题库集内）→ 保留原题，不重复新增，并在结果中提示
4. 单选/多选答案为选项字母（`A`/`A,B`）时，导入时**自动映射为对应选项文本**（保证刷题判分可直接匹配）；答案本就是选项文本则原样保留
5. 判断题答案必须为 `true` / `false`
6. 多选答案兼容分号写法（`A;B;C` 等价于 `A,B,C`）

## 题型与判分规则

| 题型 | 判分规则 |
|---|---|
| 单选 single | 机器判分，所选选项文本与答案完全一致（去除首尾空格） |
| 多选 multi | 机器判分，选项集合**必须完全匹配**，多选、少选、错选均判错 |
| 判断 judge | 机器判分，`true` / `false`（忽略大小写与首尾空格） |
| 填空 fill | 机器判分：去除首尾空格后与标准答案比对，**大小写敏感**，中间空格原样比对；多标准答案用 `;` 分隔，命中任意一个即正确；**允许手动覆写对错**（覆写后以手动结果为准） |
| 简答 essay | 无机器判分，提交后**手动标记对错** |

客观题与填空题答错自动进入错题本；同一题多次做错错题本仅存一条，累计做错次数。

## 数据存储与备份

- 数据库位置（Linux）：`~/.config/com.charm.quiz/st.db`（Windows 为 `%APPDATA%\com.charm.quiz\st.db`）
- 全部数据仅存本机，**卸载 App 会丢失数据**，请定期使用「备份恢复」页导出备份
- 备份包为自定义 `.qpbackup` 格式（JSON），仅用于本 App 恢复，不与 CSV 互通
- 恢复备份会**覆盖本地全部数据**，恢复前有二次确认弹窗；备份文件损坏时恢复失败，原数据保持不变

## 常见问题

### Linux 下窗口白屏

当前用户不在 `render` 组时 WebKitGTK 的 GPU 渲染不可用。应用启动时会自动检测并回退软件渲染；若仍白屏，可手动指定：

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run tauri dev
```

### 直接 `cargo check --target aarch64-linux-android` 报找不到 clang

NDK 工具链需显式指定（`tauri android build` 会自动处理）：

```bash
NDK_BIN="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
CC_aarch64_linux_android="$NDK_BIN/aarch64-linux-android24-clang" \
AR_aarch64_linux_android="$NDK_BIN/llvm-ar" \
cargo check --target aarch64-linux-android
```

### Android 工程模板缺失 tauri.settings.gradle / tauri.build.gradle.kts

tauri-cli 2.6.x 的 `android init` 模板偶发引用旧格式文件。若 `./gradlew` 报找不到这两个文件，删除 `settings.gradle` 中的 `apply from: 'tauri.settings.gradle'` 与 `app/build.gradle.kts` 末尾的 `apply(from = "tauri.build.gradle.kts")` 即可（新模板已由 buildSrc 插件接管 Rust 构建）。

## 许可与说明

本项目为个人自用工具；不含任何网络请求、埋点或上报代码。
