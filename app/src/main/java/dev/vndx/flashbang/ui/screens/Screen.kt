package dev.vndx.flashbang.ui.screens

import android.util.Log
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeContent
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import dev.vndx.flashbang.R
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import androidx.navigation3.runtime.NavBackStack
import androidx.navigation3.runtime.NavKey
import dev.vndx.flashbang.TAG
import dev.vndx.flashbang.ui.Sizes
import dev.vndx.flashbang.ui.TopBar

enum class Tab {
    Study {
        // TODO: Change that should be StudiesScreen
        override fun defaultScreen() = StudiesScreen()
        override fun isDefaultScreen(screen: Screen) = screen is StudiesScreen
        override fun iconResource() = R.drawable.outline_school_32
        override fun selectedIconResource() = R.drawable.baseline_school_32
    },
    Cards {
        override fun defaultScreen() = ExploreScreen()
        override fun isDefaultScreen(screen: Screen) = screen is ExploreScreen
        override fun iconResource() = R.drawable.outline_auto_awesome_mosaic_32
        override fun selectedIconResource() = R.drawable.baseline_auto_awesome_mosaic_32
    },
    Statistics {
        override fun defaultScreen() = DummyScreen(Tab.Statistics)
        override fun isDefaultScreen(screen: Screen) =
            screen is DummyScreen && screen.dummyTab == Tab.Statistics

        override fun iconResource() = R.drawable.outline_insert_chart_32
        override fun selectedIconResource() = R.drawable.baseline_insert_chart_32
    },
    Settings {
        override fun defaultScreen() = SettingsScreen()
        override fun isDefaultScreen(screen: Screen) = screen is SettingsScreen
        override fun iconResource() = R.drawable.outline_settings_32
        override fun selectedIconResource() = R.drawable.baseline_settings_32
    };

    abstract fun defaultScreen(): Screen
    abstract fun iconResource(): Int
    abstract fun selectedIconResource(): Int
    abstract fun isDefaultScreen(screen: Screen): Boolean
}

interface Screen : NavKey {
    companion object {
        fun homeScreen(): Screen = StudiesScreen()
    }

    @Composable
    fun ComposeTopBarAction(onNavigate: (Screen) -> Unit, onBack: (Int?) -> Unit) {

    }

    @Composable
    fun ComposeScaffold(
        onNavigate: (Screen) -> Unit,
        onBack: (Int?) -> Unit,
        backStack: NavBackStack,
    ) {
        Scaffold(
            containerColor = MaterialTheme.colorScheme.background,
            contentWindowInsets = WindowInsets.safeContent,
            topBar = {
                TopBar(
                    "Flashcards",
                    onBack = { backStack.removeLastOrNull() },
                    actions = {
                        ComposeTopBarAction(onNavigate, onBack)
                    })
            },
            bottomBar = {
                if (showTabs()) {
                    val paddingValues = WindowInsets.safeDrawing.asPaddingValues()
                    Column {
                        HorizontalDivider(
                            thickness = Sizes.borderWidth,
                            color = MaterialTheme.colorScheme.surfaceContainer
                        )
                        Row(
                            horizontalArrangement = Arrangement.SpaceEvenly,
                            modifier = Modifier
                                .padding(
                                    bottom = paddingValues.calculateBottomPadding(),
                                    start = 0.dp,
                                    end = 0.dp,
                                    top = 0.dp
                                )
                                .padding(0.dp, Sizes.spacingSmall)
                                .fillMaxWidth(),
                        ) {
                            Tab.entries.forEach { tab ->
                                IconButton(onClick = {
                                    // Trim the stack to the first two screens if needed (this shouldn't ever happen)
                                    if (backStack.size > 2) {
                                        Log.e(
                                            TAG,
                                            "Changing tab with a back stack of size ${backStack.size} (> 2), this shouldn't be possible, expect weird back button behavior."
                                        )
                                        backStack.removeRange(2, backStack.size - 1)
                                    }

                                    val nextScreen = tab.defaultScreen()

                                    // If we aren't on home screen the stack should be
                                    // [HomeScreen] <- [this] <- (top)
                                    if (!isHomeScreen()) {
                                        // Remove top to make space for nextScreen
                                        backStack.removeLastOrNull()
                                    }

                                    // If we're going to the HomeScreen we also need to remove it from the stack
                                    if (nextScreen.isHomeScreen()) {
                                        backStack.removeLastOrNull()
                                    }

                                    // Go there
                                    backStack.add(nextScreen)
                                }) {
                                    Icon(
                                        painter = painterResource(
                                            if (tab() == tab) {
                                                tab.selectedIconResource()
                                            } else {
                                                tab.iconResource()
                                            }
                                        ),
                                        tint = MaterialTheme.colorScheme.onBackground,
                                        contentDescription = null,
                                    )
                                }
                            }
                        }
                    }
                }
            }
        ) { padding ->
            Box(
                Modifier
                    .padding(padding)
                    .background(MaterialTheme.colorScheme.background)
            ) {
                Compose(onNavigate = onNavigate, onBack = onBack)
            }
        }
    }

    @Composable
    fun Compose(onNavigate: (Screen) -> Unit, onBack: (Int?) -> Unit): Unit {
        // If this isn't implemented, shit breaks, I don't know why or how but whatever.
        Log.e(TAG, "How did we get here ?")
        Box(
            modifier = Modifier.fillMaxSize()
        ) {
            Text(
                "This isn't funny !",
                modifier = Modifier.align(Alignment.Center),
                style = MaterialTheme.typography.headlineLarge
            )
        }
    }

    fun tab(): Tab
    fun showTabs(): Boolean = false
    fun isHomeScreen(): Boolean = false
}