
#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

readonly TARGET_HOST=pi@raspberrypi
readonly TARGET_PATH=/home/pi/hello-world
readonly TARGET_ARCH=armv7-unknown-linux-gnueabihf
readonly SOURCE_PATH=./target/${TARGET_ARCH}/release/hello-world

cargo build --release --target=${TARGET_ARCH}
rsync ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
ssh -t ${TARGET_HOST} ${TARGET_PATH}


rsync --rsync-path 'sudo -u pi rsync' ./rpsump 192.168.254.189:/home/pi

rsync ./rpsump 192.168.254.189:/home/pi

rsync --rsync-path 'sudo -u jenkins rsync' -avP --delete /var/lib/jenkins destuser@destmachine:/tmp

#docker run -it 0e6bc7b2a702
#sudo docker cp 52df12d81ebf:/usr/src/rpsump/target/debug/rpsump ~/rpsump
#works
rsync --rsync-path 'sudo -u pi rsync' -avP ./rpsump pi@192.168.254.189:/home/pi/rpsump_build
