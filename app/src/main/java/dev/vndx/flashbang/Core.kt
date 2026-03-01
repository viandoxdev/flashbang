package dev.vndx.flashbang

import android.app.Application
import android.os.storage.StorageManager
import android.util.Log
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import uniffi.mobile.CardPage
import uniffi.mobile.CardSource
import uniffi.mobile.Core as FFICore
import uniffi.mobile.FuzzyStatus
import uniffi.mobile.LoadError
import uniffi.mobile.LoadResult
import uniffi.mobile.SourceConfig
import java.io.File
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Singleton wrapper around ffi World
 */
@Singleton
class Core @Inject constructor(private val context: Application) {
    private val storageManager = context.getSystemService(StorageManager::class.java)
    private val cachePath = context.cacheDir.absolutePath
    // SourceConfig doesn't matter, since we'll be updating it at every render anyways,
    // these are just sensible defaults in case something breaks weirdly
    val core = FFICore.new(cachePath)

    private val compilationMutex = Mutex(false)

    suspend fun loadFromGithub(repo: String, branch: String, token: String?): LoadResult {
        val results = core.worldLoadFromGithub(repo, branch, token)

        results.errors.forEach {
             Log.e(TAG, "Error while loading : ${it.error} at ${it.path}")
        }

        // Mark all the newly created directories as groups, so we don't get semi cached packages
        core.worldNewCachedDirectories().forEach {
            storageManager.setCacheBehaviorGroup(File(it), true)
        }

        return results
    }

    fun inspectSource(): String? = core.worldInspectSource()

    suspend fun compileCards(cards: List<CardSource>, config: SourceConfig): List<CardPage> {
        return compilationMutex.withLock {
            core.worldPrepareSource(cards, config)
            val res = core.worldCompile()
            res
        }
    }
}