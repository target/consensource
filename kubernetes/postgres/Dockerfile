FROM postgres:12.4

COPY ./tables/ /docker-entrypoint-initdb.d
COPY ./scripts/ /scripts

RUN apt-get update && apt-get install -y sudo