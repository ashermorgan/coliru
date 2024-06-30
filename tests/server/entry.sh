#!/usr/bin/env sh

PUID=${PUID:-1000}
echo "Creating user test with PUID=$PUID ..."
adduser -h /home/test -s /bin/sh -D -u "$PUID" test
passwd -d test

echo "Starting sshd ..."
/usr/sbin/sshd -D
