#############
## Builder ##
#############

FROM rust:slim as BUILDER

WORKDIR /usr/src

# Create blank project
RUN cargo new staticpi

# We want dependencies cached, so copy those first
COPY Cargo.* /usr/src/staticpi/

# Set the working directory
WORKDIR /usr/src/staticpi

# This is a dummy build to get the dependencies cached - probably not needed - as run via a github action
RUN cargo build --release

# Now copy in the rest of the sources
COPY src /usr/src/staticpi/src/

## Touch main.rs to prevent cached release build
RUN touch /usr/src/staticpi/src/main.rs

# This is the actual application build
RUN cargo build --release

#############
## Runtime ##
#############

# FROM debian:bullseye-slim AS RUNTIME
FROM ubuntu:22.04 AS RUNTIME

ARG DOCKER_GUID=1000 \
	DOCKER_UID=1000 \
	DOCKER_TIME_CONT=America \
	DOCKER_TIME_CITY=New_York \
	DOCKER_APP_USER=app_user \
	DOCKER_APP_GROUP=app_group

ENV TZ=${DOCKER_TIME_CONT}/${DOCKER_TIME_CITY}

RUN apt-get update \
	&& apt-get install -y ca-certificates wget \
	&& update-ca-certificates \
	&& groupadd --gid ${DOCKER_GUID} ${DOCKER_APP_GROUP} \
	&& useradd --create-home --no-log-init --uid ${DOCKER_UID} --gid ${DOCKER_GUID} ${DOCKER_APP_USER} \
	&& mkdir /logs \
	&& chown ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /logs
	
WORKDIR /app

COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} ./docker/healthcheck/health_api.sh /healthcheck/
RUN chmod +x /healthcheck/health_api.sh

COPY --from=BUILDER /usr/src/staticpi/target/release/staticpi /app/

USER ${DOCKER_APP_USER}

CMD ["/app/staticpi"]