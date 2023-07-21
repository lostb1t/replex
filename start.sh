#!/bin/bash

./app/replex &

./docker-entrypoint.sh "nginx" &
# nginx  &

# Wait for any process to exit
wait -n

# Exit with status of process that exited first
exit $?
