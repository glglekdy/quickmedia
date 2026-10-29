package com.glglekdy.quickmedia.update

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.PackageInstaller
import android.util.Log
import android.widget.Toast
import androidx.core.content.IntentCompat
import com.glglekdy.quickmedia.App

/** [Updater.install] 세션의 결과. 사용자가 확인해야 하면 안드로이드의 설치 확인 화면을 띄운다. */
class InstallResultReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        when (val status = intent.getIntExtra(PackageInstaller.EXTRA_STATUS, PackageInstaller.STATUS_FAILURE)) {
            PackageInstaller.STATUS_PENDING_USER_ACTION ->
                IntentCompat.getParcelableExtra(intent, Intent.EXTRA_INTENT, Intent::class.java)
                    ?.let { context.startActivity(it.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)) }
            // 성공하면 안드로이드가 앱을 새 버전으로 바꾼다. 확인 화면에서 취소한 것은 알릴 일이 아니다.
            PackageInstaller.STATUS_SUCCESS, PackageInstaller.STATUS_FAILURE_ABORTED -> Unit
            else -> {
                Log.w(App.TAG, "업데이트 설치 실패 ($status): ${intent.getStringExtra(PackageInstaller.EXTRA_STATUS_MESSAGE)}")
                Toast.makeText(context, "업데이트를 설치하지 못했어요.", Toast.LENGTH_LONG).show()
            }
        }
    }
}
