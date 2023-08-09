#!/bin/bash
PLEX_PROTOCOL="$(echo $REPLEX_HOST | grep :// | sed -e's,^\(.*://\).*,\1,g')"
PLEX="$(echo ${REPLEX_HOST/$PLEX_PROTOCOL/})"
PLEX_PROTOCOL="${PLEX_PROTOCOL//:\/\//}"
REPLEX_PORT=300
: "${REPLEX_REDIRECT_STREAMS:=0}"
REPLEX_REDIRECT_STREAMS="${REPLEX_REDIRECT_STREAMS/true/1}"
REPLEX_REDIRECT_STREAMS="${REPLEX_REDIRECT_STREAMS/false/0}"

export PLEX
export PLEX_PROTOCOL
export REPLEX_PORT
export REPLEX_REDIRECT_STREAMS
export REPLEX_HOST

/app/replex &

/docker-entrypoint.sh "nginx" &
# nginx  &

# Wait for any process to exit
wait -n

# Exit with status of process that exited first
exit $?