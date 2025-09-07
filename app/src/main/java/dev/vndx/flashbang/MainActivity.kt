package dev.vndx.flashbang

import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.animation.SharedTransitionLayout
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.ProvidableCompositionLocal
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.lifecycle.lifecycleScope
import androidx.navigation3.runtime.entry
import androidx.navigation3.runtime.entryProvider
import androidx.navigation3.runtime.rememberNavBackStack
import androidx.navigation3.ui.NavDisplay
import dagger.hilt.android.AndroidEntryPoint
import dev.vndx.flashbang.ui.CardRepositoryDetails
import dev.vndx.flashbang.ui.CardsViewModel
import dev.vndx.flashbang.ui.FlashbangTheme
import dev.vndx.flashbang.ui.PreferencesState
import dev.vndx.flashbang.ui.SettingsViewModel
import dev.vndx.flashbang.ui.screens.CreateStudyScreen
import dev.vndx.flashbang.ui.screens.DummyScreen
import dev.vndx.flashbang.ui.screens.EditStudyScreen
import dev.vndx.flashbang.ui.screens.ExploreScreen
import dev.vndx.flashbang.ui.screens.Screen
import dev.vndx.flashbang.ui.screens.SelectionScreen
import dev.vndx.flashbang.ui.screens.SelectionViewModel
import dev.vndx.flashbang.ui.screens.SettingsScreen
import dev.vndx.flashbang.ui.screens.StudiesScreen
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.launch
import uniffi.mobile.rustSetupLogger

@OptIn(ExperimentalSharedTransitionApi::class)
val localNavSharedTransitionScope: ProvidableCompositionLocal<SharedTransitionScope> =
    compositionLocalOf {
        throw IllegalStateException(
            "Unexpected access to LocalNavSharedTransitionScope. You must provide a " +
                    "SharedTransitionScope from a call to SharedTransitionLayout() or " +
                    "SharedTransitionScope()"
        )
    }

@AndroidEntryPoint
class MainActivity() : ComponentActivity() {

    val settingsViewModel: SettingsViewModel by viewModels()
    private val cardsViewModel: CardsViewModel by viewModels()

    @OptIn(ExperimentalSharedTransitionApi::class)
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        rustSetupLogger()

        enableEdgeToEdge()

        // Listen on the first successful preference load to load cards from github
        lifecycleScope.launch {
            Log.w(TAG, "In LifecycleScope")
            settingsViewModel.preferences
                .filterIsInstance<PreferencesState.Success>()
                .take(1)
                .map {
                    Log.w(TAG, "Got first successful load")
                    val prefs = it.preferences
                    CardRepositoryDetails(prefs.repository, prefs.branch, prefs.githubToken)
                }.collect {
                    Log.w(TAG, "Sent over to cardTree")
                    cardsViewModel.load(it)
                }
        }

        setContent {
            val preferencesState by settingsViewModel.preferences.collectAsState()

            val preferences = preferencesState.preferences

            val backStack = rememberNavBackStack(StudiesScreen())
            val isSystemInDarkTheme = isSystemInDarkTheme()
            val useDarkTheme =
                when (preferences.theme) {
                    Theme.THEME_DARK -> true
                    Theme.THEME_LIGHT -> false
                    else -> isSystemInDarkTheme
                }
            val composeScreen: @Composable (Screen) -> Unit = { screen ->
                screen.ComposeScaffold(
                    onNavigate = { backStack.add(it) },
                    onBack = { backStack.removeLastOrNull() },
                    useDarkTheme = useDarkTheme,
                    backStack = backStack,
                    onChangeTheme = {
                        settingsViewModel.update {
                            setTheme(
                                if (useDarkTheme) Theme.THEME_LIGHT else Theme.THEME_DARK
                            )
                        }
                    }
                )
            }
            val transitionDuration = 200
            val transform =
                fadeIn(tween(transitionDuration)) togetherWith fadeOut(
                    tween(
                        transitionDuration,
                        delayMillis = transitionDuration * 2 / 3
                    )
                )

            FlashbangTheme(
                useDarkTheme = useDarkTheme,
                useDynamicColors = preferences.useDynamicColors
            ) {
                SharedTransitionLayout {
                    CompositionLocalProvider(localNavSharedTransitionScope provides this) {
                        when (preferencesState) {
                            is PreferencesState.Loading -> {
                                // Don't display anything until settings have been loaded to avoid flashing
                                // a different theme for example

                                Box(
                                    modifier = Modifier.fillMaxSize()
                                ) {
                                    CircularProgressIndicator(
                                        modifier = Modifier.align(Alignment.Center)
                                    )
                                }
                            }

                            is PreferencesState.Success -> {
                                NavDisplay(
                                    modifier = Modifier.fillMaxSize(),
                                    backStack = backStack,
                                    onBack = {
                                        backStack.removeLastOrNull()
                                        if (backStack.isEmpty()) {
                                            finish()
                                        }
                                    },
                                    transitionSpec = {
                                        transform
                                    },
                                    popTransitionSpec = {
                                        transform
                                    },
                                    predictivePopTransitionSpec = {
                                        transform
                                    },
                                    entryProvider = entryProvider {
                                        entry<SelectionScreen> {
                                            composeScreen(it)
                                        }
                                        entry<StudiesScreen> {
                                            composeScreen(it)
                                        }
                                        entry<ExploreScreen> {
                                            composeScreen(it)
                                        }
                                        entry<DummyScreen> {
                                            composeScreen(it)
                                        }
                                        entry<CreateStudyScreen> {
                                            composeScreen(it)
                                        }
                                        entry<EditStudyScreen> {
                                            composeScreen(it)
                                        }
                                        entry<SettingsScreen> {
                                            composeScreen(it)
                                        }
                                        entry<SettingsScreen.SettingsAppearanceScreen> {
                                            composeScreen(it)
                                        }
                                        entry<SettingsScreen.SettingsRepositoryScreen> {
                                            composeScreen(it)
                                        }
                                    }
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}