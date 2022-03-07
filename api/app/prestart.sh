#!/bin/bash

service redis-server start

if test "$0" = "/start-reload.sh"; then
    exec uvicorn --reload --reload-include="*.so" --host $HOST --port $PORT --log-level $LOG_LEVEL "$APP_MODULE"
fi
