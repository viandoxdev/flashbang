package dev.vndx.flashbang.ui.screens

import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.vndx.flashbang.ui.Sizes
import dev.vndx.flashbang.ui.StudiesViewModel
import kotlinx.serialization.Serializable

@Serializable
class EditStudyScreen(val id: Long) : Screen {
    override fun tab() = Tab.Study

    @OptIn(ExperimentalSharedTransitionApi::class)
    @Composable
    override fun Compose(onNavigate: (Screen) -> Unit, onBack: () -> Unit) {
        val viewModel: StudiesViewModel = viewModel()
        val state by viewModel.studiesState.collectAsState()
        val study = state.studies.getValue(id)

        Box(
            modifier = Modifier
                .padding(Sizes.spacingMedium)
                .fillMaxSize()
        ) {
            Text(
                study.name,
                style = MaterialTheme.typography.titleMedium,
            )
        }
    }
}