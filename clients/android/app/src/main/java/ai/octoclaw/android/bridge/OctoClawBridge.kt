package ai.octoclaw.android.bridge

/**
 * JNI bridge to OctoClaw Rust library.
 *
 * This class will be replaced by UniFFI-generated bindings.
 * For now, it provides stub implementations.
 *
 * Native library: liboctoclaw.so
 * Build command: cargo ndk -t arm64-v8a -o app/src/main/jniLibs build --release
 */
object OctoClawBridge {

    private var initialized = false

    /**
     * Initialize the OctoClaw runtime.
     * Must be called before any other methods.
     */
    fun initialize(dataDir: String): Result<Unit> {
        return runCatching {
            // TODO: Load native library
            // System.loadLibrary("octoclaw")
            // nativeInit(dataDir)
            initialized = true
        }
    }

    /**
     * Start the OctoClaw gateway.
     * @param configPath Path to octoclaw.toml config file
     */
    fun start(configPath: String): Result<Unit> {
        check(initialized) { "OctoClawBridge not initialized" }
        return runCatching {
            // TODO: nativeStart(configPath)
        }
    }

    /**
     * Stop the OctoClaw gateway.
     */
    fun stop(): Result<Unit> {
        return runCatching {
            // TODO: nativeStop()
        }
    }

    /**
     * Send a message to the agent.
     */
    fun sendMessage(message: String): Result<Unit> {
        check(initialized) { "OctoClawBridge not initialized" }
        return runCatching {
            // TODO: nativeSendMessage(message)
        }
    }

    /**
     * Poll for the next message from the agent.
     * Returns null if no message available.
     */
    fun pollMessage(): String? {
        if (!initialized) return null
        // TODO: return nativePollMessage()
        return null
    }

    /**
     * Get current agent status.
     */
    fun getStatus(): AgentStatus {
        if (!initialized) return AgentStatus.Stopped
        // TODO: return nativeGetStatus()
        return AgentStatus.Stopped
    }

    /**
     * Check if the native library is loaded.
     */
    fun isLoaded(): Boolean = initialized

    // Native method declarations (to be implemented)
    // private external fun nativeInit(dataDir: String)
    // private external fun nativeStart(configPath: String)
    // private external fun nativeStop()
    // private external fun nativeSendMessage(message: String)
    // private external fun nativePollMessage(): String?
    // private external fun nativeGetStatus(): Int
}

enum class AgentStatus {
    Stopped,
    Starting,
    Running,
    Thinking,
    Error
}

/**
 * Configuration for OctoClaw.
 */
data class OctoClawConfig(
    val provider: String = "anthropic",
    val model: String = "claude-sonnet-4-5",
    val apiKey: String = "",
    val systemPrompt: String? = null,
    val maxTokens: Int = 4096,
    val temperature: Double = 0.7
)
