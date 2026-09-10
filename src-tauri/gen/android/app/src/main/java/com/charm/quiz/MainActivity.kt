package com.charm.quiz

import android.os.Build
import android.os.Bundle
import android.view.WindowManager
import android.content.res.Configuration

class MainActivity : TauriActivity() {

    // 在 Activity 创建时应用主题
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        applyTheme()
    }

    // 当系统配置改变时（如深色模式切换），自动回调
    override fun onConfigurationChanged(newConfig: Configuration) {
        super.onConfigurationChanged(newConfig)
        applyTheme()
    }

    // 核心方法：根据当前系统主题更新状态栏和导航栏
    private fun applyTheme() {
        // 检测当前系统是否为深色模式
        val isDarkMode = (resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK) == Configuration.UI_MODE_NIGHT_YES

        // 根据主题选择状态栏/导航栏的背景色
        // 这里选择与前端页面背景色一致的颜色，以实现无缝衔接
        val barColor = if (isDarkMode) {
            android.graphics.Color.parseColor("#1a1a1a") // 深色背景
        } else {
            android.graphics.Color.parseColor("#f5f5f5") // 浅色背景（与 CSS 保持一致）
        }

        // 设置状态栏和导航栏颜色 (需要 Android 5.0 以上)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
            window.addFlags(WindowManager.LayoutParams.FLAG_DRAWS_SYSTEM_BAR_BACKGROUNDS)
            window.statusBarColor = barColor
            window.navigationBarColor = barColor

            // 控制状态栏图标颜色（深色模式下图标为白色，浅色模式下为黑色）
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                val decorView = window.decorView
                if (isDarkMode) {
                    // 深色模式：清除浅色状态栏标志，图标变为白色
                    decorView.systemUiVisibility = decorView.systemUiVisibility and android.view.View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR.inv()
                } else {
                    // 浅色模式：添加浅色状态栏标志，图标变为黑色
                    decorView.systemUiVisibility = decorView.systemUiVisibility or android.view.View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR
                }
            }
        }
    }
}