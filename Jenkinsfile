pipeline {
    agent any

    stages {
        stage('Prepare') {
            steps {
                script {
                    def dockerComposeExists = sh(script: 'command -v docker-compose', returnStatus: true) == 0
                    if (!dockerComposeExists) {
                        sh 'curl -L "https://github.com/docker/compose/releases/download/1.29.2/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose'
                        sh 'chmod +x /usr/local/bin/docker-compose'
                    }
                }
            }
        }

        stage('Build') {
            steps {
                sh 'docker-compose build'
            }
        }

        stage('Deploy') {
            steps {
                sh 'docker-compose down'
                sh 'docker-compose up -d'
            }
        }
    }
}