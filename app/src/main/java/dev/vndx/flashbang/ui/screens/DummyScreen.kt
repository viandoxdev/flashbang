package dev.vndx.flashbang.ui.screens

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import kotlinx.serialization.Serializable

@Serializable
class DummyScreen(val dummyTab: Tab) : Screen {
    override fun tab() = dummyTab

    override fun showTabs(): Boolean = true

    @Composable
    override fun Compose(onNavigate: (Screen) -> Unit, onBack: (Int?) -> Unit) {
        Box(
            modifier = Modifier.fillMaxSize()
        ) {
            Text(
                "$dummyTab Tab",
                modifier = Modifier.align(Alignment.Center),
                style = MaterialTheme.typography.headlineLarge
            )
        }
    }
}