package dev.vndx.flashbang.ui

import androidx.datastore.core.DataStore
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.vndx.flashbang.Studies
import dev.vndx.flashbang.World
import dev.vndx.flashbang.data.toProto
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.mobile.Card
import uniffi.mobile.Rating
import uniffi.mobile.Study
import javax.inject.Inject

@HiltViewModel
class StudiesViewModel @Inject constructor(
    private val dataStore: DataStore<Studies>,
    private val world: World,
) : ViewModel() {
    val studiesState = dataStore.data.map {
        world.studySetLastId(it.ids.toULong())
        StudiesState.Success(it)
    } .stateIn(
        viewModelScope, SharingStarted.WhileSubscribed(5000),
        StudiesState.Loading
    )

    private fun edit(transform: Studies.Builder.() -> Unit) {
        viewModelScope.launch {
            dataStore.updateData { studies -> studies.toBuilder().apply(transform).build() }
        }
    }

    private fun editStudy(study: Study, transform: dev.vndx.flashbang.Study.Builder.() -> Unit) {
        edit {
            val id = study.getId().toLong()
            val value = getStudiesOrThrow(id)
                .toBuilder()
                .apply(transform)
                .build()
            putStudies(id, value)
        }
    }

    fun finalizeStudy(study: Study) {
        study.finalize()
        editStudy(study) {
            setFinished(true)
        }
    }

    fun renameStudy(study: Study, name: String) {
        study.rename(name)
        editStudy(study) {
            setName(name)
        }
    }

    fun updateStudy(study: Study, rating: Rating, card: Card) {
        val cardId = card.id()
        study.update(rating, cardId)

        // TODO: update FSRS Memory state:
        //       Update rust side, which will give us new memory state
        //       and feed into proto

        editStudy(study) {
            putReviews(cardId, rating.toProto())
        }
    }
}

sealed interface StudiesState {
    data object Loading : StudiesState

    data class Success(private val inner: Studies) : StudiesState {
        override val studies: Studies get() = inner
    }

    val studies: Studies get() = Studies.getDefaultInstance()
}
