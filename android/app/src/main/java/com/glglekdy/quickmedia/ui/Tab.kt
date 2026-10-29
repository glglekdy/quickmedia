package com.glglekdy.quickmedia.ui

import androidx.annotation.DrawableRes
import com.glglekdy.quickmedia.R

enum class Tab(val label: String, @DrawableRes val icon: Int) {
    RECEIVE("받기", R.drawable.ic_download),
    LIST("목록", R.drawable.ic_list),
    SETTINGS("설정", R.drawable.ic_settings),
}
