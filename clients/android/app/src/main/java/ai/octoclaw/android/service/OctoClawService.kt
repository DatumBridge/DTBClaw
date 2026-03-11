package ai.octoclaw.android.service

import android.app.Notification
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.os.Binder
import android.os.IBinder
import androidx.core.app.NotificationCompat
import ai.octoclaw.android.MainActivity
import ai.octoclaw.android.OctoClawApp
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

/**
 * Foreground service that keeps OctoClaw running in the background.
 *
 * This service:
 * - Runs the OctoClaw Rust binary via JNI
 * - Maintains a persistent notification
 * - Handles incoming messages/events
 * - Survives app backgrounding (within Android limits)
 */
class OctoClawService : Service() {

    private val binder = LocalBinder()
    private val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())

    private val _status = MutableStateFlow(Status.Stopped)
    val status: StateFlow<Status> = _status

    private val _lastMessage = MutableStateFlow<String?>(null)
    val lastMessage: StateFlow<String?> = _lastMessage

    inner class LocalBinder : Binder() {
        fun getService(): OctoClawService = this@OctoClawService
    }

    override fun onBind(intent: Intent): IBinder = binder

    override fun onCreate() {
        super.onCreate()
        startForeground(NOTIFICATION_ID, createNotification())
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_START -> startAgent()
            ACTION_STOP -> stopAgent()
            ACTION_SEND -> intent.getStringExtra(EXTRA_MESSAGE)?.let { sendMessage(it) }
        }
        return START_STICKY
    }

    override fun onDestroy() {
        scope.cancel()
        super.onDestroy()
    }

    private fun startAgent() {
        if (_status.value == Status.Running) return

        _status.value = Status.Starting

        scope.launch {
            try {
                // TODO: Initialize and start OctoClaw native library
                // OctoClawBridge.start(configPath)

                _status.value = Status.Running

                // TODO: Start message loop
                // while (isActive) {
                //     val message = OctoClawBridge.pollMessage()
                //     message?.let { _lastMessage.value = it }
                // }
            } catch (e: Exception) {
                _status.value = Status.Error(e.message ?: "Unknown error")
            }
        }
    }

    private fun stopAgent() {
        scope.launch {
            // TODO: OctoClawBridge.stop()
            _status.value = Status.Stopped
        }
    }

    private fun sendMessage(message: String) {
        scope.launch {
            // TODO: OctoClawBridge.sendMessage(message)
        }
    }

    private fun createNotification(): Notification {
        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_IMMUTABLE
        )

        return NotificationCompat.Builder(this, OctoClawApp.CHANNEL_ID)
            .setContentTitle("OctoClaw is running")
            .setContentText("Your AI assistant is active")
            .setSmallIcon(android.R.drawable.ic_menu_manage) // TODO: Replace with custom icon
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .setSilent(true)
            .build()
    }

    companion object {
        private const val NOTIFICATION_ID = 1001
        const val ACTION_START = "ai.octoclaw.action.START"
        const val ACTION_STOP = "ai.octoclaw.action.STOP"
        const val ACTION_SEND = "ai.octoclaw.action.SEND"
        const val EXTRA_MESSAGE = "message"
    }

    sealed class Status {
        object Stopped : Status()
        object Starting : Status()
        object Running : Status()
        data class Error(val message: String) : Status()
    }
}
