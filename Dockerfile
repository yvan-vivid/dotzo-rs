# Simple environment to emulate default linux install
FROM alpine:latest
RUN apk add vim tree sudo tmux zsh openssh-client

# Workaround for a sudo issue
RUN echo "Set disable_coredump false" >> /etc/sudo.conf

# Requirements for dotzo
# bash, git, and rsync are requirements
RUN apk add bash git rsync

# Test user with sudo permission
ARG test_user=tester
RUN adduser -D -s /bin/bash $test_user \
  && echo "$test_user ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/$test_user \
  && chmod 0440 /etc/sudoers.d/$test_user

USER tester
ARG dotzo_dev=/home/tester/dotzo-dev
ENV PATH="$dotzo_dev:${PATH}"
WORKDIR /home/tester

COPY ./target/x86_64-unknown-linux-musl/release/dotzo /usr/bin/dotzo
