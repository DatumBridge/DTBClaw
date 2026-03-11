package ai.octoclaw.android

import android.app.Application
import android.app.NotificationChannel
import android.app.NotificationManager
import android.os.Build
import androidx.work.Configuration
import androidx.work.WorkManager
import ai.octoclaw.android.data.SettingsRepository
import ai.octoclaw.android.worker.HeartbeatWorker
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch

class OctoClawApp : Application(), Configuration.Provider {

    companion object {
        const val CHANNEL_ID = "octoclaw_service"
        const val CHANNEL_NAME = "OctoClaw Agent"
        const val AGENT_CHANNEL_ID = "octoclaw_agent"
        const val AGENT_CHANNEL_NAME = "Agent Messages"

        // Singleton instance for easy access
        lateinit var instance: OctoClawApp
            private set
    }

    // Application scope for coroutines
    private val applicationScope = CoroutineScope(SupervisorJob() + Dispatchers.Main)

    // Lazy initialized repositories
    val settingsRepository by lazy { SettingsRepository(this) }

    override fun onCreate() {
        super.onCreate()
        instance = this

        createNotificationChannels()
        initializeWorkManager()

        // Schedule heartbeat if auto-start is enabled
        applicationScope.launch {
            val settings = settingsRepository.settings.first()
            if (settings.autoStart && settings.isConfigured()) {
                HeartbeatWorker.scheduleHeartbeat(
                    this@OctoClawApp,
                    settings.heartbeatIntervalMinutes.toLong()
                )
            }
        }

        // Listen for settings changes and update heartbeat schedule
        applicationScope.launch {
            settingsRepository.settings
                .map { Triple(it.autoStart, it.isConfigured(), it.heartbeatIntervalMinutes) }
                .distinctUntilChanged()
                .collect { (autoStart, isConfigured, intervalMinutes) ->
                    if (autoStart && isConfigured) {
                        HeartbeatWorker.scheduleHeartbeat(this@OctoClawApp, intervalMinutes.toLong())
                    } else {
                        HeartbeatWorker.cancelHeartbeat(this@OctoClawApp)
                    }
                }
        }

        // TODO: Initialize native library
        // System.loadLibrary("octoclaw_android")
    }

    private fun createNotificationChannels() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val manager = getSystemService(NotificationManager::class.java)

            // Service channel (foreground service - low priority, silent)
            val serviceChannel = NotificationChannel(
                CHANNEL_ID,
                CHANNEL_NAME,
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "OctoClaw background service notification"
                setShowBadge(false)
                enableVibration(false)
                setSound(null, null)
            }

            // Agent messages channel (high priority for important messages)
            val agentChannel = NotificationChannel(
                AGENT_CHANNEL_ID,
                AGENT_CHANNEL_NAME,
                NotificationManager.IMPORTANCE_HIGH
            ).apply {
                description = "Messages and alerts from your AI agent"
                enableVibration(true)
                setShowBadge(true)
            }

            manager.createNotificationChannel(serviceChannel)
            manager.createNotificationChannel(agentChannel)
        }
    }

    private fun initializeWorkManager() {
        // WorkManager is initialized via Configuration.Provider
        // This ensures it's ready before any work is scheduled
    }

    // Configuration.Provider implementation for custom WorkManager setup
    override val workManagerConfiguration: Configuration
        get() = Configuration.Builder()
            .setMinimumLoggingLevel(android.util.Log.INFO)
            .build()
}
