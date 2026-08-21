package com.fish.plugin

import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.components.JBLabel
import com.intellij.ui.components.JBPanel
import com.intellij.ui.content.ContentFactory
import java.awt.BorderLayout
import javax.swing.JButton

class FishToolWindowFactory : ToolWindowFactory {
    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        val panel = JBPanel<JBPanel<*>>(BorderLayout())
        val titleLabel = JBLabel("Fish Build Orchestrator & DAG Visualizer")
        val buildButton = JButton("Run Build").apply {
            addActionListener {
                val action = FishRunTaskAction()
                action.executeFishCommand(project, "build")
            }
        }

        panel.add(titleLabel, BorderLayout.NORTH)
        panel.add(buildButton, BorderLayout.CENTER)

        val contentFactory = ContentFactory.getInstance()
        val content = contentFactory.createContent(panel, "DAG Graph", false)
        toolWindow.contentManager.addContent(content)
    }
}
