#!/bin/bash
# http="http://"
# https="https://"
# PLEX=$(echo $REPLEX_HOST)
# PLEX=${PLEX#"$https"}
# PLEX=${PLEX#"$http"}
PLEX_PROTOCOL="$(echo $REPLEX_HOST | grep :// | sed -e's,^\(.*://\).*,\1,g')"
PLEX="$(echo ${REPLEX_HOST/$PLEX_PROTOCOL/})"
PLEX_PROTOCOL="${PLEX_PROTOCOL//:\/\//}"
REPLEX_PORT=300
export PLEX
export PLEX_PROTOCOL
export REPLEX_PORT

/app/replex &

/docker-entrypoint.sh "nginx" &
# nginx  &

# Wait for any process to exit
wait -n

# Exit with status of process that exited first
exit $?