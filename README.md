## 初始化结果

### 1. 工程生成

   npm run tauri android init 成功生成 src-tauri/gen/android/ Gradle 工程，使用已安装的 NDK 29.0.13599879。

### 2. 关键配置（与 tauri.conf.json 一致）

|配置项|值|
|-----|---|
|applicationId / namespace|`com.charm.quiz`|
|compileSdk / targetSdk|36|
|minSdk|24|
|ABIs|arm64-v8a / armeabi-v7a / x86 / x86_64|

### 3. 修复了一个 tauri-cli 2.6.2 模板 bug

   tauri android init 生成的工程引用了 3 个未生成的旧格式文件（稳定复现）：

- settings.gradle 引用 tauri.settings.gradle ❌
- app/build.gradle.kts 引用 tauri.build.gradle.kts ❌

   而新模板已改用 buildSrc 里的 Kotlin 插件（RustPlugin.kt/BuildTask.kt）驱动 Rust 构建。修复方式：

- 删除上述两个失效引用
- 补上 tauri.properties（版本号配置，app/build.gradle.kts 需要读取）

   修复后 ./gradlew tasks BUILD SUCCESSFUL。

### 4. 验证结果

- ✅ Gradle 工程配置加载成功
- ✅ Rust Android 目标编译通过（cargo check --target aarch64-linux-android，22s）
  - 直接跑 cargo 会报 failed to find tool "aarch64-linux-android-clang"，这是预期行为——需设置 NDK 工具链环境变量（Tauri 构建命令会自动处理）：
   [bash]
       NDK_BIN="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
       export CC_aarch64_linux_android="$NDK_BIN/aarch64-linux-android24-clang"
       export AR_aarch64_linux_android="$NDK_BIN/llvm-ar"

## 后续构建 APK 命令

   [bash]

# 连接真机/模拟器调试

   `npm run tauri android dev`

# 打 debug APK（仅 arm64，较快）

   `npm run tauri android build -- --debug --target aarch64`

# 打 release APK（需先生成签名密钥）

   `npm run tauri android build -- --target aarch64`
