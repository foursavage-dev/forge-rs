package com.fish.plugin

import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.execution.process.OSProcessHandler
import com.intellij.execution.ui.ConsoleViewContentType
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.project.Project
import java.nio.charset.StandardCharsets

class FishRunTaskAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        executeFishCommand(project, "build")
    }

    fun executeFishCommand(project: Project, command: String) {
        val basePath = project.basePath ?: return
        val commandLine = GeneralCommandLine("fish", command)
            .withWorkDirectory(basePath)
            .withCharset(StandardCharsets.UTF_8)

        try {
            val processHandler = OSProcessHandler(commandLine)
            processHandler.startNotify()
        } catch (_: Exception) {
        }
    }
}
