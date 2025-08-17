package dev.vndx.flashbang

import android.app.Application
import android.os.storage.StorageManager
import android.util.Log
import uniffi.mobile.Card
import uniffi.mobile.World as FFIWorld
import uniffi.mobile.FuzzyStatus
import uniffi.mobile.LoadError
import uniffi.mobile.SourceConfig
import uniffi.mobile.Tag
import java.io.File
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Singleton wrapper around ffi World
 */
@Singleton
class World @Inject constructor(private val context: Application) {
    private val storageManager = context.getSystemService(StorageManager::class.java)
    private val cachePath = context.cacheDir.absolutePath
    // SourceConfig doesn't matter, since we'll be updating it at every render anyways,
    // these are just sensible defaults in case something breaks weirdly
    val world = FFIWorld.empty(cachePath, SourceConfig(400u, 12u))

    fun loadFromGithub(repo: String, branch: String, token: String?): List<LoadError> {
        val errors = world.loadFromGithub(repo, branch, token).onEach {
            Log.e(TAG, "Error while loading : ${it.error} at ${it.path}")
        }

        // Mark all the newly created directories as groups, so we don't get semi cached packages
        world.newCachedDirectories().forEach {
            storageManager.setCacheBehaviorGroup(File(it), true)
        }

        return errors
    }

    fun studySetLastId(ids: ULong) = world.studySetLastId(ids)
    fun cards(): List<Card> = world.cards()
    fun roots(): List<Tag> = world.roots()

    fun fuzzyInit(pattern: String) = world.fuzzyInit(pattern)
    fun fuzzyTick(): FuzzyStatus = world.fuzzyTick()
    fun fuzzyResults(): List<Card> = world.fuzzyResults()
}