package com.vibecode.nexus

import android.Manifest
import android.content.Intent
import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Checklist
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Mic
import androidx.compose.material.icons.filled.Psychology
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.core.content.ContextCompat
import androidx.core.content.PermissionChecker
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.vibecode.nexus.data.ConnectionSettings
import com.vibecode.nexus.data.NexusApiClient
import com.vibecode.nexus.data.UiPreferences
import com.vibecode.nexus.speech.SpeechRecognizerManager
import com.vibecode.nexus.ui.components.NexusFooter
import com.vibecode.nexus.ui.screen.SparkHistoryScreen
import com.vibecode.nexus.ui.screen.SparkScreen
import com.vibecode.nexus.ui.screen.DashboardScreen
import com.vibecode.nexus.ui.screen.PairScreen
import com.vibecode.nexus.ui.screen.ProjectsScreen
import com.vibecode.nexus.ui.screen.SettingsScreen
import com.vibecode.nexus.ui.screen.TasksScreen
import com.vibecode.nexus.ui.screen.WelcomeScreen
import com.vibecode.nexus.ui.theme.NexusTheme
import com.vibecode.nexus.ui.theme.ThemeMode
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch

data class BottomNavItem(
    val route: String,
    val label: String,
    val icon: ImageVector
)

class MainActivity : ComponentActivity() {

    // Sprint Nightvision: Dashboard als Home, Sparks (History) statt Recording-Screen
    // in der Nav. Recording bleibt per Deep-Link/QuickAction vom Dashboard erreichbar.
    private val bottomNavItems = listOf(
        BottomNavItem("dashboard", "Home", Icons.Default.Home),
        BottomNavItem("history", "Sparks", Icons.Default.Psychology),
        BottomNavItem("tasks", "Aufgaben", Icons.Default.Checklist),
        BottomNavItem("projects", "Projekte", Icons.Default.Folder),
        BottomNavItem("settings", "Settings", Icons.Default.Settings),
    )

    // Holds a raw pairing URI that needs to be consumed by the UI layer.
    // Filled from incoming Intents (VIEW action with scheme "nexus"), drained
    // by a LaunchedEffect that calls ConnectionSettings.saveFromQr().
    private val pendingPairingUri = MutableStateFlow<String?>(null)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        consumePairingFromIntent(intent)
        setContent {
            val uiPreferences = remember { UiPreferences(this) }
            var themeMode by remember { mutableStateOf(uiPreferences.themeMode) }
            NexusTheme(themeMode = themeMode) {
                val navController = rememberNavController()
                val connectionSettings = remember { ConnectionSettings(this) }
                val apiClient = remember { NexusApiClient(connectionSettings) }
                val pendingUri by pendingPairingUri.collectAsState()
                val speechManager = remember { SpeechRecognizerManager(this) }

                DisposableEffect(Unit) {
                    speechManager.initialize()
                    onDispose {
                        speechManager.destroy()
                        apiClient.close()
                    }
                }

                var hasAudioPermission by remember {
                    mutableStateOf(
                        ContextCompat.checkSelfPermission(
                            this, Manifest.permission.RECORD_AUDIO
                        ) == PermissionChecker.PERMISSION_GRANTED
                    )
                }

                val audioPermissionLauncher = rememberLauncherForActivityResult(
                    ActivityResultContracts.RequestPermission()
                ) { granted ->
                    hasAudioPermission = granted
                }

                // Periodic health check
                var isConnected by remember { mutableStateOf<Boolean?>(null) }
                var isPaired by remember { mutableStateOf(connectionSettings.isPaired) }

                LaunchedEffect(isPaired) {
                    while (true) {
                        if (isPaired) {
                            isConnected = apiClient.checkHealth()
                        }
                        delay(15_000)
                    }
                }

                // Refresh paired state when navigating
                val navBackStackEntry by navController.currentBackStackEntryAsState()
                LaunchedEffect(navBackStackEntry) {
                    isPaired = connectionSettings.isPaired
                }

                // Consume deep-link pairings
                LaunchedEffect(pendingUri) {
                    val uri = pendingUri ?: return@LaunchedEffect
                    val parsed = connectionSettings.saveFromQr(uri)
                    if (!parsed) {
                        Log.w("MainActivity", "Pairing deep-link could not be parsed: $uri")
                        pendingPairingUri.value = null
                        return@LaunchedEffect
                    }
                    val handshake = apiClient.pairHandshake()
                    if (handshake.isFailure) {
                        Log.w("MainActivity", "Pair handshake failed", handshake.exceptionOrNull())
                        connectionSettings.clear()
                        isPaired = false
                        pendingPairingUri.value = null
                        return@LaunchedEffect
                    }
                    isPaired = true
                    isConnected = apiClient.checkHealth()
                    (application as? NexusApplication)?.let { app ->
                        app.applicationScope.launch { app.runDiagnostics() }
                    }
                    val currentRoute = navController.currentDestination?.route
                    if (currentRoute in listOf("welcome", "pair")) {
                        navController.navigate("dashboard") {
                            popUpTo("welcome") { inclusive = true }
                        }
                    }
                    pendingPairingUri.value = null
                }

                val currentRoute = navBackStackEntry?.destination?.route

                Scaffold(
                    bottomBar = {
                        Column {
                            NavigationBar {
                                bottomNavItems.forEach { item ->
                                    NavigationBarItem(
                                        selected = currentRoute == item.route,
                                        onClick = {
                                            if (currentRoute != item.route) {
                                                navController.navigate(item.route) {
                                                    popUpTo(navController.graph.startDestinationId) {
                                                        saveState = true
                                                    }
                                                    launchSingleTop = true
                                                    restoreState = true
                                                }
                                            }
                                        },
                                        icon = { Icon(item.icon, contentDescription = item.label) },
                                        label = { Text(item.label) }
                                    )
                                }
                            }
                            NexusFooter()
                        }
                    }
                ) { innerPadding ->
                    val startDest = if (connectionSettings.isPaired) "dashboard" else "welcome"
                    NavHost(
                        navController = navController,
                        startDestination = startDest,
                        modifier = Modifier.padding(innerPadding)
                    ) {
                        composable("welcome") {
                            WelcomeScreen(
                                onContinue = { navController.navigate("pair") }
                            )
                        }
                        composable("pair") {
                            PairScreen(
                                connectionSettings = connectionSettings,
                                onPaired = {
                                    isPaired = true
                                    navController.navigate("dashboard") {
                                        popUpTo("welcome") { inclusive = true }
                                    }
                                },
                                onBack = { navController.popBackStack() }
                            )
                        }
                        composable("dashboard") {
                            DashboardScreen(
                                apiClient = apiClient,
                                isPaired = isPaired,
                                onNavigate = { route ->
                                    navController.navigate(route) {
                                        popUpTo(navController.graph.startDestinationId) {
                                            saveState = true
                                        }
                                        launchSingleTop = true
                                        restoreState = true
                                    }
                                },
                                onCreateTask = {
                                    navController.navigate("tasks") {
                                        popUpTo(navController.graph.startDestinationId) {
                                            saveState = true
                                        }
                                        launchSingleTop = true
                                        restoreState = true
                                    }
                                }
                            )
                        }
                        composable("spark") {
                            SparkScreen(
                                speechManager = speechManager,
                                apiClient = apiClient,
                                connectionSettings = connectionSettings,
                                isPaired = isPaired,
                                isConnected = isConnected,
                                hasPermission = hasAudioPermission,
                                onRequestPermission = {
                                    audioPermissionLauncher.launch(Manifest.permission.RECORD_AUDIO)
                                }
                            )
                        }
                        composable("history") {
                            SparkHistoryScreen(apiClient = apiClient)
                        }
                        composable("tasks") {
                            TasksScreen(
                                apiClient = apiClient,
                                isPaired = isPaired
                            )
                        }
                        composable("projects") {
                            ProjectsScreen(
                                apiClient = apiClient,
                                isPaired = isPaired
                            )
                        }
                        composable("settings") {
                            SettingsScreen(
                                connectionSettings = connectionSettings,
                                apiClient = apiClient,
                                themeMode = themeMode,
                                onThemeChange = { mode ->
                                    uiPreferences.themeMode = mode
                                    themeMode = mode
                                },
                                onNavigateBack = {
                                    navController.popBackStack()
                                },
                                onRestartWizard = {
                                    navController.popBackStack(
                                        route = navController.graph.startDestinationRoute ?: "tasks",
                                        inclusive = false,
                                    )
                                }
                            )
                        }
                    }
                }
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        consumePairingFromIntent(intent)
    }

    private fun consumePairingFromIntent(intent: Intent?) {
        val data = intent?.data ?: return
        if (data.scheme == "nexus" && data.host == "pair") {
            pendingPairingUri.value = data.toString()
        }
    }
}
