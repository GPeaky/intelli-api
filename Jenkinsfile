pipeline {
    agent any

    stages {
        stage('Checkout') {
            steps {
                checkout scm
            }
        }

        stage('Setup Environment') {
            steps {
                sh 'cp /var/lib/jenkins/secrets/intelli-certs /var/lib/jenkins/workspace/intelli/certs'
                sh 'cp /var/lib/jenkins/secrets/intelli.gerardz.de.key /var/lib/jenkins/workspace/intelli/certs/intelli.gerardz.de.key'
            }
        }

        stage('Build with Docker Compose') {
            steps {
                sh 'docker-compose build'
            }
        }

        stage('Run with Docker Compose') {
            steps {
                sh 'docker-compose up -d'
            }
        }
    }
}
