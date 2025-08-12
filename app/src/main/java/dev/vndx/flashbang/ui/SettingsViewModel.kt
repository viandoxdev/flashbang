package dev.vndx.flashbang.ui

import android.util.Log
import androidx.datastore.core.DataStore
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.vndx.flashbang.Preferences
import dev.vndx.flashbang.TAG
import dev.vndx.flashbang.Theme
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class SettingsViewModel @Inject constructor(
    private val dataStore: DataStore<Preferences>
) : ViewModel() {
    val preferences = dataStore.data.map {
        Log.w(TAG, "Got dataStore read")
        PreferencesState.Success(it)
    } .stateIn(
        viewModelScope, SharingStarted.WhileSubscribed(5000),
        PreferencesState.Loading
    )

    fun update(transform: Preferences.Builder.() -> Unit) {
        viewModelScope.launch {
            dataStore.updateData { prefs -> prefs.toBuilder().apply(transform).build() }
        }
    }
}

sealed interface PreferencesState {
    data object Loading : PreferencesState

    data class Success(private val inner: Preferences) : PreferencesState {
        override val preferences: Preferences get() = inner
    }

    val preferences: Preferences get() = Preferences.getDefaultInstance()
}