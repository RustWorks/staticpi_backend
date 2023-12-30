FROM alpine:3.19

ARG DOCKER_GUID=1000 \
	DOCKER_UID=1000 \
	DOCKER_TIME_CONT=America \
	DOCKER_TIME_CITY=New_York \
	DOCKER_APP_USER=app_user \
	DOCKER_APP_GROUP=app_group

ENV TZ=${DOCKER_TIME_CONT}/${DOCKER_TIME_CITY}

RUN apk add --update --no-cache gnupg tzdata age postgresql16-client \
	&& cp /usr/share/zoneinfo/${TZ} /etc/localtime \
	&& echo ${TZ} > /etc/timezone \
	&& addgroup -g ${DOCKER_GUID} -S ${DOCKER_APP_GROUP} \
	&& adduser -u ${DOCKER_UID} -S -G ${DOCKER_APP_GROUP} ${DOCKER_APP_USER} \
	&& mkdir /backups /redis_data /logs

COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} ./backup/create_backup.sh /create_backup.sh
RUN chmod +x /create_backup.sh

USER ${DOCKER_APP_USER}

CMD [ "/create_backup.sh" ]