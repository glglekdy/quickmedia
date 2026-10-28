package com.glglekdy.quickmedia

import android.app.Application
import android.util.Log
import com.yausername.ffmpeg.FFmpeg
import com.yausername.youtubedl_android.YoutubeDL
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import java.io.File

/** 다운로드 도구(yt-dlp·파이썬·ffmpeg) 준비 상태. */
sealed interface Tools {
    data object Loading : Tools
    data object Ready : Tools
    data class Failed(val reason: String) : Tools
}

class App : Application() {
    override fun onCreate() {
        super.onCreate()
        // 처음 켤 때와 앱을 업데이트한 뒤에는 파이썬·ffmpeg 를 푸느라 몇 초 걸린다. 화면을 막지 않게 뒤에서 한다.
        scope.launch(Dispatchers.IO) {
            // 받다가 앱이 꺼졌으면 조각 파일이 남는다. 새 프로세스에서는 아직 아무것도 받고 있지 않다.
            File(cacheDir, "dl").deleteRecursively()
            _tools.value = try {
                YoutubeDL.getInstance().init(this@App)
                FFmpeg.getInstance().init(this@App)
                Log.i(TAG, "다운로드 도구 준비됨")
                Tools.Ready
            } catch (e: Exception) {
                Log.e(TAG, "다운로드 도구 준비 실패", e)
                Tools.Failed("다운로드 도구를 준비하지 못했어요. 앱을 다시 켜 보세요.")
            }
        }
    }

    companion object {
        const val TAG = "QuickMedia"

        /** 앱이 살아 있는 동안 도는 작업용 */
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

        private val _tools = MutableStateFlow<Tools>(Tools.Loading)
        val tools: StateFlow<Tools> = _tools.asStateFlow()
    }
}
