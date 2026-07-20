allprojects {
    repositories {
        google()
        mavenCentral()
    }
    tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile>().configureEach {
        kotlinOptions.jvmTarget = "11"
    }
    tasks.withType<JavaCompile>().configureEach {
        sourceCompatibility = "11"
        targetCompatibility = "11"
    }
}

// Kotlin 2.x 默认会校验 Java/Kotlin JVM 目标一致性，
// 某些第三方插件（如 audioplayers_android）的 Java 目标无法覆盖，
// 此处关闭此校验。
tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile>().configureEach {
    compilerOptions.jvmTargetValidationMode.set(org.jetbrains.kotlin.gradle.dsl.JvmTargetValidationMode.WARNING)
}

val newBuildDir: Directory =
    rootProject.layout.buildDirectory
        .dir("../../build")
        .get()
rootProject.layout.buildDirectory.value(newBuildDir)

subprojects {
    val newSubprojectBuildDir: Directory = newBuildDir.dir(project.name)
    project.layout.buildDirectory.value(newSubprojectBuildDir)
}
subprojects {
    project.evaluationDependsOn(":app")
}

tasks.register<Delete>("clean") {
    delete(rootProject.layout.buildDirectory)
}
