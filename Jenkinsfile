pipeline {
    agent any

    stages {
        stage('Build') {
            steps {
                // Este comando construirá todos los servicios definidos en tu docker-compose.yml
                sh 'docker-compose build'
            }
        }

        stage('Deploy') {
            steps {
                // Detiene y elimina los contenedores existentes
                sh 'docker-compose down'

                // Inicia los nuevos contenedores
                sh 'docker-compose up -d'
            }
        }
    }
}
