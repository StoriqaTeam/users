node {
    def bin
    def app

    stage('Clone repository') {
        checkout scm
        sh 'git submodule update --init --recursive'
    }

    stage('Build app') {
        sh 'cp -f docker/Dockerfile.build Dockerfile'
        bin = docker.build("storiqateam/stq-users-interm:${env.BRANCH_NAME}")
        sh 'rm -f Dockerfile'
    }

    stage('Get binary') {
        sh "docker run -i --rm --volume ${env.WORKSPACE}:/mnt/ storiqateam/stq-users-interm:${env.BRANCH_NAME} cp -f /app/target/release/users /mnt/"
    }

    stage('Build app image') {
        sh 'cp -f docker/Dockerfile.run Dockerfile'
        app = docker.build("storiqateam/stq-users:${env.BRANCH_NAME}")
        sh 'rm -f Dockerfile'
        sh 'rm -f users'
    }

    stage('Push image') {
        docker.withRegistry('https://registry.hub.docker.com', '4ca2ddae-a205-45f5-aaf7-333789c385cd') {
            app.push("${env.BRANCH_NAME}${env.BUILD_NUMBER}")
            app.push("${env.BRANCH_NAME}")
        }
    }
}
