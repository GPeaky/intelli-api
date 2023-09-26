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
                sh 'cp -r /var/lib/jenkins/secrets/intelli-certs /var/lib/jenkins/workspace/intelli/certs'
                sh 'cp /var/lib/jenkins/secrets/intelli.env /var/lib/jenkins/workspace/intelli/.env'
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
