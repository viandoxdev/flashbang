package dev.vndx.flashbang.ui.screens

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import dev.vndx.flashbang.ui.Sizes
import kotlinx.serialization.Serializable

@Serializable
class SourcePreviewScreen(val source: String) : Screen {
    override fun tab(): Tab = Tab.Cards

    @Composable
    override fun Compose(onNavigate: (Screen) -> Unit, onBack: (Int?) -> Unit) {
        LazyColumn (
            modifier = Modifier.fillMaxSize().padding(Sizes.spacingMedium)
        ) {
            item {
                Text(
                    text = source,
                    fontFamily = FontFamily.Monospace,
                )
            }
        }
    }
}