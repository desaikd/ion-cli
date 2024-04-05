/*
 * This file was generated by the Gradle 'init' task.
 *
 * This generated file contains a sample Java library project to get you started.
 * For more details on building Java & JVM projects, please refer to https://docs.gradle.org/8.6/userguide/building_java_projects.html in the Gradle documentation.
 */

plugins {
    // Apply the java plugin for API and implementation separation.
    java
}

repositories {
    // Use Maven Central for resolving dependencies.
    mavenCentral()
}

dependencies {
    implementation("com.amazon.ion:ion-java:1.11.4")

    // Use JUnit Jupiter for testing.
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
    testImplementation("org.junit.jupiter:junit-jupiter:5.7.1")
}

val ionSchemaSourceCodeDir = "../../schema/"
val generatedIonSchemaModelDir = "${layout.buildDirectory.get()}/generated/java"
sourceSets {
    main {
        java.srcDir(generatedIonSchemaModelDir)
    }
}


tasks {
    val ionCodegen = create<Exec>("ionCodegen") {
        inputs.files(ionSchemaSourceCodeDir)
        outputs.file(generatedIonSchemaModelDir)

        val ionCli = "../../../target/debug/ion"

        commandLine(ionCli)
            .args(
                "beta", "generate",
                "-l", "java",
                "-n", "org.example",
                "-d", ionSchemaSourceCodeDir,
                "-o", generatedIonSchemaModelDir,
            )
            .workingDir(rootProject.projectDir)
    }

    withType<JavaCompile> {
        options.encoding = "UTF-8"
        // The `release` option is not available for the Java 8 compiler, but if we're building with Java 8 we don't
        // need it anyway.
        if (JavaVersion.current() != JavaVersion.VERSION_1_8) {
            options.release.set(8)
        }

        dependsOn(ionCodegen)
    }
}

tasks.named<Test>("test") {
    // Use JUnit Platform for unit tests.
    useJUnitPlatform()
}
