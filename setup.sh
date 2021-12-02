#!/bin/bash

# stop any running postgres
docker-compose stop psql

# start postgres
docker-compose up -d psql

# wait for postgres to boot
tries=1
until psql -h 127.0.0.1 -U tguard -q -c "select version()" >/dev/null 2>/dev/null; do
  echo "--- Waiting for PostgreSQL...";
  sleep 1;
  let "tries++"
  if [ $tries -eq 10 ]; then
    echo "--- ERROR PostgreSQL did not start";
    exit 1;
  fi;
done

# load database schema
psql -h 127.0.0.1 -U tguard < backend/db.sql

# stop postgres
docker-compose stop psql
