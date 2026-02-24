package dev.vndx.flashbang.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.vndx.flashbang.Core
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.stateIn
import uniffi.mobile.FuzzyStatus
import javax.inject.Inject

@HiltViewModel
class ExploreViewModel @Inject constructor(
    private val core: Core
) : ViewModel() {

    private val _searchQuery = MutableStateFlow("")
    val searchQuery = _searchQuery.asStateFlow()

    @OptIn(FlowPreview::class, ExperimentalCoroutinesApi::class)
    val searchResultIds: StateFlow<List<String>> = _searchQuery
        .debounce(100)
        .flatMapLatest { query ->
            if (query.isEmpty()) {
                flow { emit(emptyList<String>()) }
            } else {
                searchFlow(query)
            }
        }
        .flowOn(Dispatchers.IO)
        .stateIn(
            viewModelScope,
            SharingStarted.WhileSubscribed(5_000),
            emptyList()
        )

    fun onSearchQueryChanged(query: String) {
        _searchQuery.value = query
    }

    private fun searchFlow(query: String): Flow<List<String>> = flow {
        core.core.fuzzyInit(query)

        var run = true
        var lastResults: List<String> = emptyList()

        while (run) {
            val status = core.core.fuzzyTick()
            if (status != FuzzyStatus.STALE) {
                val rawResults = core.core.fuzzyResults()

                val results = rawResults.map { it.data() }

                if (results != lastResults) {
                    emit(results)
                    lastResults = results
                }

                run = status != FuzzyStatus.FINISHED
            }
            if (run) {
                delay(50)
            }
        }
    }
}
