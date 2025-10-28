# Jenkins Integration for PMAT Mutation Testing

Complete guide for integrating PMAT mutation testing into Jenkins CI/CD pipelines.

## Table of Contents

- [Quick Start](#quick-start)
- [Basic Setup](#basic-setup)
- [Multi-Language Support](#multi-language-support)
- [Quality Gates](#quality-gates)
- [Advanced Patterns](#advanced-patterns)
- [Blue Ocean Integration](#blue-ocean-integration)
- [Artifacts and Reports](#artifacts-and-reports)
- [Pipeline Libraries](#pipeline-libraries)
- [Distributed Builds](#distributed-builds)
- [Troubleshooting](#troubleshooting)
- [Best Practices](#best-practices)
- [Complete Production Example](#complete-production-example)

## Quick Start

Create `Jenkinsfile`:

```groovy
pipeline {
    agent any

    stages {
        stage('Install PMAT') {
            steps {
                sh 'cargo install pmat'
            }
        }

        stage('Mutation Testing') {
            steps {
                sh 'pmat mutate --target src/ --failures-only'
            }
        }
    }
}
```

## Basic Setup

### Declarative Pipeline

`Jenkinsfile`:

```groovy
pipeline {
    agent {
        docker {
            image 'rust:latest'
            args '-v cargo-cache:/usr/local/cargo'
        }
    }

    options {
        timeout(time: 1, unit: 'HOURS')
        timestamps()
        buildDiscarder(logRotator(numToKeepStr: '30'))
    }

    stages {
        stage('Install PMAT') {
            steps {
                sh '''
                    if [ ! -f /usr/local/cargo/bin/pmat ]; then
                        cargo install pmat
                    else
                        echo "Using cached PMAT binary"
                        pmat --version
                    fi
                '''
            }
        }

        stage('Build') {
            steps {
                sh 'cargo build --release'
            }
        }

        stage('Unit Tests') {
            steps {
                sh 'cargo test --all-features'
            }
        }

        stage('Mutation Testing') {
            steps {
                sh 'pmat mutate --target src/ --output-format json > mutation-results.json'
                sh 'pmat mutate --target src/ --output-format markdown > mutation-report.md'
            }
        }
    }

    post {
        always {
            archiveArtifacts artifacts: 'mutation-results.json,mutation-report.md', fingerprint: true
        }
        success {
            echo '✅ Mutation testing passed!'
        }
        failure {
            echo '❌ Mutation testing failed!'
        }
    }
}
```

### Scripted Pipeline

```groovy
node {
    def pmatInstalled = false

    try {
        stage('Checkout') {
            checkout scm
        }

        stage('Install PMAT') {
            sh '''
                if [ ! -f /usr/local/cargo/bin/pmat ]; then
                    cargo install pmat
                else
                    echo "Using cached PMAT"
                    pmat --version
                fi
            '''
            pmatInstalled = true
        }

        stage('Mutation Testing') {
            if (pmatInstalled) {
                sh 'pmat mutate --target src/ --failures-only'
            }
        }

    } catch (Exception e) {
        currentBuild.result = 'FAILURE'
        throw e
    } finally {
        archiveArtifacts artifacts: '*.json,*.md', allowEmptyArchive: true
    }
}
```

## Multi-Language Support

### Rust Projects

```groovy
pipeline {
    agent {
        docker {
            image 'rust:1.75.0'
            args '-v cargo-cache:/usr/local/cargo -v ${WORKSPACE}:/workspace'
        }
    }

    environment {
        CARGO_HOME = '/usr/local/cargo'
        MUTATION_THRESHOLD = '85'
    }

    stages {
        stage('Setup') {
            steps {
                sh '''
                    if [ ! -f ${CARGO_HOME}/bin/pmat ]; then
                        cargo install pmat
                    fi
                '''
            }
        }

        stage('Build') {
            steps {
                sh 'cargo build --release'
            }
        }

        stage('Test') {
            steps {
                sh 'cargo test'
            }
        }

        stage('Mutation Testing') {
            steps {
                script {
                    sh '''
                        pmat mutate --target src/ --threshold ${MUTATION_THRESHOLD} \
                            --output-format json > mutation-results.json
                    '''

                    def mutationResults = readJSON file: 'mutation-results.json'
                    def score = mutationResults.mutation_score

                    echo "Mutation Score: ${score}%"

                    if (score < env.MUTATION_THRESHOLD.toFloat()) {
                        error("Mutation score ${score}% below threshold ${env.MUTATION_THRESHOLD}%")
                    }
                }
            }
        }
    }

    post {
        always {
            archiveArtifacts artifacts: 'mutation-results.json', fingerprint: true
        }
    }
}
```

### Python Projects

```groovy
pipeline {
    agent {
        docker {
            image 'python:3.11'
            args '-v cargo-cache:/root/.cargo'
        }
    }

    environment {
        MUTATION_THRESHOLD = '80'
    }

    stages {
        stage('Install Rust and PMAT') {
            steps {
                sh '''
                    apt-get update && apt-get install -y curl build-essential
                    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                    . $HOME/.cargo/env
                    cargo install pmat
                '''
            }
        }

        stage('Install Python Dependencies') {
            steps {
                sh '''
                    python -m venv venv
                    . venv/bin/activate
                    pip install -r requirements.txt
                '''
            }
        }

        stage('Run Tests') {
            steps {
                sh '''
                    . venv/bin/activate
                    pytest --cov=src tests/
                '''
            }
        }

        stage('Mutation Testing') {
            steps {
                sh '''
                    . $HOME/.cargo/env
                    pmat mutate --target src/ --threshold ${MUTATION_THRESHOLD} \
                        --output-format markdown > mutation-report.md
                '''
            }
        }
    }

    post {
        always {
            archiveArtifacts artifacts: 'mutation-report.md', fingerprint: true
        }
    }
}
```

### TypeScript/JavaScript Projects

```groovy
pipeline {
    agent {
        docker {
            image 'node:18'
            args '-v cargo-cache:/root/.cargo -v node-modules-cache:/workspace/node_modules'
        }
    }

    environment {
        MUTATION_THRESHOLD = '85'
    }

    stages {
        stage('Install Rust and PMAT') {
            steps {
                sh '''
                    apt-get update && apt-get install -y curl build-essential
                    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                    . $HOME/.cargo/env
                    cargo install pmat
                '''
            }
        }

        stage('Install Node Dependencies') {
            steps {
                sh 'npm ci'
            }
        }

        stage('Run Jest Tests') {
            steps {
                sh 'npm test'
            }
        }

        stage('Mutation Testing') {
            steps {
                sh '''
                    . $HOME/.cargo/env
                    pmat mutate --target src/ --threshold ${MUTATION_THRESHOLD} \
                        --output-format json > mutation-results.json
                '''
            }
        }
    }

    post {
        always {
            archiveArtifacts artifacts: 'mutation-results.json,coverage/', fingerprint: true
        }
    }
}
```

### Multi-Language Matrix

```groovy
pipeline {
    agent none

    stages {
        stage('Mutation Testing Matrix') {
            matrix {
                axes {
                    axis {
                        name 'LANGUAGE'
                        values 'rust', 'python', 'typescript'
                    }
                }

                agent {
                    docker {
                        image "${getDockerImage(LANGUAGE)}"
                    }
                }

                stages {
                    stage('Setup') {
                        steps {
                            script {
                                setupLanguage(env.LANGUAGE)
                            }
                        }
                    }

                    stage('Test') {
                        steps {
                            script {
                                runTests(env.LANGUAGE)
                            }
                        }
                    }

                    stage('Mutation Test') {
                        steps {
                            script {
                                runMutationTests(env.LANGUAGE)
                            }
                        }
                    }
                }
            }
        }
    }
}

def getDockerImage(language) {
    switch(language) {
        case 'rust':
            return 'rust:latest'
        case 'python':
            return 'python:3.11'
        case 'typescript':
            return 'node:18'
        default:
            error("Unknown language: ${language}")
    }
}

def setupLanguage(language) {
    sh '''
        apt-get update && apt-get install -y curl build-essential
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        . $HOME/.cargo/env
        cargo install pmat
    '''

    switch(language) {
        case 'rust':
            sh 'cargo build --release'
            break
        case 'python':
            sh 'python -m venv venv && . venv/bin/activate && pip install -r requirements.txt'
            break
        case 'typescript':
            sh 'npm ci'
            break
    }
}

def runTests(language) {
    switch(language) {
        case 'rust':
            sh 'cargo test'
            break
        case 'python':
            sh '. venv/bin/activate && pytest'
            break
        case 'typescript':
            sh 'npm test'
            break
    }
}

def runMutationTests(language) {
    def target = language == 'rust' ? 'src/' : "${language}/src/"
    sh """
        . \$HOME/.cargo/env
        pmat mutate --target ${target} --output-format json > mutation-results-${language}.json
    """
}
```

## Quality Gates

### Fail Build on Low Score

```groovy
pipeline {
    agent any

    environment {
        MUTATION_THRESHOLD = '85'
    }

    stages {
        stage('Mutation Testing') {
            steps {
                script {
                    sh 'pmat mutate --target src/ --output-format json > mutation-results.json'

                    def results = readJSON file: 'mutation-results.json'
                    def score = results.mutation_score
                    def killed = results.killed
                    def survived = results.survived

                    echo "📊 Mutation Testing Results:"
                    echo "  Score: ${score}%"
                    echo "  Killed: ${killed}"
                    echo "  Survived: ${survived}"

                    if (score < env.MUTATION_THRESHOLD.toFloat()) {
                        currentBuild.result = 'UNSTABLE'
                        error("❌ Mutation score ${score}% below threshold ${env.MUTATION_THRESHOLD}%")
                    } else {
                        echo "✅ Mutation score ${score}% meets quality gate"
                    }
                }
            }
        }
    }
}
```

### Multi-Level Thresholds

```groovy
pipeline {
    agent any

    stages {
        stage('Mutation Testing with Thresholds') {
            steps {
                script {
                    sh 'pmat mutate --target src/ --output-format json > mutation-results.json'

                    def results = readJSON file: 'mutation-results.json'
                    def score = results.mutation_score

                    echo "Mutation Score: ${score}%"

                    // Critical threshold: Hard fail
                    if (score < 80.0) {
                        currentBuild.result = 'FAILURE'
                        error("🔴 CRITICAL: Score ${score}% below 80% (hard fail)")
                    }

                    // Warning threshold: Unstable
                    if (score < 85.0) {
                        currentBuild.result = 'UNSTABLE'
                        echo "🟡 WARNING: Score ${score}% below recommended 85%"
                    }

                    // Excellent threshold
                    if (score >= 90.0) {
                        echo "🟢 EXCELLENT: Score ${score}% above 90%"
                    }
                }
            }
        }
    }
}
```

### Per-Module Thresholds

```groovy
pipeline {
    agent any

    stages {
        stage('Mutation Testing - Authentication') {
            steps {
                script {
                    sh 'pmat mutate --target src/auth/ --threshold 90 --output-format json > mutation-auth.json'
                    verifyMutationScore('mutation-auth.json', 90)
                }
            }
        }

        stage('Mutation Testing - API') {
            steps {
                script {
                    sh 'pmat mutate --target src/api/ --threshold 85 --output-format json > mutation-api.json'
                    verifyMutationScore('mutation-api.json', 85)
                }
            }
        }

        stage('Mutation Testing - Database') {
            steps {
                script {
                    sh 'pmat mutate --target src/database/ --threshold 80 --output-format json > mutation-database.json'
                    verifyMutationScore('mutation-database.json', 80)
                }
            }
        }
    }
}

def verifyMutationScore(file, threshold) {
    def results = readJSON file: file
    def score = results.mutation_score

    if (score < threshold) {
        error("Mutation score ${score}% below threshold ${threshold}%")
    } else {
        echo "✅ Score ${score}% meets threshold ${threshold}%"
    }
}
```

## Advanced Patterns

### Scheduled Nightly Builds

```groovy
pipeline {
    agent any

    triggers {
        // Run every night at 2 AM
        cron('0 2 * * *')
    }

    stages {
        stage('Comprehensive Mutation Testing') {
            steps {
                sh '''
                    pmat mutate --target src/ --output-format json > mutation-results-nightly.json
                    pmat mutate --target src/ --output-format markdown > mutation-report-nightly.md
                '''
            }
        }
    }

    post {
        always {
            archiveArtifacts artifacts: 'mutation-results-nightly.json,mutation-report-nightly.md', fingerprint: true
        }
        failure {
            emailext(
                to: 'dev-team@example.com',
                subject: "Nightly Mutation Testing Failed - Build ${env.BUILD_NUMBER}",
                body: """
                    Nightly mutation testing failed.

                    Build: ${env.BUILD_URL}

                    Check the mutation report for details.
                """,
                attachmentsPattern: 'mutation-report-nightly.md'
            )
        }
    }
}
```

### Differential Mutation Testing

```groovy
pipeline {
    agent any

    stages {
        stage('Get Changed Files') {
            steps {
                script {
                    if (env.CHANGE_ID) {
                        // Pull request build
                        def changedFiles = sh(
                            script: "git diff --name-only origin/${env.CHANGE_TARGET}...HEAD | grep '\\.rs\$' | tr '\\n' ' '",
                            returnStdout: true
                        ).trim()

                        env.CHANGED_FILES = changedFiles

                        if (changedFiles) {
                            echo "Changed Rust files: ${changedFiles}"
                        } else {
                            echo "No Rust files changed, skipping mutation testing"
                            currentBuild.result = 'SUCCESS'
                            return
                        }
                    }
                }
            }
        }

        stage('Differential Mutation Testing') {
            when {
                expression { env.CHANGED_FILES != null && env.CHANGED_FILES != '' }
            }
            steps {
                script {
                    def files = env.CHANGED_FILES.split(' ')
                    files.each { file ->
                        echo "Testing ${file}..."
                        sh "pmat mutate --target ${file} --failures-only --output-format markdown >> mutation-report-diff.md"
                    }
                }
            }
        }
    }

    post {
        always {
            archiveArtifacts artifacts: 'mutation-report-diff.md', allowEmptyArchive: true
        }
    }
}
```

### Parallel Module Testing

```groovy
pipeline {
    agent any

    stages {
        stage('Parallel Mutation Testing') {
            parallel {
                stage('Auth Module') {
                    steps {
                        sh 'pmat mutate --target src/auth/ --threshold 90 --output-format json > mutation-auth.json'
                    }
                }

                stage('API Module') {
                    steps {
                        sh 'pmat mutate --target src/api/ --threshold 85 --output-format json > mutation-api.json'
                    }
                }

                stage('Database Module') {
                    steps {
                        sh 'pmat mutate --target src/database/ --threshold 80 --output-format json > mutation-database.json'
                    }
                }

                stage('Utils Module') {
                    steps {
                        sh 'pmat mutate --target src/utils/ --threshold 75 --output-format json > mutation-utils.json'
                    }
                }
            }
        }

        stage('Aggregate Results') {
            steps {
                script {
                    def totalMutants = 0
                    def totalKilled = 0

                    ['auth', 'api', 'database', 'utils'].each { module ->
                        def results = readJSON file: "mutation-${module}.json"
                        totalMutants += results.total_mutants
                        totalKilled += results.killed
                    }

                    def overallScore = (totalKilled / totalMutants) * 100

                    echo "📊 Overall Mutation Score: ${overallScore}%"
                    echo "Total Mutants: ${totalMutants}"
                    echo "Total Killed: ${totalKilled}"

                    // Save aggregate report
                    writeFile file: 'mutation-summary.txt', text: """
Mutation Testing Summary
========================
Overall Score: ${overallScore}%
Total Mutants: ${totalMutants}
Total Killed: ${totalKilled}
                    """
                }
            }
        }
    }
}
```

## Blue Ocean Integration

### Blue Ocean Pipeline

```groovy
pipeline {
    agent {
        docker {
            image 'rust:latest'
        }
    }

    stages {
        stage('Build') {
            steps {
                echo 'Building application...'
                sh 'cargo build --release'
            }
        }

        stage('Test') {
            steps {
                echo 'Running unit tests...'
                sh 'cargo test'
            }
        }

        stage('Mutation Testing') {
            steps {
                echo 'Running mutation testing...'
                sh 'pmat mutate --target src/ --output-format json > mutation-results.json'

                script {
                    def results = readJSON file: 'mutation-results.json'
                    def score = results.mutation_score

                    // Blue Ocean recognizes this status
                    if (score >= 90) {
                        currentBuild.description = "🟢 Mutation Score: ${score}%"
                    } else if (score >= 80) {
                        currentBuild.description = "🟡 Mutation Score: ${score}%"
                    } else {
                        currentBuild.description = "🔴 Mutation Score: ${score}%"
                    }
                }
            }
        }
    }

    post {
        always {
            archiveArtifacts artifacts: 'mutation-results.json', fingerprint: true
        }
        success {
            echo '✅ Build succeeded!'
        }
        unstable {
            echo '⚠️  Build unstable!'
        }
        failure {
            echo '❌ Build failed!'
        }
    }
}
```

## Artifacts and Reports

### Multiple Output Formats

```groovy
pipeline {
    agent any

    stages {
        stage('Mutation Testing - All Formats') {
            steps {
                sh '''
                    pmat mutate --target src/ --output-format text > mutation-report.txt
                    pmat mutate --target src/ --output-format json > mutation-results.json
                    pmat mutate --target src/ --output-format markdown > mutation-report.md
                '''
            }
        }
    }

    post {
        always {
            archiveArtifacts artifacts: 'mutation-*', fingerprint: true

            // Publish HTML report
            publishHTML([
                allowMissing: false,
                alwaysLinkToLastBuild: true,
                keepAll: true,
                reportDir: '.',
                reportFiles: 'mutation-report.md',
                reportName: 'Mutation Testing Report',
                reportTitles: ''
            ])
        }
    }
}
```

### JUnit XML Integration

```groovy
pipeline {
    agent any

    stages {
        stage('Mutation Testing') {
            steps {
                sh 'pmat mutate --target src/ --output-format json > mutation-results.json'

                script {
                    // Convert JSON to JUnit XML
                    def results = readJSON file: 'mutation-results.json'

                    def junitXml = """<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="Mutation Testing" tests="${results.total_mutants}" failures="${results.survived}">
"""
                    results.mutants.each { mutant ->
                        def status = mutant.status
                        def location = mutant.location
                        def mutation = mutant.mutation

                        junitXml += """  <testcase classname="mutation" name="${location}">
"""
                        if (status == "Survived") {
                            junitXml += """    <failure message="Mutation survived: ${mutation}"/>
"""
                        }
                        junitXml += """  </testcase>
"""
                    }
                    junitXml += "</testsuite>"

                    writeFile file: 'mutation-junit.xml', text: junitXml
                }
            }
        }
    }

    post {
        always {
            junit 'mutation-junit.xml'
        }
    }
}
```

## Pipeline Libraries

### Shared Library

`vars/mutationTest.groovy`:

```groovy
def call(Map config = [:]) {
    def target = config.target ?: 'src/'
    def threshold = config.threshold ?: 85
    def outputFormat = config.outputFormat ?: 'json'
    def failuresOnly = config.failuresOnly ?: false

    def command = "pmat mutate --target ${target} --threshold ${threshold} --output-format ${outputFormat}"

    if (failuresOnly) {
        command += " --failures-only"
    }

    command += " > mutation-results.${outputFormat}"

    sh command

    if (outputFormat == 'json') {
        def results = readJSON file: "mutation-results.${outputFormat}"
        def score = results.mutation_score

        echo "Mutation Score: ${score}%"

        if (score < threshold) {
            error("Mutation score ${score}% below threshold ${threshold}%")
        }

        return results
    }
}
```

**Usage**:

```groovy
@Library('shared-pipeline-library') _

pipeline {
    agent any

    stages {
        stage('Mutation Testing') {
            steps {
                script {
                    mutationTest(
                        target: 'src/',
                        threshold: 90,
                        outputFormat: 'json',
                        failuresOnly: false
                    )
                }
            }
        }
    }
}
```

### Reusable Functions

`vars/pmatUtils.groovy`:

```groovy
def installPmat() {
    sh '''
        if [ ! -f /usr/local/cargo/bin/pmat ]; then
            cargo install pmat
        else
            echo "PMAT already installed"
            pmat --version
        fi
    '''
}

def runMutationTests(target, threshold) {
    sh "pmat mutate --target ${target} --threshold ${threshold} --output-format json > mutation-results.json"

    def results = readJSON file: 'mutation-results.json'
    return results
}

def verifyMutationScore(results, threshold) {
    def score = results.mutation_score

    if (score < threshold) {
        error("Mutation score ${score}% below threshold ${threshold}%")
    } else {
        echo "✅ Mutation score ${score}% meets threshold ${threshold}%"
    }
}
```

**Usage**:

```groovy
@Library('shared-pipeline-library') _

pipeline {
    agent any

    stages {
        stage('Setup') {
            steps {
                script {
                    pmatUtils.installPmat()
                }
            }
        }

        stage('Mutation Testing') {
            steps {
                script {
                    def results = pmatUtils.runMutationTests('src/', 85)
                    pmatUtils.verifyMutationScore(results, 85)
                }
            }
        }
    }
}
```

## Distributed Builds

### Agent Labels

```groovy
pipeline {
    agent none

    stages {
        stage('Build') {
            agent {
                label 'linux && rust'
            }
            steps {
                sh 'cargo build --release'
            }
        }

        stage('Mutation Testing') {
            agent {
                label 'linux && rust && high-cpu'
            }
            steps {
                sh 'pmat mutate --target src/ --jobs 8'
            }
        }
    }
}
```

### Kubernetes Agent

```groovy
pipeline {
    agent {
        kubernetes {
            yaml """
apiVersion: v1
kind: Pod
metadata:
  labels:
    jenkins: agent
spec:
  containers:
  - name: rust
    image: rust:1.75.0
    command:
    - cat
    tty: true
    resources:
      requests:
        memory: "4Gi"
        cpu: "2"
      limits:
        memory: "8Gi"
        cpu: "4"
"""
        }
    }

    stages {
        stage('Install PMAT') {
            steps {
                container('rust') {
                    sh 'cargo install pmat'
                }
            }
        }

        stage('Mutation Testing') {
            steps {
                container('rust') {
                    sh 'pmat mutate --target src/ --jobs 4'
                }
            }
        }
    }
}
```

## Troubleshooting

### Long-Running Tests

```groovy
pipeline {
    agent any

    options {
        timeout(time: 2, unit: 'HOURS')
    }

    stages {
        stage('Mutation Testing') {
            steps {
                sh 'pmat mutate --target src/ --timeout 60 --jobs 2'
            }
        }
    }
}
```

### Memory Issues

```groovy
pipeline {
    agent {
        docker {
            image 'rust:latest'
            args '-m 8g --memory-swap 8g'
        }
    }

    environment {
        CARGO_BUILD_JOBS = '2'
    }

    stages {
        stage('Mutation Testing') {
            steps {
                sh 'pmat mutate --target src/ --jobs 1'
            }
        }
    }
}
```

### Debug Mode

```groovy
pipeline {
    agent any

    environment {
        RUST_LOG = 'debug'
        RUST_BACKTRACE = 'full'
    }

    stages {
        stage('Mutation Testing Debug') {
            steps {
                sh 'pmat mutate --target src/ --verbose 2>&1 | tee mutation-debug.log'
            }
        }
    }

    post {
        always {
            archiveArtifacts artifacts: 'mutation-debug.log', allowEmptyArchive: true
        }
    }
}
```

## Best Practices

1. **Cache PMAT binary** - Use Docker volumes or Jenkins workspace caching
2. **Run mutation tests after unit tests** - Don't waste time on broken code
3. **Use parallel stages** - Test multiple modules concurrently
4. **Set appropriate timeouts** - Prevent hanging builds
5. **Archive artifacts** - Keep historical mutation reports for trend analysis
6. **Use Blue Ocean** - Better visualization of pipeline stages
7. **Create shared libraries** - Reuse mutation testing logic across projects
8. **Schedule comprehensive runs** - Run full mutation testing nightly
9. **Use distributed builds** - Leverage Jenkins agents for parallel execution
10. **Integrate with notifications** - Email/Slack alerts for failed builds

## Complete Production Example

`Jenkinsfile` for production-ready mutation testing:

```groovy
@Library('shared-pipeline-library') _

pipeline {
    agent none

    options {
        timeout(time: 2, unit: 'HOURS')
        timestamps()
        buildDiscarder(logRotator(numToKeepStr: '30', artifactNumToKeepStr: '30'))
        disableConcurrentBuilds()
    }

    parameters {
        choice(name: 'MUTATION_MODE', choices: ['fast', 'full', 'differential'], description: 'Mutation testing mode')
        string(name: 'THRESHOLD', defaultValue: '85', description: 'Mutation score threshold')
    }

    environment {
        CARGO_HOME = "${WORKSPACE}/.cargo"
        RUST_VERSION = '1.75.0'
    }

    stages {
        stage('Checkout') {
            agent {
                label 'linux'
            }
            steps {
                checkout scm
            }
        }

        stage('Build') {
            agent {
                docker {
                    image "rust:${RUST_VERSION}"
                    args '-v cargo-cache:/usr/local/cargo'
                    reuseNode true
                }
            }
            steps {
                sh 'cargo build --release'
                stash includes: 'target/release/**', name: 'release-build'
            }
        }

        stage('Unit Tests') {
            agent {
                docker {
                    image "rust:${RUST_VERSION}"
                    reuseNode true
                }
            }
            steps {
                sh 'cargo test --all-features'
            }
        }

        stage('Integration Tests') {
            agent {
                docker {
                    image "rust:${RUST_VERSION}"
                    reuseNode true
                }
            }
            steps {
                sh 'cargo test --test integration_tests'
            }
        }

        stage('Mutation Testing') {
            agent {
                docker {
                    image "rust:${RUST_VERSION}"
                    args '-v cargo-cache:/usr/local/cargo -m 8g --memory-swap 8g'
                    reuseNode true
                }
            }
            steps {
                script {
                    unstash 'release-build'

                    // Install PMAT if not cached
                    sh '''
                        if [ ! -f /usr/local/cargo/bin/pmat ]; then
                            cargo install pmat
                        else
                            pmat --version
                        fi
                    '''

                    // Choose mutation testing strategy
                    switch(params.MUTATION_MODE) {
                        case 'fast':
                            sh 'pmat mutate --target src/ --failures-only --output-format json > mutation-results.json'
                            break

                        case 'full':
                            parallel(
                                'auth': {
                                    sh 'pmat mutate --target src/auth/ --threshold 90 --output-format json > mutation-auth.json'
                                },
                                'api': {
                                    sh 'pmat mutate --target src/api/ --threshold 85 --output-format json > mutation-api.json'
                                },
                                'database': {
                                    sh 'pmat mutate --target src/database/ --threshold 80 --output-format json > mutation-database.json'
                                },
                                'utils': {
                                    sh 'pmat mutate --target src/utils/ --threshold 75 --output-format json > mutation-utils.json'
                                }
                            )
                            break

                        case 'differential':
                            if (env.CHANGE_ID) {
                                def changedFiles = sh(
                                    script: "git diff --name-only origin/${env.CHANGE_TARGET}...HEAD | grep '\\.rs\$' | tr '\\n' ' '",
                                    returnStdout: true
                                ).trim()

                                if (changedFiles) {
                                    def files = changedFiles.split(' ')
                                    files.each { file ->
                                        sh "pmat mutate --target ${file} --failures-only --output-format markdown >> mutation-report-diff.md"
                                    }
                                } else {
                                    echo "No Rust files changed, skipping differential mutation testing"
                                }
                            }
                            break
                    }

                    // Generate aggregate report for full mode
                    if (params.MUTATION_MODE == 'full') {
                        def totalMutants = 0
                        def totalKilled = 0

                        ['auth', 'api', 'database', 'utils'].each { module ->
                            if (fileExists("mutation-${module}.json")) {
                                def results = readJSON file: "mutation-${module}.json"
                                totalMutants += results.total_mutants
                                totalKilled += results.killed
                            }
                        }

                        def overallScore = totalMutants > 0 ? (totalKilled / totalMutants) * 100 : 0

                        writeFile file: 'mutation-summary.md', text: """
# 🧬 Mutation Testing Summary

**Overall Score:** ${overallScore}%

## Module Breakdown

| Module | Score | Threshold | Status |
|--------|-------|-----------|--------|
"""
                        ['auth': 90, 'api': 85, 'database': 80, 'utils': 75].each { module, threshold ->
                            if (fileExists("mutation-${module}.json")) {
                                def results = readJSON file: "mutation-${module}.json"
                                def score = results.mutation_score
                                def status = score >= threshold ? '✅' : '❌'
                                writeFile file: 'mutation-summary.md', text: readFile('mutation-summary.md') +
                                    "| ${module} | ${score}% | ${threshold}% | ${status} |\n"
                            }
                        }

                        echo readFile('mutation-summary.md')

                        // Set build status
                        if (overallScore < params.THRESHOLD.toFloat()) {
                            currentBuild.result = 'UNSTABLE'
                            error("Overall mutation score ${overallScore}% below threshold ${params.THRESHOLD}%")
                        }
                    }
                }
            }
        }
    }

    post {
        always {
            node('linux') {
                archiveArtifacts artifacts: 'mutation-*.json,mutation-*.md', allowEmptyArchive: true, fingerprint: true

                publishHTML([
                    allowMissing: true,
                    alwaysLinkToLastBuild: true,
                    keepAll: true,
                    reportDir: '.',
                    reportFiles: 'mutation-summary.md',
                    reportName: 'Mutation Testing Report'
                ])
            }
        }
        success {
            echo '✅ Pipeline succeeded!'
        }
        unstable {
            emailext(
                to: 'dev-team@example.com',
                subject: "⚠️  Mutation Testing Unstable - ${env.JOB_NAME} #${env.BUILD_NUMBER}",
                body: """
                    Mutation testing results below threshold.

                    Build: ${env.BUILD_URL}

                    Check the mutation report for details.
                """,
                attachmentsPattern: 'mutation-summary.md'
            )
        }
        failure {
            emailext(
                to: 'dev-team@example.com',
                subject: "❌ Pipeline Failed - ${env.JOB_NAME} #${env.BUILD_NUMBER}",
                body: """
                    Pipeline failed.

                    Build: ${env.BUILD_URL}

                    Check the console output for details.
                """
            )
        }
    }
}
```

## Additional Resources

- **PMAT Documentation**: `server/README.md`
- **Jenkins Pipeline Docs**: https://www.jenkins.io/doc/book/pipeline/
- **Blue Ocean Docs**: https://www.jenkins.io/doc/book/blueocean/
- **Mutation Testing Concepts**: `examples/*/README.md`
- **GitHub Actions Guide**: `docs/ci-cd/github-actions-integration.md`
- **GitLab CI Guide**: `docs/ci-cd/gitlab-ci-integration.md`

## Summary

Jenkins provides powerful CI/CD capabilities for mutation testing:

✅ **Declarative and Scripted pipelines** - Choose your preferred syntax
✅ **Docker integration** - Consistent build environments
✅ **Parallel execution** - Test multiple modules concurrently
✅ **Shared libraries** - Reusable mutation testing logic
✅ **Blue Ocean UI** - Modern visualization of pipeline stages
✅ **Distributed builds** - Leverage Jenkins agents and Kubernetes
✅ **Rich plugin ecosystem** - Email, Slack, HTML reports, JUnit integration
✅ **Flexible scheduling** - Cron-based nightly builds
✅ **Artifact management** - Archive and track mutation reports
✅ **Quality gates** - Fail builds on low mutation scores

For production deployments, combine unit tests, integration tests, and mutation testing with appropriate thresholds and quality gates.
