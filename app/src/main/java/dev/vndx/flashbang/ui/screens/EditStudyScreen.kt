package dev.vndx.flashbang.ui.screens

import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedCard
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import dev.vndx.flashbang.localNavSharedTransitionScope
import dev.vndx.flashbang.ui.Sizes
import kotlinx.serialization.Serializable

@Serializable
class EditStudyScreen(val handle: Int, val name: String) : Screen {
    override fun tab() = Tab.Study

    @OptIn(ExperimentalSharedTransitionApi::class)
    @Composable
    override fun Compose(onNavigate: (Screen) -> Unit) {
        Box(
            modifier = Modifier
                .padding(Sizes.spacingMedium)
                .fillMaxSize()
        ) {
            Text(
                name,
                style = MaterialTheme.typography.titleMedium,
            )
        }
    }
}