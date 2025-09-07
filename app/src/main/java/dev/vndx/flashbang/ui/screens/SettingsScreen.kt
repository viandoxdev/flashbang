package dev.vndx.flashbang.ui.screens

import androidx.activity.compose.LocalActivity
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.ViewModelStoreOwner
import androidx.lifecycle.viewmodel.compose.viewModel
import com.google.protobuf.Internal
import dev.vndx.flashbang.DateFormat
import dev.vndx.flashbang.Preferences
import dev.vndx.flashbang.R
import dev.vndx.flashbang.Theme
import dev.vndx.flashbang.data.dateTimeFormatter
import dev.vndx.flashbang.ui.CardRepositoryDetails
import dev.vndx.flashbang.ui.CardsViewModel
import dev.vndx.flashbang.ui.SettingsAction
import dev.vndx.flashbang.ui.SettingsCategory
import dev.vndx.flashbang.ui.SettingsSelect
import dev.vndx.flashbang.ui.SettingsSwitch
import dev.vndx.flashbang.ui.SettingsTextField
import dev.vndx.flashbang.ui.SettingsViewModel
import kotlinx.serialization.Serializable
import java.time.LocalDateTime

fun Internal.EnumLite.isReal(): Boolean {
    return try {
        this.number > 0
    } catch (_: IllegalArgumentException) {
        false
    }
}

@Composable
fun Theme.name(): String = stringResource(
    when (this) {
        Theme.THEME_DARK -> R.string.theme_dark
        Theme.THEME_LIGHT -> R.string.theme_light
        Theme.THEME_SYSTEM -> R.string.theme_system
        else -> R.string.corrupt
    }
)

@Composable
fun DateFormat.name(): String {
    val format = stringResource(
        when (this) {
            DateFormat.DATE_FORMAT_SLASH_D_M_YY -> R.string.date_format_slash_d_m_yy
            DateFormat.DATE_FORMAT_SLASH_DD_MM_YY -> R.string.date_format_slash_dd_mm_yy
            DateFormat.DATE_FORMAT_SLASH_MM_DD_YY -> R.string.date_format_slash_mm_dd_yy
            DateFormat.DATE_FORMAT_DASH_YYYY_MM_DD -> R.string.date_format_dash_yyyy_mm_dd
            DateFormat.DATE_FORMAT_SPACE_DD_MMM_YYYY -> R.string.date_format_space_dd_mmm_yyyy
            DateFormat.DATE_FORMAT_SPACE_MMM_DD_YYYY -> R.string.date_format_space_mmm_dd_yyyy
            else -> R.string.corrupt
        }
    )

    val example = LocalDateTime.now().format(this.dateTimeFormatter())

    return "$format ($example)"
}

@Serializable
open class SettingsScreen : Screen {
    // Only show tabs on root settings screen
    override fun showTabs(): Boolean = this::class == SettingsScreen::class
    override fun tab(): Tab = Tab.Settings

    open fun LazyListScope.composeItems(
        preferences: Preferences,
        vm: SettingsViewModel,
        onNavigate: (Screen) -> Unit
    ) {
        item {
            SettingsCategory(
                painter = painterResource(R.drawable.outline_palette_32),
                title = stringResource(R.string.appearance),
                subtitle = stringResource(R.string.appearance_subtitle),
                onClick = { onNavigate(SettingsAppearanceScreen()) },
            )
        }
        item {
            SettingsCategory(
                painter = painterResource(R.drawable.outline_package_2_32),
                title = stringResource(R.string.card_repository),
                subtitle = stringResource(R.string.card_repository_subtitle),
                onClick = { onNavigate(SettingsRepositoryScreen()) },
            )
        }
    }

    @Composable
    override fun Compose(onNavigate: (Screen) -> Unit, onBack: () -> Unit) {
        val vm: SettingsViewModel = viewModel()
        val preferencesState by vm.preferences.collectAsState()
        val preferences = preferencesState.preferences

        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
        ) {
            composeItems(preferences, vm, onNavigate)
        }
    }

    @Serializable
    class SettingsAppearanceScreen : SettingsScreen() {
        override fun LazyListScope.composeItems(
            preferences: Preferences,
            vm: SettingsViewModel,
            onNavigate: (Screen) -> Unit
        ) {
            item {
                SettingsSwitch(
                    title = stringResource(R.string.use_dynamic_colors),
                    checked = preferences.useDynamicColors,
                    onCheckedChange = { vm.update { setUseDynamicColors(it) } })
            }
            item {
                SettingsSelect(
                    title = stringResource(R.string.theme),
                    selected = preferences.theme.name(),
                    options = Theme.entries.filter { it.isReal() }.map { Pair(it.name(), it) },
                    subtitle = preferences.theme.name(),
                    onSelect = { (_, theme) -> vm.update { setTheme(theme) } }
                )
            }
            item {
                SettingsSelect(
                    title = stringResource(R.string.date_format),
                    selected = preferences.dateFormat.name(),
                    options = DateFormat.entries.filter { it.isReal() }
                        .map { Pair(it.name(), it) },
                    subtitle = preferences.dateFormat.name(),
                    onSelect = { (_, format) -> vm.update { setDateFormat(format) } }
                )
            }
        }
    }

    @Serializable
    class SettingsRepositoryScreen : SettingsScreen() {
        override fun LazyListScope.composeItems(
            preferences: Preferences,
            vm: SettingsViewModel,
            onNavigate: (Screen) -> Unit
        ) {
            item {
                SettingsTextField(
                    title = stringResource(R.string.repository),
                    subtitle = preferences.repository,
                    value = preferences.repository,
                    onValueChange = { vm.update { setRepository(it) } }
                )
            }
            item {
                SettingsTextField(
                    title = stringResource(R.string.branch),
                    subtitle = preferences.branch,
                    value = preferences.branch,
                    onValueChange = { vm.update { setBranch(it) } }
                )
            }
            item {
                SettingsTextField(
                    title = stringResource(R.string.token),
                    subtitle = preferences.githubToken,
                    value = preferences.githubToken,
                    onValueChange = { vm.update { setGithubToken(it) } }
                )
            }
            item {
                val cardsViewModel: CardsViewModel =
                    viewModel(viewModelStoreOwner = LocalActivity.current as ViewModelStoreOwner)

                SettingsAction(
                    title = stringResource(R.string.reload),
                    onClick = {
                        cardsViewModel.load(
                            CardRepositoryDetails(
                                preferences.repository,
                                preferences.branch,
                                preferences.githubToken
                            )
                        )
                    }
                )
            }
        }
    }
}