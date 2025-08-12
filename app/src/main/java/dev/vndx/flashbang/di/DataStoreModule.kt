package dev.vndx.flashbang.di

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.dataStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import dev.vndx.flashbang.Preferences
import dev.vndx.flashbang.data.PreferencesSerializer
import javax.inject.Singleton

private val preferencesFileName = "preferences.pb"

private val Context.preferencesStore: DataStore<Preferences> by dataStore(
    preferencesFileName,
    serializer = PreferencesSerializer
)

@Module
@InstallIn(SingletonComponent::class)
object DataStoreModule {
    @Provides
    @Singleton
    fun providePreferencesDataStore(@ApplicationContext context: Context): DataStore<Preferences> = context.preferencesStore
}