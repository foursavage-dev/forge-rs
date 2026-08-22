plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "1.9.22"
    id("org.jetbrains.intellij") version "1.17.2"
}

group = "com.fish"
version = "0.4.0"

repositories {
    mavenCentral()
}

intellij {
    version.set("2023.3.4")
    type.set("IC")
    plugins.set(listOf("com.intellij.platform.lsp"))
}

tasks {
    withType<JavaCompile> {
        sourceCompatibility = "17"
        targetCompatibility = "17"
    }
    withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile> {
        kotlinOptions.jvmTarget = "17"
    }
    patchPluginXml {
        sinceBuild.set("233")
        untilBuild.set("243.*")
    }
}
