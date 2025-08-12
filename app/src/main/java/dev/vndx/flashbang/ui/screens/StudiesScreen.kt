package dev.vndx.flashbang.ui.screens

import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ElevatedCard
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedCard
import androidx.compose.material3.SuggestionChip
import androidx.compose.material3.Surface
import androidx.compose.material3.SwipeToDismissBox
import androidx.compose.material3.SwipeToDismissBoxDefaults
import androidx.compose.material3.SwipeToDismissBoxState
import androidx.compose.material3.SwipeToDismissBoxValue
import androidx.compose.material3.Text
import androidx.compose.material3.rememberSwipeToDismissBoxState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import dev.vndx.flashbang.R
import dev.vndx.flashbang.ui.Sizes
import dev.vndx.flashbang.ui.Study
import kotlinx.serialization.Serializable
import java.time.LocalDate

@Serializable
class StudiesScreen() : Screen {
    override fun tab(): Tab = Tab.Study
    override fun showTabs(): Boolean = true
    override fun isHomeScreen(): Boolean = true

    @Composable
    override fun Compose(onNavigate: (Screen) -> Unit) {
        LazyColumn(
            modifier = Modifier.Companion
                .fillMaxSize()
                .padding(Sizes.spacingMedium, 0.dp),
            verticalArrangement = Arrangement.spacedBy(Sizes.spacingMedium)
        ) {
            items(3) { index ->
                Study(
                    name = "Scheduled",
                    cards = 55,
                    handle = index,
                    description = stringResource(
                        R.string.selected,
                        "Lorem ipsum dolor sit amet I don't know the rest of that sentence but I need more words."
                    ),
                    date = LocalDate.now(),
                    progress = 0.5f,
                    onEdit = { onNavigate(EditStudyScreen(index, "Scheduled")) }
                )
            }
        }
    }
}