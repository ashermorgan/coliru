# An SSH server with authentication disabled for the user "test"

FROM alpine

RUN apk update && apk add openssh

RUN ssh-keygen -A

RUN echo 'PasswordAuthentication yes' >> /etc/ssh/sshd_config
RUN echo 'PermitEmptyPasswords yes' >> /etc/ssh/sshd_config
RUN echo 'PermitRootLogin yes' >> /etc/ssh/sshd_config

COPY entry.sh /entry.sh

EXPOSE 22

CMD ["/entry.sh"]
