pipeline {
    agent any

    stages {
        stage('Checkout') {
            steps {
                // Asegura que tienes el código más reciente del repositorio
                checkout scm
            }
        }

        stage('Setup Environment') {
            sh 'cp /var/jenkins_home/secrets/intelli.env .'
        }

        stage('Build with Docker Compose') {
            steps {
                // Construye usando docker-compose
                sh 'docker-compose build'
            }
        }

        stage('Run with Docker Compose') {
            steps {
                // Arranca tu aplicación con docker-compose en background
                sh 'docker-compose up -d'
            }
        }
    }
}
