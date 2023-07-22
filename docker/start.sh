#!/bin/bash

# TODO: remove protocol for nginx upstream block
http="http://"
https="http://"
NGINX_PLEX=$(echo $REPLEX_HOST)
NGINX_PLEX=${REPLEX_HOST#"$http"}
NGINX_PLEX=${REPLEX_HOST#"$https"}
export NGINX_PLEX

./app/replex &

./docker-entrypoint.sh "nginx-debug" &
# nginx  &

# Wait for any process to exit
wait -n

# Exit with status of process that exited first
exit $?
