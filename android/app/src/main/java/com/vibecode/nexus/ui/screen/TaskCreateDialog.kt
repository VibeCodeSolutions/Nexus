package com.vibecode.nexus.ui.screen

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExposedDropdownMenuBox
import androidx.compose.material3.ExposedDropdownMenuDefaults
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MenuAnchorType
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.vibecode.nexus.data.model.ProjectResponse

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TaskCreateDialog(
    projects: List<ProjectResponse>,
    onDismiss: () -> Unit,
    onCreate: (title: String, projectId: String?, priority: String) -> Unit
) {
    var title by remember { mutableStateOf("") }
    var selectedProject by remember { mutableStateOf<ProjectResponse?>(null) }
    var selectedPriority by remember { mutableStateOf("medium") }
    var projectDropdownExpanded by remember { mutableStateOf(false) }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Neuer Task") },
        text = {
            Column {
                OutlinedTextField(
                    value = title,
                    onValueChange = { title = it },
                    label = { Text("Titel") },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true
                )

                Spacer(Modifier.height(16.dp))

                // Project dropdown
                ExposedDropdownMenuBox(
                    expanded = projectDropdownExpanded,
                    onExpandedChange = { projectDropdownExpanded = it }
                ) {
                    OutlinedTextField(
                        value = selectedProject?.name ?: "Kein Projekt",
                        onValueChange = {},
                        readOnly = true,
                        label = { Text("Projekt") },
                        trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded = projectDropdownExpanded) },
                        modifier = Modifier
                            .fillMaxWidth()
                            .menuAnchor(MenuAnchorType.PrimaryNotEditable)
                    )
                    ExposedDropdownMenu(
                        expanded = projectDropdownExpanded,
                        onDismissRequest = { projectDropdownExpanded = false }
                    ) {
                        DropdownMenuItem(
                            text = { Text("Kein Projekt") },
                            onClick = {
                                selectedProject = null
                                projectDropdownExpanded = false
                            }
                        )
                        projects.forEach { project ->
                            DropdownMenuItem(
                                text = { Text(project.name) },
                                onClick = {
                                    selectedProject = project
                                    projectDropdownExpanded = false
                                }
                            )
                        }
                    }
                }

                Spacer(Modifier.height(16.dp))

                // Priority chips
                Text("Priorität", modifier = Modifier.padding(bottom = 4.dp))
                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    listOf("low" to "Niedrig", "medium" to "Mittel", "high" to "Hoch").forEach { (value, label) ->
                        FilterChip(
                            selected = selectedPriority == value,
                            onClick = { selectedPriority = value },
                            label = { Text(label) }
                        )
                    }
                }
            }
        },
        confirmButton = {
            TextButton(
                onClick = { onCreate(title, selectedProject?.id, selectedPriority) },
                enabled = title.isNotBlank()
            ) {
                Text("Erstellen")
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("Abbrechen")
            }
        }
    )
}
